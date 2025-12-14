mod assets;
mod cli;
mod common;
mod config;
mod decors;
mod generators;
mod input;
mod post_processing;
mod prompt;
mod screenshot;
mod summary;
mod tips;
mod wallpapers;

mod capture;
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(unix)]
mod pty;
mod utils;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(any(target_os = "linux", target_os = "netbsd"))]
use crate::linux::*;
#[cfg(target_os = "macos")]
use crate::macos::*;
#[cfg(target_os = "windows")]
use crate::windows::*;

use crate::cli::{launch, resolve_profiled_settings, CliArgs};
use crate::common::utils::{clear_screen, parse_delay, print_tree_list, HumanReadable};
use crate::common::{Margin, PlatformApi};
use crate::config::{expand_home, handle_init_config, handle_list_profiles};
use crate::generators::{check_for_gif, check_for_mp4, generate_gif, generate_mp4};
use crate::post_processing::{
    post_process_effects, post_process_file, post_process_screenshots, PostProcessingOptions,
};
use crate::summary::print_recording_summary;
use crate::tips::show_tip;
use crate::wallpapers::{get_ventura_wallpaper, is_builtin_wallpaper, load_and_validate_wallpaper};

use crate::capture::{capture_thread, CaptureContext, CaptureEvent, EventRouter, FlashEvent};
use crate::input::{HotkeyConfig, InputState, KeyboardMonitor};
use crate::prompt::{start_background_prompt, PromptResult};
#[cfg(unix)]
use crate::pty::PtyShell;
use crate::screenshot::ScreenshotInfo;
use crate::utils::{target_file, DEFAULT_EXT, MOVIE_EXT};
use anyhow::{bail, Context};
use image::FlatSamples;
use image::{DynamicImage, GenericImageView};
use std::borrow::Borrow;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant, SystemTime};
use std::{env, thread};
use tempfile::TempDir;

pub type Image = FlatSamples<Vec<u8>>;
pub type ImageOnHeap = Box<Image>;
pub type WindowId = u64;
pub type WindowList = Vec<WindowListEntry>;
pub type WindowListEntry = (Option<String>, WindowId);
pub type Result<T> = anyhow::Result<T>;

macro_rules! prof {
    ($($something:expr;)+) => {
        {
            let start = Instant::now();
            $(
                $something;
            )*
            start.elapsed()
        }
    };
}

/// Initialize logging to a file (./t-rec-recording.log).
///
/// Default level is INFO, can be overridden with RUST_LOG env var.
/// Logging to a file avoids interfering with the terminal output.
fn init_logging() {
    use env_logger::{Builder, Target};
    use std::io::Write as _;

    let log_file = File::create("t-rec-recording.log").ok();

    let mut builder = Builder::new();

    // Set default filter to INFO, allow RUST_LOG to override
    builder.filter_level(log::LevelFilter::Info);
    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse_filters(&rust_log);
    }

    // Format with timestamp and level
    builder.format(|buf, record| {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let secs = now.as_secs();
        let millis = now.subsec_millis();
        writeln!(
            buf,
            "[{}.{:03} {} {}] {}",
            secs,
            millis,
            record.level(),
            record.target(),
            record.args()
        )
    });

    // Write to file if available, otherwise stderr
    if let Some(file) = log_file {
        builder.target(Target::Pipe(Box::new(file)));
    }

    builder.init();
}

fn main() -> Result<()> {
    // Initialize logging to file (./t-rec-recording.log)
    init_logging();

    let args = launch();

    // Handle config-related commands first
    if args.init_config {
        return handle_init_config();
    }
    if args.list_profiles {
        return handle_list_profiles();
    }
    if args.list_windows {
        return ls_win();
    }

    let settings = resolve_profiled_settings(&args)?;

    let program: String = {
        if let Some(prog) = &args.program {
            prog.to_string()
        } else {
            let default = DEFAULT_SHELL.to_owned();
            env::var("SHELL").unwrap_or(default)
        }
    };
    let (win_id, window_name) = current_win_id(&args)?;
    let mut api = setup()?;
    api.calibrate(win_id)?;

    // Validate wallpaper BEFORE recording starts
    let wallpaper_config = validate_wallpaper_config(&settings, &api, win_id)?;

    let should_generate_gif = !settings.video_only();
    let should_generate_video = settings.video() || settings.video_only();
    let (start_delay, end_delay, idle_pause) = (
        parse_delay(settings.start_pause.as_deref(), "start-pause")?,
        parse_delay(settings.end_pause.as_deref(), "end-pause")?,
        parse_delay(Some(settings.idle_pause()), "idle-pause")?,
    );
    let fps = settings.fps();

    if should_generate_gif {
        check_for_gif()?;
    }
    if should_generate_video {
        check_for_mp4()?;
    }

    // the nice thing is the cleanup on drop
    let tempdir = Arc::new(Mutex::new(
        TempDir::new().context("Cannot create tempdir.")?,
    ));
    let time_codes = Arc::new(Mutex::new(Vec::new()));
    let screenshots = Arc::new(Mutex::new(Vec::<ScreenshotInfo>::new()));

    // Create event channels
    let (capture_tx, capture_rx) = mpsc::channel::<CaptureEvent>();
    let (flash_tx, flash_rx) = mpsc::channel::<FlashEvent>();

    // Create event router for keyboard monitor
    let router = EventRouter::new(Some(capture_tx.clone()), Some(flash_tx));

    // Create input state for screenshot flag
    let input_state = Arc::new(InputState::new());

    // Shared idle duration tracking (for accurate timecodes)
    let idle_duration = Arc::new(Mutex::new(Duration::from_millis(0)));
    let recording_start = Instant::now();

    // Create capture context with screenshot support
    let photograph = {
        let ctx = CaptureContext {
            win_id,
            time_codes: time_codes.clone(),
            tempdir: tempdir.clone(),
            natural: settings.natural(),
            idle_pause,
            fps,
            screenshots: Some(screenshots.clone()),
        };
        thread::spawn(move || -> Result<()> { capture_thread(capture_rx, api, ctx) })
    };

    clear_screen();
    io::stdout().flush().unwrap();
    if settings.verbose() {
        println!(
            "Frame cache dir: {:?}",
            tempdir.lock().expect("Cannot lock tempdir resource").path()
        );
        if let Some(window) = window_name {
            println!("Recording window: {:?}", window);
        } else {
            println!("Recording window id: {}", win_id);
        }
    }
    if !settings.quiet() {
        println!("[t-rec]: Press Ctrl+D to end recording");
        println!("[t-rec]: F2 = Screenshot");
        // println!("[t-rec]: F2 = Screenshot, F3 = Toggle keystroke capture"); // future feature

        // little countdown before starting
        for i in (1..=3).rev() {
            // Move cursor up, clear line, print countdown
            print!("\r[t-rec]: Recording starts in {}...", i);
            io::stdout().flush().ok();
            thread::sleep(Duration::from_secs(1));
        }
        // hack to overwrite the line
        print!("\r[t-rec]: Recording!                         \n");
        io::stdout().flush().ok();
        thread::sleep(Duration::from_millis(250));
    }
    // Clear screen before spawning shell
    print!("\x1b[2J\x1b[H");
    io::stdout().flush().ok();

    // Send start event to capture thread
    capture_tx
        .send(CaptureEvent::Start)
        .context("Cannot start capture thread")?;

    // Spawn shell with PTY for proper terminal interaction
    #[cfg(unix)]
    let shell_result = {
        let mut pty_shell = PtyShell::spawn(&program)?;
        let shell_stdin = pty_shell.get_writer()?;
        let should_exit = Arc::new(AtomicBool::new(false));
        let should_exit_for_output = should_exit.clone();
        let should_exit_for_monitor = should_exit.clone();

        // Forward PTY output to stdout in background
        let output_thread = thread::spawn(move || pty_shell.forward_output(should_exit_for_output));

        // Run keyboard monitor (blocks until Ctrl+D or shell exit)
        let hotkey_config = HotkeyConfig::default();
        let keyboard_monitor = KeyboardMonitor::new(
            input_state,
            idle_duration,
            recording_start,
            hotkey_config,
            router,
        );

        // Handle flash events in background (for visual feedback)
        let flash_handler = thread::spawn(move || {
            while let Ok(event) = flash_rx.recv() {
                match event {
                    FlashEvent::ScreenshotTaken => {
                        log::info!("Screenshot taken");
                    }
                    FlashEvent::KeyPressed { .. } => {
                        unimplemented!("Capturing keys and flashing them will be coming soon!")
                    }
                }
            }
        });

        // Run keyboard monitor - this blocks until exit
        if let Err(e) = keyboard_monitor.run(shell_stdin, should_exit_for_monitor) {
            log::error!("Keyboard monitor error: {}", e);
        }

        // Signal output thread to stop
        should_exit.store(true, Ordering::Release);

        // Wait for threads to finish
        let _ = output_thread.join();
        drop(flash_handler);
        Ok::<(), anyhow::Error>(())
    };

    #[cfg(not(unix))]
    let shell_result = {
        // On non-Unix platforms, fall back to simple shell spawn
        use std::process::Command;
        let mut shell = Command::new(&program)
            .spawn()
            .context(format!("failed to start {:?}", &program))?;
        let _ = shell.wait();
        Ok::<(), anyhow::Error>(())
    };

    shell_result?;

    // Stop capture thread
    let _ = capture_tx.send(CaptureEvent::Stop);

    // Wait for capture thread to finish
    photograph
        .join()
        .unwrap()
        .context("Cannot finish recording thread")?;

    let frame_count = time_codes.lock().unwrap().borrow().len();
    print_recording_summary(&settings, frame_count);

    println!();
    println!("🎆 Applying effects (might take a bit)");
    show_tip();

    // Build post-processing options
    let post_opts = if let Some((ref wallpaper, padding)) = wallpaper_config {
        PostProcessingOptions::new(settings.decor(), settings.bg())
            .with_wallpaper(wallpaper, padding)
    } else {
        PostProcessingOptions::new(settings.decor(), settings.bg())
    };

    // Collect frame file paths and apply effects
    {
        let temp_path = tempdir.lock().unwrap().path().to_path_buf();
        let codes = time_codes.lock().unwrap();
        let frame_files: Vec<_> = codes
            .iter()
            .map(|tc| temp_path.join(crate::utils::file_name_for(tc, crate::utils::IMG_EXT)))
            .collect();
        post_process_effects(&frame_files, &post_opts);
    }

    let target = target_file(settings.output());

    {
        let screenshots_list = screenshots.lock().unwrap();
        if !screenshots_list.is_empty() {
            println!();
            println!("📸 Processing {} screenshot(s)...", screenshots_list.len());
            let saved_screenshots =
                post_process_screenshots(&screenshots_list, &target, &post_opts);

            // Print saved screenshots with tree-style formatting
            if !saved_screenshots.is_empty() {
                println!("Screenshots saved:");
                print_tree_list(&saved_screenshots);
            }
        }
    }
    println!();

    let mut time = Duration::default();

    // Start video prompt in background if we might need to ask
    // This runs while GIF is being generated, so user can answer early
    let video_prompt = if !should_generate_video && !settings.quiet() {
        start_background_prompt("🎬 Also generate MP4 video?", 15)
    } else {
        None
    };

    if should_generate_gif {
        time += prof! {
            generate_gif(
                &time_codes.lock().unwrap(),
                tempdir.lock().unwrap().borrow(),
                &format!("{}.{}", target, DEFAULT_EXT),
                start_delay,
                end_delay
            )?;
        };
    }

    // Determine if we should generate video:
    // - If already requested via CLI/config, generate it
    // - Otherwise, check the background prompt result
    let should_generate_video = if should_generate_video {
        true
    } else if let Some(prompt) = video_prompt {
        match prompt.wait() {
            PromptResult::Yes => {
                check_for_mp4()?;
                true
            }
            PromptResult::No | PromptResult::Timeout => false,
        }
    } else {
        false
    };

    if should_generate_video {
        time += prof! {
            generate_mp4(
                &time_codes.lock().unwrap(),
                tempdir.lock().unwrap().borrow(),
                &format!("{}.{}", target, MOVIE_EXT),
                fps,
            )?;
        }
    }

    println!("Time: {}", time.as_human_readable());

    Ok(())
}

/// Validates and loads the wallpaper configuration before recording starts.
///
/// Returns `Some((wallpaper, padding))` if wallpaper is configured, `None` otherwise.
/// Fails early with a clear error message if the wallpaper is invalid or too small.
fn validate_wallpaper_config(
    settings: &config::ProfileSettings,
    api: &impl PlatformApi,
    win_id: WindowId,
) -> Result<Option<(DynamicImage, u32)>> {
    let wp_value = match &settings.wallpaper {
        Some(v) => v,
        None => return Ok(None),
    };

    // Expand $HOME in wallpaper path
    let wp_value = expand_home(wp_value);
    let padding = settings.wallpaper_padding();

    // Capture a screenshot to get terminal dimensions
    let screenshot = api.capture_window_screenshot(win_id)?;
    let terminal_width = screenshot.layout.width;
    let terminal_height = screenshot.layout.height;

    let wallpaper = if is_builtin_wallpaper(&wp_value) {
        match wp_value.to_lowercase().as_str() {
            "ventura" => {
                // Validate built-in wallpaper dimensions too
                let wp = get_ventura_wallpaper();
                let (wp_width, wp_height) = wp.dimensions();
                let min_width = terminal_width + (padding * 2);
                let min_height = terminal_height + (padding * 2);

                if wp_width < min_width || wp_height < min_height {
                    bail!(
                        "Terminal size {}x{} with {}px padding exceeds built-in wallpaper size {}x{}.\n\
                         Try reducing the terminal size or padding.",
                        terminal_width,
                        terminal_height,
                        padding,
                        wp_width,
                        wp_height
                    );
                }
                wp.clone()
            }
            _ => bail!("Unknown built-in wallpaper: {}", wp_value),
        }
    } else {
        // Custom wallpaper path - validate before recording
        let path = Path::new(&wp_value);
        load_and_validate_wallpaper(path, terminal_width, terminal_height, padding)?
    };

    Ok(Some((wallpaper, padding)))
}

///
/// determines the WindowId either by env var 'WINDOWID'
/// or by the env var 'TERM_PROGRAM' and then asking the window manager for all visible windows
/// and finding the Terminal in that list
/// panics if WindowId was not was not there
fn current_win_id(args: &CliArgs) -> Result<(WindowId, Option<String>)> {
    match args.win_id.ok_or_else(|| env::var("WINDOWID")) {
        Ok(win_id) => Ok((win_id, None)),
        Err(_) => {
            let terminal = env::var("TERM_PROGRAM").context(
                "Env variable 'TERM_PROGRAM' was empty but is needed for figure out the WindowId. Please set it to e.g. TERM_PROGRAM=alacitty",
            );
            if let Ok(terminal) = terminal {
                let (win_id, name) = get_window_id_for(terminal).context(
                    "Cannot determine the WindowId of this terminal. Please set env variable 'WINDOWID' and try again.",
                )?;
                Ok((win_id, Some(name)))
            } else {
                let api = setup()?;
                let win_id = api.get_active_window()?;
                Ok((win_id, None))
            }
        }
    }
}

/// finds the window id for a given terminal / program by name
pub fn get_window_id_for(terminal: String) -> Result<(WindowId, String)> {
    let api = setup()?;
    for term in terminal.to_lowercase().split('.') {
        for (window_owner, window_id) in api.window_list()? {
            if let Some(window_owner) = window_owner {
                let window = &window_owner.to_lowercase();
                let terminal = &terminal.to_lowercase();
                if window.contains(term) || terminal.contains(window) {
                    return Ok((window_id, terminal.to_owned()));
                }
            }
        }
    }

    bail!("Cannot determine the window id from the available window list.")
}

///
/// lists all windows with name and id
pub fn ls_win() -> Result<()> {
    let api = setup()?;
    let mut list = api.window_list()?;
    list.sort();

    println!("Window | Id");
    for (window_owner, window_id) in list.iter() {
        if let (Some(window_owner), window_id) = (window_owner, window_id) {
            println!("{} | {}", window_owner, window_id)
        }
    }

    Ok(())
}
