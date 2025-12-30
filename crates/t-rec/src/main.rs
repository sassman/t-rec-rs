//! t-rec - Terminal Recorder
//!
//! A blazingly fast terminal recorder that generates GIFs and MP4s.

mod assets;
mod cli;
mod common;
mod config;
mod decors;
mod event_router;
mod generators;
mod input;
mod logging;
mod output;
mod post_processing;
mod prompt;
mod recorder;
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

use crate::generators::{check_for_gif, check_for_mp4};
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
use crate::linux::*;
use crate::logging::init_logging;
#[cfg(target_os = "macos")]
use crate::macos::*;
use crate::recorder::runtime::Runtime;
#[cfg(target_os = "windows")]
use crate::windows::*;

use crate::cli::{launch, resolve_profiled_settings, CliArgs};
use crate::common::utils::parse_delay;
use crate::common::{Platform, PlatformApi, PlatformApiFactory};
use crate::config::{expand_home, handle_init_config, handle_list_profiles, ProfileSettings};
use crate::output::OutputGenerator;
use crate::recorder::{RecordingSession, SessionConfig};
use crate::summary::print_recording_summary;
use crate::wallpapers::{get_ventura_wallpaper, is_builtin_wallpaper, load_and_validate_wallpaper};

use anyhow::{bail, Context};
use image::{DynamicImage, FlatSamples, GenericImageView};
use std::env;
use std::path::Path;
use std::time::Duration;

// Re-export common types
pub type Image = FlatSamples<Vec<u8>>;
pub type ImageOnHeap = Box<Image>;
pub type WindowId = u64;
pub type WindowList = Vec<WindowListEntry>;
pub type WindowListEntry = (Option<String>, WindowId);
pub type Result<T> = anyhow::Result<T>;

// Re-export Margin for other modules
pub use crate::common::Margin;

fn main() -> Result<()> {
    init_logging();

    let args = launch();

    // Handle config-related commands first
    if args.init_config {
        return handle_init_config();
    }
    if args.list_profiles {
        return handle_list_profiles();
    }

    let mut api = Platform::setup()?;
    if args.list_windows {
        return ls_win(&api);
    }

    // TODO: 1. this whole block could be hidden inside of `SessionConfig::from_args()
    // Resolve settings from CLI args and config file
    let settings = resolve_profiled_settings(&args)?;

    validate_prerequisites(&settings)?;

    // Determine shell program
    let program = args
        .program
        .clone()
        .unwrap_or_else(|| env::var("SHELL").unwrap_or_else(|_| DEFAULT_SHELL.to_owned()));

    // Get window ID and optional name
    let (win_id, window_name) = current_win_id(&api, &args)?;
    api.calibrate(win_id)?;

    // TODO(release): this should be removed eventually
    #[cfg(target_os = "macos")]
    if args.test_flash {
        return run_test_flash(win_id);
    }

    // Validate wallpaper BEFORE recording starts
    let wallpaper_config = validate_wallpaper_config(&settings, &api, win_id)?;

    // Parse delay settings
    let (start_delay, end_delay, idle_pause) = (
        parse_delay(settings.start_pause.as_deref(), "start-pause")?,
        parse_delay(settings.end_pause.as_deref(), "end-pause")?,
        parse_delay(Some(settings.idle_pause()), "idle-pause")?,
    );

    // Build session configuration
    let session_config = SessionConfig::builder()
        .win_id(win_id)
        .window_name(window_name)
        .program(program)
        .using_profile(&settings)
        .idle_pause(idle_pause)
        .start_delay(start_delay.unwrap_or(Duration::ZERO))
        .end_delay(end_delay.unwrap_or(Duration::ZERO))
        .wallpaper(wallpaper_config.clone())
        .build();
    // TODO: 1. ending (start see above)

    // Run recording session
    let session = RecordingSession::new(session_config, Box::new(api), Runtime::new())?;
    let output_config = session.output_config();
    let result = session.run()?;

    // Print recording summary
    print_recording_summary(&settings, result.frame_count);

    // Generate outputs (GIF, MP4, screenshots)
    OutputGenerator::new(result, output_config).process()?;

    Ok(())
}

/// Validate required tools
fn validate_prerequisites(settings: &ProfileSettings) -> Result<()> {
    if !settings.video_only() {
        check_for_gif()?;
    }
    if settings.video() || settings.video_only() {
        check_for_mp4()?;
    }

    Ok(())
}

/// Test flash indicator (macOS only).
#[cfg(target_os = "macos")]
fn run_test_flash(win_id: WindowId) -> Result<()> {
    use osd_flash::prelude::*;
    use std::thread;

    fn show_camera_flash(win_id: u64) -> osd_flash::Result<()> {
        OsdFlashBuilder::new()
            .dimensions(120.0)
            .position(FlashPosition::TopRight)
            .margin(20.0)
            .level(WindowLevel::AboveAll)
            .attach_to_window(win_id)
            .build()?
            .draw(CameraIcon::new(120.0).build())
            .show_for_seconds(1.5)
    }

    println!("Testing screen flash indicator within a background thread...");
    println!("Showing flash indicator from main thread...");
    if let Err(e) = show_camera_flash(win_id) {
        log::warn!("Failed to show flash: {}", e);
    }
    thread::sleep(Duration::from_secs(5));

    thread::spawn(move || {
        println!("Showing flash indicator from background thread...");
        if let Err(e) = show_camera_flash(win_id) {
            log::warn!("Failed to show flash: {}", e);
        }
        thread::sleep(Duration::from_secs(5));
    })
    .join()
    .unwrap();

    println!("Test complete!");
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

/// Determines the WindowId either by env var 'WINDOWID'
/// or by the env var 'TERM_PROGRAM' and then asking the window manager for all visible windows
/// and finding the Terminal in that list.
fn current_win_id(api: &impl PlatformApi, args: &CliArgs) -> Result<(WindowId, Option<String>)> {
    match args.win_id.ok_or_else(|| env::var("WINDOWID")) {
        Ok(win_id) => Ok((win_id, None)),
        Err(_) => {
            let terminal = env::var("TERM_PROGRAM").context(
                "Env variable 'TERM_PROGRAM' was empty but is needed for figure out the WindowId. Please set it to e.g. TERM_PROGRAM=alacitty",
            );
            if let Ok(terminal) = terminal {
                let (win_id, name) = get_window_id_for(api, terminal).context(
                    "Cannot determine the WindowId of this terminal. Please set env variable 'WINDOWID' and try again.",
                )?;
                Ok((win_id, Some(name)))
            } else {
                let win_id = api.get_active_window()?;
                Ok((win_id, None))
            }
        }
    }
}

/// Finds the window id for a given terminal / program by name.
pub fn get_window_id_for(api: &impl PlatformApi, terminal: String) -> Result<(WindowId, String)> {
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

/// Lists all windows with name and id.
pub fn ls_win(api: &impl PlatformApi) -> Result<()> {
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
