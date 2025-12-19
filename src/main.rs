mod assets;
mod cli;
mod common;
mod config;
mod decors;
mod generators;
mod prompt;
mod summary;
mod tips;
mod wallpapers;

mod capture;
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
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
use crate::common::utils::{clear_screen, parse_delay, HumanReadable};
use crate::common::{Margin, PlatformApi};
use crate::config::{expand_home, handle_init_config, handle_list_profiles};
use crate::decors::{apply_big_sur_corner_effect, apply_shadow_effect};
use crate::generators::{check_for_gif, check_for_mp4, generate_gif, generate_mp4};
use crate::summary::print_recording_summary;
use crate::tips::show_tip;
use crate::wallpapers::{
    apply_wallpaper_effect, get_ventura_wallpaper, is_builtin_wallpaper,
    load_and_validate_wallpaper,
};

use crate::capture::{capture_thread, CaptureContext};
use crate::prompt::{start_background_prompt, PromptResult};
use crate::utils::{sub_shell_thread, target_file, DEFAULT_EXT, MOVIE_EXT};
use anyhow::{bail, Context};
use image::FlatSamples;
use image::{DynamicImage, GenericImageView};
use std::borrow::Borrow;
use std::io::{self, Write};
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
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

fn main() -> Result<()> {
    env_logger::init();

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

    let (program, program_args): (String, Vec<String>) = args
        .program
        .split_first()
        .map(|(prog, rest)| (prog.clone(), rest.to_vec()))
        .unwrap_or_else(|| (env::var("SHELL").unwrap_or_else(|_| DEFAULT_SHELL.to_owned()), vec![]));
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
    let (tx, rx) = mpsc::channel();
    let photograph = {
        let ctx = CaptureContext {
            win_id,
            time_codes: time_codes.clone(),
            tempdir: tempdir.clone(),
            natural: settings.natural(),
            idle_pause,
            fps,
        };
        thread::spawn(move || -> Result<()> { capture_thread(&rx, api, ctx) })
    };
    let interact =
        thread::spawn(move || -> Result<()> { sub_shell_thread(&program, &program_args).map(|_| ()) });

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
    }
    thread::sleep(Duration::from_millis(1250));
    clear_screen();

    interact
        .join()
        .unwrap()
        .context("Cannot launch the sub shell")?;
    tx.send(()).context("Cannot stop the recording thread")?;
    photograph
        .join()
        .unwrap()
        .context("Cannot launch the recording thread")?;

    let frame_count = time_codes.lock().unwrap().borrow().len();
    print_recording_summary(&settings, frame_count);

    println!();
    println!("🎆 Applying effects (might take a bit)");
    show_tip();

    apply_big_sur_corner_effect(
        &time_codes.lock().unwrap(),
        tempdir.lock().unwrap().borrow(),
    );

    if settings.decor() == "shadow" {
        apply_shadow_effect(
            &time_codes.lock().unwrap(),
            tempdir.lock().unwrap().borrow(),
            settings.bg().to_string(),
        );
    }

    if let Some((wallpaper, padding)) = wallpaper_config {
        apply_wallpaper_effect(
            &time_codes.lock().unwrap(),
            tempdir.lock().unwrap().borrow(),
            &wallpaper,
            padding,
        );
    }

    let target = target_file(settings.output());
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
