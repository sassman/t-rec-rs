//! Recording session orchestration.
//!
//! The `RecordingSession` is the main entry point for recording. It:
//! - Manages the lifecycle of all recording actors (capture, input, shell)
//! - Coordinates startup and shutdown
//! - Collects results for post-processing

use crate::capture::{capture_thread, CaptureContext};
use crate::common::utils::clear_screen;
use crate::common::PlatformApi;
use crate::config::defaults::FPS;
use crate::config::ProfileSettings;
use crate::event_router::*;
use crate::input::{HotkeyConfig, InputState, KeyboardMonitor};
use crate::output::OutputConfig;
use crate::screenshot::ScreenshotInfo;
use crate::types::{BackgroundColor, Decor};

use anyhow::Context;
use image::DynamicImage;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

use super::runtime::{Actor, Runtime};
use crate::{Result, WindowId};

#[cfg(unix)]
use crate::pty::PtyShell;

/// Configuration for post-processing effects.
///
/// Uses type-safe enums for decoration and background color configuration.
#[derive(Clone)]
pub struct PostProcessConfig {
    /// Decoration style (None or Shadow)
    pub decor: Decor,
    /// Background color for shadow effect
    pub bg_color: BackgroundColor,
    /// Optional pre-validated wallpaper configuration (image, padding)
    pub wallpaper: Option<(DynamicImage, u32)>,
    /// Pause to add at the start of the GIF
    pub start_delay: Duration,
    /// Pause to add at the end of the GIF
    pub end_delay: Duration,
    /// Frames per second
    pub fps: u8,
}

/// Configuration for a recording session.
///
/// This struct contains all the configuration needed to run a recording session.
/// It is typically built from `ProfileSettings` combined with runtime values.
pub struct SessionConfig {
    /// Window to record
    pub win_id: WindowId,
    /// Optional window name (for display purposes)
    pub window_name: Option<String>,
    /// Shell program to spawn
    pub program: String,
    /// Frames per second
    pub fps: u8,
    /// Natural mode (variable timing based on activity)
    pub natural: bool,
    /// Pause capture when idle (duration to wait before pausing)
    pub idle_pause: Option<Duration>,
    /// Output file path (without extension)
    pub output_path: PathBuf,
    /// Generate GIF output
    pub generate_gif: bool,
    /// Generate MP4 output
    pub generate_video: bool,
    /// Verbose logging
    pub verbose: bool,
    /// Quiet mode (no output)
    pub quiet: bool,
    /// Post-processing configuration
    pub post_process: PostProcessConfig,
}

impl SessionConfig {
    /// Create a new SessionConfig builder.
    pub fn builder() -> SessionConfigBuilder {
        SessionConfigBuilder::default()
    }
}

/// Builder for `SessionConfig`.
#[derive(Default)]
pub struct SessionConfigBuilder {
    win_id: Option<WindowId>,
    window_name: Option<String>,
    program: Option<String>,
    fps: Option<u8>,
    natural: Option<bool>,
    idle_pause: Option<Duration>,
    output_path: Option<PathBuf>,
    generate_gif: Option<bool>,
    generate_video: Option<bool>,
    verbose: Option<bool>,
    quiet: Option<bool>,
    decor: Option<Decor>,
    bg_color: Option<BackgroundColor>,
    wallpaper: Option<(DynamicImage, u32)>,
    start_delay: Option<Duration>,
    end_delay: Option<Duration>,
}

impl SessionConfigBuilder {
    /// Set the window ID to record.
    pub fn win_id(mut self, win_id: WindowId) -> Self {
        self.win_id = Some(win_id);
        self
    }

    /// Set the optional window name.
    pub fn window_name(mut self, name: Option<String>) -> Self {
        self.window_name = name;
        self
    }

    /// Set the shell program to spawn.
    pub fn program(mut self, program: String) -> Self {
        self.program = Some(program);
        self
    }

    /// Apply settings from a ProfileSettings.
    pub fn using_profile(mut self, settings: &ProfileSettings) -> Self {
        self.fps = Some(settings.fps());
        self.natural = Some(settings.natural());
        self.verbose = Some(settings.verbose());
        self.quiet = Some(settings.quiet());
        self.generate_gif = Some(!settings.video_only());
        self.generate_video = Some(settings.video() || settings.video_only());
        self.output_path = Some(PathBuf::from(settings.output()));
        // Parse decor and bg from config strings to enums
        self.decor = Some(settings.decor().parse().unwrap_or_default());
        self.bg_color = Some(settings.bg().parse().unwrap_or_default());
        self
    }

    /// Set the idle pause duration.
    pub fn idle_pause(mut self, duration: Option<Duration>) -> Self {
        self.idle_pause = duration;
        self
    }

    /// Set the start delay for GIF output.
    pub fn start_delay(mut self, delay: Duration) -> Self {
        self.start_delay = Some(delay);
        self
    }

    /// Set the end delay for GIF output.
    pub fn end_delay(mut self, delay: Duration) -> Self {
        self.end_delay = Some(delay);
        self
    }

    /// Set the pre-validated wallpaper configuration.
    pub fn wallpaper(mut self, wallpaper: Option<(DynamicImage, u32)>) -> Self {
        self.wallpaper = wallpaper;
        self
    }

    /// Build the SessionConfig.
    ///
    /// # Panics
    /// Panics if required fields are not set (win_id, program).
    pub fn build(self) -> SessionConfig {
        SessionConfig {
            win_id: self.win_id.expect("win_id is required"),
            window_name: self.window_name,
            program: self.program.expect("program is required"),
            fps: self.fps.unwrap_or(FPS),
            natural: self.natural.unwrap_or(false),
            idle_pause: self.idle_pause,
            output_path: self.output_path.unwrap_or_else(|| PathBuf::from("t-rec")),
            generate_gif: self.generate_gif.unwrap_or(true),
            generate_video: self.generate_video.unwrap_or(false),
            verbose: self.verbose.unwrap_or(false),
            quiet: self.quiet.unwrap_or(false),
            post_process: PostProcessConfig {
                decor: self.decor.unwrap_or_default(),
                bg_color: self.bg_color.unwrap_or_default(),
                wallpaper: self.wallpaper,
                start_delay: self.start_delay.unwrap_or(Duration::ZERO),
                end_delay: self.end_delay.unwrap_or(Duration::ZERO),
                fps: self.fps.unwrap_or(FPS),
            },
        }
    }
}

/// Result of a completed recording session.
pub struct RecordingResult {
    /// Number of frames captured
    pub frame_count: usize,
    /// Screenshots taken during recording
    pub screenshots: Vec<ScreenshotInfo>,
    /// Temporary directory containing frame files
    pub tempdir: Arc<Mutex<TempDir>>,
    /// Time codes for each frame (milliseconds since start)
    pub time_codes: Arc<Mutex<Vec<u128>>>,
}

/// Orchestrates a complete recording session.
pub struct RecordingSession {
    config: SessionConfig,
    api: Box<dyn PlatformApi>,
    runtime: Runtime,
    // todo: introdcue the `CatpureContext` on this level
}

impl RecordingSession {
    /// Create a new recording session with the given configuration.
    pub fn new(config: SessionConfig, api: Box<dyn PlatformApi>, runtime: Runtime) -> Result<Self> {
        Ok(Self {
            config,
            api,
            runtime,
        })
    }

    /// Run the complete recording session.
    ///
    /// This method blocks until recording is complete (user presses Ctrl+D).
    /// Returns the recording result for post-processing.
    pub fn run(self) -> Result<RecordingResult> {
        let config = self.config;
        let api = self.api;
        let mut runtime = self.runtime;

        // Create temp directory for frames
        let tempdir = Arc::new(Mutex::new(
            TempDir::new().context("Cannot create tempdir.")?,
        ));
        let time_codes = Arc::new(Mutex::new(Vec::new()));
        let screenshots = Arc::new(Mutex::new(Vec::<ScreenshotInfo>::new()));

        let router = EventRouter::new();

        // Create input state for screenshot flag
        let input_state = Arc::new(InputState::new());

        // Shared idle duration tracking (for accurate timecodes)
        let idle_duration = Arc::new(Mutex::new(Duration::from_millis(0)));
        let recording_start = Instant::now();

        runtime.spawn(Actor::Photographer, {
            let ctx = CaptureContext {
                win_id: config.win_id,
                time_codes: time_codes.clone(),
                tempdir: tempdir.clone(),
                natural: config.natural,
                idle_pause: config.idle_pause,
                fps: config.fps,
                screenshots: Some(screenshots.clone()),
            };
            let event_rx = router.subscribe();
            move || -> Result<()> { capture_thread(event_rx, api, ctx) }
        });

        clear_screen();

        // Print verbose info
        if config.verbose {
            println!(
                "[t-rec]: Frame cache dir: {:?}",
                tempdir.lock().expect("Cannot lock tempdir resource").path()
            );
            if let Some(ref window) = config.window_name {
                println!("[t-rec]: Recording window: {:?}", window);
            } else {
                println!("[t-rec]: Recording window id: {}", config.win_id);
            }
        }

        // Print startup messages and countdown
        if !config.quiet {
            println!("[t-rec]: Press Ctrl+D to end recording");
            println!("[t-rec]: F2 = Screenshot");

            // Countdown before starting
            for i in (1..=3).rev() {
                print!("\r[t-rec]: Recording starts in {}...", i);
                io::stdout().flush().ok();
                thread::sleep(Duration::from_secs(1));
            }
            print!("\r[t-rec]: Recording!                         \n");
            io::stdout().flush().ok();
            thread::sleep(Duration::from_millis(250));
        }

        clear_screen();

        Self::run_recording(
            runtime,
            &config,
            router,
            input_state,
            idle_duration,
            recording_start,
        )?;

        // restore the terminal state
        // Clear screen
        print!("\x1b[2J\x1b[H");
        io::stdout().flush().ok();

        // Collect results
        let frame_count = time_codes.lock().unwrap().len();
        let screenshots_result = screenshots.lock().unwrap().clone();

        Ok(RecordingResult {
            frame_count,
            screenshots: screenshots_result,
            tempdir,
            time_codes,
        })
    }

    fn run_recording(
        mut runtime: Runtime,
        config: &SessionConfig,
        router: EventRouter,
        input_state: Arc<InputState>,
        idle_duration: Arc<Mutex<Duration>>,
        recording_start: Instant,
    ) -> Result<()> {
        let mut pty_shell = PtyShell::spawn(&config.program)?;
        let shell_stdin = pty_shell.get_writer()?;

        // Spawn shell forwarder actor
        runtime.spawn(Actor::ShellForwarder, {
            let event_rx = router.subscribe();
            move || pty_shell.forward_output(event_rx)
        });

        // Create keyboard monitor
        let hotkey_config = HotkeyConfig::default();
        let keyboard_monitor = KeyboardMonitor::new(
            input_state,
            idle_duration,
            recording_start,
            hotkey_config,
            router.clone(),
        );

        // Spawn keyboard monitor actor
        runtime.spawn(Actor::InputHandler, {
            let event_rx = router.subscribe();
            move || {
                keyboard_monitor.run(shell_stdin, event_rx)?;
                Ok(())
            }
        });

        use super::presenter::{create_presenter, Presenter};

        #[cfg(all(target_os = "macos", feature = "osd-flash-indicator"))]
        log::debug!("OSD flash indicator: enabled (Skylight)");

        #[cfg(not(all(target_os = "macos", feature = "osd-flash-indicator")))]
        log::debug!("OSD flash indicator: disabled (no-op)");

        let mut presenter = create_presenter(config.win_id);

        // Shell bootstrap delay
        thread::sleep(Duration::from_millis(350));

        // Send start event to capture thread
        router
            .try_send(Event::Capture(CaptureEvent::Start))
            .context("Cannot start capture thread")?;

        // blocking the main thread here (macOS Skylight requirement)
        presenter.run(router.subscribe())?;

        // Signal all actors to stop
        router.shutdown();

        // Wait for all actors to complete
        runtime.join_all()?;

        Ok(())
    }

    /// Build the output configuration from the session configuration.
    pub(crate) fn output_config(&self) -> crate::output::OutputConfig {
        OutputConfig {
            output_path: self.config.output_path.clone(),
            generate_gif: self.config.generate_gif,
            generate_video: self.config.generate_video,
            quiet: self.config.quiet,
            post_process: self.config.post_process.clone(),
        }
    }
}
