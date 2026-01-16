//! Headless recorder for programmatic window recording.
//!
//! This module provides a simple API for recording any window to GIF and/or MP4 files
//! without needing a PTY or shell. This is useful for recording animations,
//! GUI applications, or any other visual content.
//!
//! # Features
//!
//! - **Post-processing pipeline**: Apply shadow effects, corner radius, and wallpaper backgrounds
//! - **Multiple output formats**: Generate GIF, MP4, or both
//! - **Builder pattern**: Ergonomic configuration with sensible defaults
//! - **Type-safe configuration**: Use enums for decor, background color, and wallpaper options
//!
//! # Example
//!
//! ```ignore
//! use t_rec::HeadlessRecorder;
//! use t_rec::types::{Decor, BackgroundColor};
//! use t_rec::wallpapers::Wallpaper;
//!
//! // Type-safe enum API (compile-time checked)
//! let recorder = HeadlessRecorder::builder()
//!     .window_id(12345)
//!     .fps(30)
//!     .decor(Decor::Shadow)
//!     .bg_color(BackgroundColor::White)
//!     .wallpaper(Wallpaper::Ventura, 60)
//!     .output_gif("demo.gif")
//!     .build()?;
//!
//! recorder.start()?;
//! // ... run your animation ...
//! let outputs = recorder.stop_and_generate()?;
//! println!("Generated: {:?}", outputs);
//! ```

use crate::capture::{capture_thread, CaptureContext};
use crate::common::{Platform, PlatformApi, PlatformApiFactory};
use crate::event_router::{CaptureEvent, Event, EventRouter};
use crate::generators::{check_for_gif, check_for_mp4, generate_gif, generate_mp4};
use crate::post_processing::{post_process_effects, PostProcessingOptions};
use crate::types::{BackgroundColor, Decor};
use crate::utils::{file_name_for, DEFAULT_EXT, IMG_EXT, MOVIE_EXT};
use crate::wallpapers::{
    get_ventura_wallpaper, load_and_validate_wallpaper, Wallpaper, WallpaperConfig,
};
use crate::WindowId;

use anyhow::{bail, Context, Result};
use image::{DynamicImage, GenericImageView};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use tempfile::TempDir;

/// Configuration for the headless recorder.
///
/// This struct uses strongly-typed enums for configuration options, providing
/// compile-time safety and clear documentation of valid values.
#[derive(Debug, Clone)]
pub struct HeadlessRecorderConfig {
    /// Window ID to capture.
    pub window_id: WindowId,
    /// Frames per second (1-60, default 15).
    pub fps: u8,
    /// If true, save all frames without idle detection.
    pub natural: bool,
    /// Maximum pause duration to preserve (None = skip all identical frames).
    pub idle_pause: Option<Duration>,
    /// Pause to add at the start of the GIF.
    pub start_pause: Option<Duration>,
    /// Pause to add at the end of the GIF.
    pub end_pause: Option<Duration>,

    // Post-processing options
    /// Decoration style (None or Shadow).
    pub decor: Decor,
    /// Background color for shadow effect.
    pub bg_color: BackgroundColor,
    /// Wallpaper and padding configuration.
    pub wallpaper: Option<WallpaperConfig>,

    // Output options
    /// Whether to generate GIF output.
    pub generate_gif: bool,
    /// Whether to generate MP4 output.
    pub generate_mp4: bool,
    /// Output path for GIF (if generate_gif is true).
    pub gif_path: Option<PathBuf>,
    /// Output path for MP4 (if generate_mp4 is true).
    pub mp4_path: Option<PathBuf>,
}

/// Result of the recording, containing paths to all generated output files.
#[derive(Debug, Clone)]
pub struct RecordingOutput {
    /// Path to the generated GIF file (if GIF generation was enabled).
    pub gif_path: Option<PathBuf>,
    /// Path to the generated MP4 file (if MP4 generation was enabled).
    pub mp4_path: Option<PathBuf>,
    /// Number of frames captured.
    pub frame_count: usize,
}

/// Builder for creating a [`HeadlessRecorder`].
///
/// The builder uses type-safe enums for all configuration options.
/// Validation happens at enum construction time (via factory methods),
/// so the builder itself is always infallible.
///
/// # Example
///
/// ```ignore
/// use t_rec::types::{Decor, BackgroundColor};
/// use t_rec::wallpapers::Wallpaper;
///
/// // Type-safe enum API (compile-time checked)
/// let recorder = HeadlessRecorderBuilder::new()
///     .window_id(window.window_id())
///     .fps(30)
///     .decor(Decor::Shadow)
///     .bg_color(BackgroundColor::White)
///     .wallpaper(Wallpaper::Ventura, 60)
///     .output_gif("animation.gif")
///     .build()?;
///
/// // Custom colors and wallpapers use factory methods for validation
/// let recorder = HeadlessRecorderBuilder::new()
///     .window_id(window.window_id())
///     .bg_color(BackgroundColor::custom("#ff0000")?)  // Validates hex format
///     .wallpaper(Wallpaper::custom("/path/to/bg.png")?, 80)  // Validates path exists
///     .output_gif("animation.gif")
///     .build()?;
/// ```
#[derive(Debug, Clone)]
pub struct HeadlessRecorderBuilder {
    window_id: Option<WindowId>,
    fps: u8,
    natural: bool,
    idle_pause: Option<Duration>,
    start_pause: Option<Duration>,
    end_pause: Option<Duration>,

    // Post-processing (type-safe enums)
    decor: Decor,
    bg_color: BackgroundColor,
    wallpaper: Option<WallpaperConfig>,

    // Output
    generate_gif: bool,
    generate_mp4: bool,
    gif_path: Option<PathBuf>,
    mp4_path: Option<PathBuf>,
}

impl Default for HeadlessRecorderBuilder {
    fn default() -> Self {
        Self {
            window_id: None,
            fps: 15,
            natural: false,
            idle_pause: None,
            start_pause: None,
            end_pause: None,
            // Using type-safe enums for configuration
            decor: Decor::Shadow,
            bg_color: BackgroundColor::Transparent,
            wallpaper: None,
            generate_gif: false,
            generate_mp4: false,
            gif_path: None,
            mp4_path: None,
        }
    }
}

impl HeadlessRecorderBuilder {
    /// Create a new builder with default settings.
    ///
    /// Default values:
    /// - fps: 15
    /// - decor: "shadow"
    /// - bg_color: "#00000000" (transparent)
    /// - natural: false (skip identical frames)
    /// - idle_pause: None
    pub fn new() -> Self {
        Self::default()
    }

    /// Set default options for demo recordings.
    ///
    /// Sets:
    /// - fps: 15
    /// - decor: None (no decoration)
    /// - wallpaper: Ventura with 60px padding
    pub fn with_defaults_for_demo(mut self) -> Self {
        self.fps = 15;
        self.decor = Decor::None;
        self.wallpaper(Wallpaper::Ventura, 60)
    }

    /// Set the window ID to record.
    ///
    /// This is required. Use `window.window_id()` from core-animation to get the ID.
    pub fn window_id(mut self, id: WindowId) -> Self {
        self.window_id = Some(id);
        self
    }

    /// Set the frames per second (1-60).
    ///
    /// Higher FPS means smoother animations but larger files.
    /// Default is 15 FPS.
    pub fn fps(mut self, fps: u8) -> Self {
        self.fps = fps.clamp(1, 60);
        self
    }

    /// Set the output GIF file path.
    ///
    /// Calling this enables GIF generation. If you want both GIF and MP4,
    /// call both [`output_gif`](Self::output_gif) and [`output_mp4`](Self::output_mp4).
    pub fn output_gif(mut self, path: impl AsRef<Path>) -> Self {
        self.generate_gif = true;
        self.gif_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the output MP4 file path.
    ///
    /// Calling this enables MP4 generation. Requires ffmpeg to be installed.
    /// If you want both GIF and MP4, call both [`output_gif`](Self::output_gif)
    /// and [`output_mp4`](Self::output_mp4).
    pub fn output_mp4(mut self, path: impl AsRef<Path>) -> Self {
        self.generate_mp4 = true;
        self.mp4_path = Some(path.as_ref().to_path_buf());
        self
    }

    /// Generate both GIF and MP4 outputs with the same base path.
    ///
    /// For example, `output_both("demo")` will generate `demo.gif` and `demo.mp4`.
    pub fn output_both(mut self, base_path: impl AsRef<Path>) -> Self {
        let base = base_path.as_ref();
        self.generate_gif = true;
        self.generate_mp4 = true;
        self.gif_path = Some(base.with_extension(DEFAULT_EXT));
        self.mp4_path = Some(base.with_extension(MOVIE_EXT));
        self
    }

    /// Set the decoration style.
    ///
    /// Available options:
    /// - [`Decor::None`] - No decoration
    /// - [`Decor::Shadow`] - Add a drop shadow effect (default)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use t_rec::types::Decor;
    ///
    /// let recorder = HeadlessRecorder::builder()
    ///     .window_id(12345)
    ///     .decor(Decor::Shadow)
    ///     .output_gif("demo.gif")
    ///     .build()?;
    /// ```
    pub fn decor(mut self, decor: Decor) -> Self {
        self.decor = decor;
        self
    }

    /// Set the background color for the shadow effect.
    ///
    /// Available options:
    /// - [`BackgroundColor::Transparent`] - Fully transparent (default)
    /// - [`BackgroundColor::White`] - White background
    /// - [`BackgroundColor::Black`] - Black background
    /// - [`BackgroundColor::custom()`] - Custom hex color (validates format)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use t_rec::types::BackgroundColor;
    ///
    /// // Predefined color
    /// let recorder = HeadlessRecorder::builder()
    ///     .window_id(12345)
    ///     .bg_color(BackgroundColor::White)
    ///     .output_gif("demo.gif")
    ///     .build()?;
    ///
    /// // Custom hex color (validation happens at custom() call)
    /// let recorder = HeadlessRecorder::builder()
    ///     .window_id(12345)
    ///     .bg_color(BackgroundColor::custom("#ff5500")?)
    ///     .output_gif("demo.gif")
    ///     .build()?;
    /// ```
    pub fn bg_color(mut self, color: BackgroundColor) -> Self {
        self.bg_color = color;
        self
    }

    /// Set the wallpaper background with padding.
    ///
    /// Available wallpaper options:
    /// - [`Wallpaper::Ventura`] - Built-in macOS Ventura wallpaper
    /// - [`Wallpaper::custom()`] - Custom wallpaper file (validates path exists)
    ///
    /// Padding is the amount of space (in pixels) between the frame and the wallpaper edge.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use t_rec::wallpapers::Wallpaper;
    ///
    /// // Built-in wallpaper
    /// let recorder = HeadlessRecorder::builder()
    ///     .window_id(12345)
    ///     .wallpaper(Wallpaper::Ventura, 60)
    ///     .output_gif("demo.gif")
    ///     .build()?;
    ///
    /// // Custom wallpaper (validation happens at custom() call)
    /// let recorder = HeadlessRecorder::builder()
    ///     .window_id(12345)
    ///     .wallpaper(Wallpaper::custom("/path/to/bg.png")?, 80)
    ///     .output_gif("demo.gif")
    ///     .build()?;
    /// ```
    pub fn wallpaper(mut self, wallpaper: Wallpaper, padding: u32) -> Self {
        self.wallpaper = Some(WallpaperConfig::new(wallpaper, padding));
        self
    }

    /// Clear any configured wallpaper.
    pub fn no_wallpaper(mut self) -> Self {
        self.wallpaper = None;
        self
    }

    /// Enable natural mode (save all frames without idle detection).
    ///
    /// By default, identical consecutive frames are skipped to reduce file size.
    /// Enable this to preserve exact timing.
    pub fn natural(mut self, natural: bool) -> Self {
        self.natural = natural;
        self
    }

    /// Set the idle pause threshold.
    ///
    /// When set, identical frames will be preserved up to this duration,
    /// then skipped. This helps preserve intentional pauses while still
    /// compressing long idle periods.
    pub fn idle_pause(mut self, duration: Duration) -> Self {
        self.idle_pause = Some(duration);
        self
    }

    /// Set a pause to add at the start of the GIF.
    pub fn start_pause(mut self, duration: Duration) -> Self {
        self.start_pause = Some(duration);
        self
    }

    /// Set a pause to add at the end of the GIF.
    pub fn end_pause(mut self, duration: Duration) -> Self {
        self.end_pause = Some(duration);
        self
    }

    /// Build the recorder.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - `window_id` is not set
    /// - No output path is set (either GIF or MP4)
    /// - The platform API fails to initialize
    /// - ImageMagick is not installed (for GIF generation)
    /// - ffmpeg is not installed (for MP4 generation)
    pub fn build(self) -> Result<HeadlessRecorder> {
        let window_id = self
            .window_id
            .context("window_id is required for HeadlessRecorder")?;

        if !self.generate_gif && !self.generate_mp4 {
            bail!("At least one output format must be specified (use output_gif or output_mp4)");
        }

        // Validate dependencies
        if self.generate_gif {
            check_for_gif().context("GIF generation requires ImageMagick's 'convert' command")?;
        }

        if self.generate_mp4 {
            check_for_mp4().context("MP4 generation requires ffmpeg with libx264 support")?;
        }

        let config = HeadlessRecorderConfig {
            window_id,
            fps: self.fps,
            natural: self.natural,
            idle_pause: self.idle_pause,
            start_pause: self.start_pause,
            end_pause: self.end_pause,
            decor: self.decor,
            bg_color: self.bg_color,
            wallpaper: self.wallpaper,
            generate_gif: self.generate_gif,
            generate_mp4: self.generate_mp4,
            gif_path: self.gif_path,
            mp4_path: self.mp4_path,
        };

        HeadlessRecorder::new(config)
    }
}

/// A headless recorder for capturing window content to GIF.
///
/// This recorder runs capture in a background thread and does not require
/// a PTY or shell. It's designed for programmatic recording of animations
/// and GUI applications.
///
/// # Example
///
/// ```ignore
/// use t_rec::HeadlessRecorder;
/// use core_animation::prelude::*;
///
/// // Create a window with animation
/// let window = WindowBuilder::new()
///     .title("Demo")
///     .size(400.0, 300.0)
///     .build();
///
/// // Set up the recorder
/// let recorder = HeadlessRecorder::builder()
///     .window_id(window.window_id())
///     .fps(30)
///     .output_gif("demo.gif")
///     .build()?;
///
/// // Start recording
/// recorder.start();
///
/// // Run the animation
/// window.show_for(5.seconds());
///
/// // Stop and generate the GIF
/// let output_path = recorder.stop_and_generate()?;
/// println!("Saved to: {}", output_path.display());
/// ```
pub struct HeadlessRecorder {
    config: HeadlessRecorderConfig,
    state: RecorderState,
}

/// Internal state for the recorder.
enum RecorderState {
    /// Recorder is ready to start.
    Ready { api: Box<dyn PlatformApi> },
    /// Recorder is actively capturing.
    Recording {
        router: EventRouter,
        capture_handle: JoinHandle<Result<()>>,
        tempdir: Arc<Mutex<TempDir>>,
        time_codes: Arc<Mutex<Vec<u128>>>,
    },
    /// Recorder has been consumed (stop_and_generate called).
    Consumed,
}

impl HeadlessRecorder {
    /// Create a new headless recorder with the given configuration.
    fn new(config: HeadlessRecorderConfig) -> Result<Self> {
        let api = Platform::setup()?;

        // Note: Calibration is deferred to start() so the window has time to become visible
        Ok(Self {
            config,
            state: RecorderState::Ready { api },
        })
    }

    /// Create a builder for configuring a new recorder.
    pub fn builder() -> HeadlessRecorderBuilder {
        HeadlessRecorderBuilder::new()
    }

    /// Start recording.
    ///
    /// This calibrates the capture API for the target window and spawns a
    /// background thread that captures frames at the configured FPS.
    /// Call [`stop_and_generate`](Self::stop_and_generate) to stop recording and
    /// generate the output GIF.
    ///
    /// **Important:** The window must be visible before calling this method.
    /// Call `window.show()` and optionally wait briefly before starting.
    ///
    /// # Panics
    ///
    /// Panics if called when the recorder is not in the Ready state
    /// (i.e., recording has already started or finished).
    ///
    /// # Errors
    ///
    /// Returns an error if calibration fails (e.g., window not visible).
    pub fn start(&mut self) -> Result<()> {
        // Take ownership of the current state
        let old_state = std::mem::replace(&mut self.state, RecorderState::Consumed);

        let mut api = match old_state {
            RecorderState::Ready { api } => api,
            RecorderState::Recording { .. } => {
                panic!("HeadlessRecorder::start() called while already recording")
            }
            RecorderState::Consumed => {
                panic!("HeadlessRecorder::start() called after stop_and_generate()")
            }
        };

        // Calibrate now that the window should be visible
        api.calibrate(self.config.window_id)
            .context("Failed to calibrate for window. Is the window visible?")?;

        let tempdir = Arc::new(Mutex::new(
            TempDir::new().expect("Failed to create temp directory"),
        ));
        let time_codes = Arc::new(Mutex::new(Vec::new()));
        let router = EventRouter::new();

        // Set up capture context
        let ctx = CaptureContext {
            win_id: self.config.window_id,
            time_codes: time_codes.clone(),
            tempdir: tempdir.clone(),
            natural: self.config.natural,
            idle_pause: self.config.idle_pause,
            fps: self.config.fps,
            screenshots: None,
        };

        // Subscribe before sending start event
        let event_rx = router.subscribe();

        // Spawn capture thread
        let capture_handle = thread::spawn(move || capture_thread(event_rx, api, ctx));

        // Send start event
        router.send(Event::Capture(CaptureEvent::Start));

        self.state = RecorderState::Recording {
            router,
            capture_handle,
            tempdir,
            time_codes,
        };

        Ok(())
    }

    /// Stop recording and generate the output files.
    ///
    /// This stops the capture thread, waits for it to finish, applies post-processing
    /// effects (shadow, corner radius, wallpaper), and then generates the output files
    /// (GIF and/or MP4).
    ///
    /// # Returns
    ///
    /// Returns a [`RecordingOutput`] containing paths to all generated files.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Recording was not started
    /// - The capture thread failed
    /// - No frames were captured
    /// - Wallpaper file doesn't exist or is invalid
    /// - GIF generation failed (e.g., ImageMagick not installed)
    /// - MP4 generation failed (e.g., ffmpeg not installed)
    pub fn stop_and_generate(mut self) -> Result<RecordingOutput> {
        // Take ownership of the current state
        let old_state = std::mem::replace(&mut self.state, RecorderState::Consumed);

        let (router, capture_handle, tempdir, time_codes) = match old_state {
            RecorderState::Recording {
                router,
                capture_handle,
                tempdir,
                time_codes,
            } => (router, capture_handle, tempdir, time_codes),
            RecorderState::Ready { .. } => {
                bail!("HeadlessRecorder::stop_and_generate() called before start()")
            }
            RecorderState::Consumed => {
                bail!("HeadlessRecorder::stop_and_generate() called twice")
            }
        };

        // Signal the capture thread to stop
        router.send(Event::Capture(CaptureEvent::Stop));

        // Wait for the capture thread to finish
        capture_handle
            .join()
            .map_err(|_| anyhow::anyhow!("Capture thread panicked"))?
            .context("Capture thread failed")?;

        // Get the captured data
        let time_codes_vec = time_codes.lock().unwrap().clone();
        let tempdir_guard = tempdir.lock().unwrap();

        let frame_count = time_codes_vec.len();
        if frame_count == 0 {
            bail!("No frames were captured");
        }

        // Collect frame file paths for post-processing
        let frame_files: Vec<PathBuf> = time_codes_vec
            .iter()
            .map(|tc| tempdir_guard.path().join(file_name_for(tc, IMG_EXT)))
            .filter(|p| p.exists())
            .collect();

        // Load wallpaper if configured
        let wallpaper: Option<DynamicImage> = if let Some(ref wp_config) = self.config.wallpaper {
            Some(self.load_wallpaper(&wp_config.wallpaper, &frame_files, wp_config.padding)?)
        } else {
            None
        };

        // Build post-processing options
        let post_opts = if let Some(ref wp) = wallpaper {
            let padding = self
                .config
                .wallpaper
                .as_ref()
                .map(|c| c.padding)
                .unwrap_or(0);
            PostProcessingOptions::new(self.config.decor, &self.config.bg_color)
                .with_wallpaper(wp, padding)
        } else {
            PostProcessingOptions::new(self.config.decor, &self.config.bg_color)
        };

        // Apply post-processing effects to all frames
        post_process_effects(&frame_files, &post_opts);

        // Generate outputs
        let mut gif_output_path: Option<PathBuf> = None;
        let mut mp4_output_path: Option<PathBuf> = None;

        // Generate GIF if enabled
        if self.config.generate_gif {
            if let Some(ref gif_path) = self.config.gif_path {
                let gif_path_str = gif_path.to_string_lossy().to_string();
                generate_gif(
                    &time_codes_vec,
                    &tempdir_guard,
                    &gif_path_str,
                    self.config.start_pause,
                    self.config.end_pause,
                )
                .context("Failed to generate GIF")?;
                gif_output_path = Some(gif_path.clone());
            }
        }

        // Generate MP4 if enabled
        if self.config.generate_mp4 {
            if let Some(ref mp4_path) = self.config.mp4_path {
                let mp4_path_str = mp4_path.to_string_lossy().to_string();
                generate_mp4(
                    &time_codes_vec,
                    &tempdir_guard,
                    &mp4_path_str,
                    self.config.fps,
                )
                .context("Failed to generate MP4")?;
                mp4_output_path = Some(mp4_path.clone());
            }
        }

        Ok(RecordingOutput {
            gif_path: gif_output_path,
            mp4_path: mp4_output_path,
            frame_count,
        })
    }

    /// Load and validate a wallpaper image.
    ///
    /// Handles both built-in wallpapers (like Ventura) and custom file paths.
    fn load_wallpaper(
        &self,
        wallpaper: &Wallpaper,
        frame_files: &[PathBuf],
        padding: u32,
    ) -> Result<DynamicImage> {
        // Get dimensions from the first frame to validate wallpaper size
        let (frame_width, frame_height) = if let Some(first_frame) = frame_files.first() {
            let img = image::open(first_frame)
                .context("Failed to open first frame to determine dimensions")?;
            img.dimensions()
        } else {
            bail!("No frames available to determine dimensions for wallpaper validation");
        };

        match wallpaper {
            Wallpaper::Ventura => {
                // Get the cached Ventura wallpaper
                let wp = get_ventura_wallpaper();
                let (wp_width, wp_height) = wp.dimensions();
                let min_width = frame_width + (padding * 2);
                let min_height = frame_height + (padding * 2);

                if wp_width < min_width || wp_height < min_height {
                    bail!(
                        "Frame size {}x{} with {}px padding exceeds built-in wallpaper size {}x{}.\n\
                         Try reducing the frame size or padding.",
                        frame_width,
                        frame_height,
                        padding,
                        wp_width,
                        wp_height
                    );
                }
                Ok(wp.clone())
            }
            Wallpaper::Custom(validated_path) => {
                // Handle custom wallpaper file (path already validated at construction)
                load_and_validate_wallpaper(
                    validated_path.as_path(),
                    frame_width,
                    frame_height,
                    padding,
                )
            }
        }
    }

    /// Check if recording is currently active.
    pub fn is_recording(&self) -> bool {
        matches!(self.state, RecorderState::Recording { .. })
    }

    /// Get the configured window ID.
    pub fn window_id(&self) -> WindowId {
        self.config.window_id
    }

    /// Get the configured FPS.
    pub fn fps(&self) -> u8 {
        self.config.fps
    }

    /// Get the configured GIF output path (if any).
    pub fn gif_path(&self) -> Option<&Path> {
        self.config.gif_path.as_deref()
    }

    /// Get the configured MP4 output path (if any).
    pub fn mp4_path(&self) -> Option<&Path> {
        self.config.mp4_path.as_deref()
    }

    /// Get the configured decoration style.
    pub fn decor(&self) -> Decor {
        self.config.decor
    }

    /// Get the configured background color.
    pub fn bg_color(&self) -> &BackgroundColor {
        &self.config.bg_color
    }

    /// Get the configured wallpaper (if any).
    pub fn wallpaper_config(&self) -> Option<&WallpaperConfig> {
        self.config.wallpaper.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_requires_window_id() {
        let result = HeadlessRecorderBuilder::new()
            .fps(30)
            .output_gif("test.gif")
            .build();

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("window_id is required"));
    }

    #[test]
    fn test_builder_requires_output_format() {
        let result = HeadlessRecorderBuilder::new()
            .window_id(12345)
            .fps(30)
            .build();

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err
            .to_string()
            .contains("At least one output format must be specified"));
    }

    #[test]
    fn test_builder_fps_clamping() {
        let builder = HeadlessRecorderBuilder::new().fps(100);
        assert_eq!(builder.fps, 60);

        let builder = HeadlessRecorderBuilder::new().fps(0);
        assert_eq!(builder.fps, 1);
    }

    #[test]
    fn test_builder_default_fps() {
        let builder = HeadlessRecorderBuilder::new();
        assert_eq!(builder.fps, 15);
    }

    #[test]
    fn test_builder_default_decor() {
        let builder = HeadlessRecorderBuilder::new();
        assert_eq!(builder.decor, Decor::Shadow);
    }

    #[test]
    fn test_builder_default_bg_color() {
        let builder = HeadlessRecorderBuilder::new();
        assert_eq!(builder.bg_color, BackgroundColor::Transparent);
    }

    #[test]
    fn test_builder_decor() {
        let builder = HeadlessRecorderBuilder::new().decor(Decor::None);
        assert_eq!(builder.decor, Decor::None);

        let builder = HeadlessRecorderBuilder::new().decor(Decor::Shadow);
        assert_eq!(builder.decor, Decor::Shadow);
    }

    #[test]
    fn test_builder_bg_color() {
        let builder = HeadlessRecorderBuilder::new().bg_color(BackgroundColor::White);
        assert_eq!(builder.bg_color, BackgroundColor::White);

        let builder =
            HeadlessRecorderBuilder::new().bg_color(BackgroundColor::custom("#ff5500").unwrap());
        assert_eq!(builder.bg_color.as_str(), "#ff5500");
    }

    #[test]
    fn test_builder_wallpaper_ventura() {
        let builder = HeadlessRecorderBuilder::new().wallpaper(Wallpaper::Ventura, 50);
        assert!(builder.wallpaper.is_some());
        let wp_config = builder.wallpaper.unwrap();
        assert_eq!(wp_config.wallpaper, Wallpaper::Ventura);
        assert_eq!(wp_config.padding, 50);
    }

    #[test]
    fn test_builder_no_wallpaper() {
        let builder = HeadlessRecorderBuilder::new()
            .wallpaper(Wallpaper::Ventura, 50)
            .no_wallpaper();
        assert!(builder.wallpaper.is_none());
    }

    #[test]
    fn test_builder_output_gif() {
        let builder = HeadlessRecorderBuilder::new().output_gif("test.gif");
        assert!(builder.generate_gif);
        assert!(!builder.generate_mp4);
        assert_eq!(builder.gif_path, Some(PathBuf::from("test.gif")));
    }

    #[test]
    fn test_builder_output_mp4() {
        let builder = HeadlessRecorderBuilder::new().output_mp4("test.mp4");
        assert!(!builder.generate_gif);
        assert!(builder.generate_mp4);
        assert_eq!(builder.mp4_path, Some(PathBuf::from("test.mp4")));
    }

    #[test]
    fn test_builder_output_both() {
        let builder = HeadlessRecorderBuilder::new().output_both("demo");
        assert!(builder.generate_gif);
        assert!(builder.generate_mp4);
        assert_eq!(builder.gif_path, Some(PathBuf::from("demo.gif")));
        assert_eq!(builder.mp4_path, Some(PathBuf::from("demo.mp4")));
    }

    #[test]
    fn test_recording_output_debug() {
        let output = RecordingOutput {
            gif_path: Some(PathBuf::from("test.gif")),
            mp4_path: Some(PathBuf::from("test.mp4")),
            frame_count: 100,
        };
        let debug_str = format!("{:?}", output);
        assert!(debug_str.contains("test.gif"));
        assert!(debug_str.contains("test.mp4"));
        assert!(debug_str.contains("100"));
    }

    #[test]
    fn test_with_defaults_for_demo() {
        let builder = HeadlessRecorderBuilder::new().with_defaults_for_demo();
        assert_eq!(builder.fps, 15);
        assert_eq!(builder.decor, Decor::None);
        assert!(builder.wallpaper.is_some());
        let wp_config = builder.wallpaper.unwrap();
        assert_eq!(wp_config.wallpaper, Wallpaper::Ventura);
        assert_eq!(wp_config.padding, 60);
    }
}
