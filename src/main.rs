mod cli;
mod common;
mod decor_effect;
mod gif_gen;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "linux")]
use crate::linux::*;
#[cfg(target_os = "macos")]
use crate::macos::*;
#[cfg(target_os = "windows")]
use crate::win::*;

use crate::cli::launch;
use crate::common::PlatformApi;
use crate::decor_effect::{apply_big_sur_corner_effect, apply_shadow_effect};
use crate::gif_gen::*;
use anyhow::{bail, Context};
use image::{save_buffer, FlatSamples};
use std::borrow::Borrow;
use std::ffi::OsStr;
use std::ops::{Add, Sub};
use std::process::{Command, ExitStatus, Output};
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, thread};
use tempfile::TempDir;

pub type ImageOnHeap = Box<FlatSamples<Vec<u8>>>;
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
    let args = launch();
    if args.is_present("list-windows") {
        return ls_win();
    }

    let program: String = {
        if args.is_present("program") {
            args.value_of("program").unwrap().to_owned()
        } else {
            let default = "/bin/sh".to_owned();
            env::var("SHELL").unwrap_or(default)
        }
    };
    let (win_id, window_name) =
        current_win_id().context("Cannot retrieve the window id of the to be recorded window.")?;
    let mut api = setup()?;
    api.calibrate(win_id)?;

    let force_natural = args.is_present("natural-mode");

    check_for_imagemagick()?;

    // the nice thing is the cleanup on drop
    let tempdir = Arc::new(Mutex::new(
        TempDir::new().context("Cannot create tempdir.")?,
    ));
    let time_codes = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = mpsc::channel();
    let photograph = {
        let tempdir = tempdir.clone();
        let time_codes = time_codes.clone();
        let force_natural = force_natural;
        thread::spawn(move || -> Result<()> {
            capture_thread(&rx, api, win_id, time_codes, tempdir, force_natural)
        })
    };
    let interact = thread::spawn(move || -> Result<()> { sub_shell_thread(&program).map(|_| ()) });

    clear_screen();
    if args.is_present("verbose") {
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
    println!("Press Ctrl+D to end recording");
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

    println!(
        "\n🎆 Applying effects to {} frames (might take a bit)",
        time_codes.lock().unwrap().borrow().len()
    );
    apply_big_sur_corner_effect(
        &time_codes.lock().unwrap(),
        tempdir.lock().unwrap().borrow(),
    )?;

    if let Some("shadow") = args.value_of("decor") {
        apply_shadow_effect(
            &time_codes.lock().unwrap(),
            tempdir.lock().unwrap().borrow(),
            args.value_of("bg").unwrap().to_string(),
        )?
    }

    let time = prof! {

    generate_gif_with_convert(
        &time_codes.lock().unwrap(),
        tempdir.lock().unwrap().borrow(),
    )
    .map(|_| ())?;
    };
    println!("Time: {}ms", time.as_millis());

    Ok(())
}

/// TODO: that works strange on ubuntu
/// escape sequences that clears the screen
fn clear_screen() {
    print!("{esc}[2J", esc = 27 as char);
    print!("{esc}[H", esc = 27 as char);
}

///
/// captures screenshots as file on disk
/// collects also the timecodes when they have been captured
/// stops once receiving something in rx
fn capture_thread(
    rx: &Receiver<()>,
    api: Box<dyn PlatformApi>,
    win_id: WindowId,
    time_codes: Arc<Mutex<Vec<u128>>>,
    tempdir: Arc<Mutex<TempDir>>,
    force_natural: bool,
) -> Result<()> {
    let duration = Duration::from_millis(250);
    let start = Instant::now();
    let mut idle_duration = Duration::from_millis(0);
    let mut last_frame: Option<ImageOnHeap> = None;
    let mut identical_frames = 0;
    let mut last_now = Instant::now();
    loop {
        // blocks for a timeout
        if rx.recv_timeout(duration).is_ok() {
            break;
        }
        let now = Instant::now();
        let effective_now = now.sub(idle_duration);
        let tc = effective_now.saturating_duration_since(start).as_millis();
        let image = api.capture_window_screenshot(win_id)?;
        if !force_natural {
            if last_frame.is_some()
                && image
                    .samples
                    .as_slice()
                    .eq(last_frame.as_ref().unwrap().samples.as_slice())
            {
                identical_frames += 1;
            } else {
                identical_frames = 0;
            }
        }

        if identical_frames > 0 {
            // let's track now the duration as idle
            idle_duration = idle_duration.add(now.duration_since(last_now));
        } else {
            save_frame(&image, tc, tempdir.lock().unwrap().borrow(), file_name_for)?;
            time_codes.lock().unwrap().push(tc);
            last_frame = Some(image);
            identical_frames = 0;
        }
        last_now = now;
    }

    Ok(())
}

///
/// saves a frame as a tga file
pub fn save_frame(
    image: &ImageOnHeap,
    time_code: u128,
    tempdir: &TempDir,
    file_name_for: fn(&u128, &str) -> String,
) -> Result<()> {
    save_buffer(
        tempdir.path().join(file_name_for(&time_code, "tga")),
        &image.samples,
        image.layout.width,
        image.layout.height,
        image.color_hint.unwrap(),
    )
    .context("Cannot save frame")
}

///
/// starts the main program and keeps interacting with the user
/// blocks until termination
fn sub_shell_thread<T: AsRef<OsStr> + Clone>(program: T) -> Result<ExitStatus> {
    Command::new(program.clone())
        .spawn()
        .context(format!("failed to start {:?}", program.as_ref()))?
        .wait()
        .context("Something went wrong waiting for the sub shell.")
}

///
/// determines the WindowId either by env var 'WINDOWID'
/// or by the env var 'TERM_PROGRAM' and then asking the window manager for all visible windows
/// and finding the Terminal in that list
/// panics if WindowId was not was not there
fn current_win_id() -> Result<(WindowId, Option<String>)> {
    match env::var("WINDOWID") {
        Ok(win_id) => {
            let win_id = win_id
                .parse::<u64>()
                .context("Cannot parse env variable 'WINDOWID' as number")?;
            Ok((win_id, None))
        }
        Err(_) => {
            let terminal = env::var("TERM_PROGRAM").context(
                "Env variable 'TERM_PROGRAM' was empty but is needed for figure out the WindowId. Please set it to e.g. TERM_PROGRAM=alacitty",
            );
            if terminal.is_ok() {
                let (win_id, name) = get_window_id_for(terminal.unwrap()).context(
                    "Cannot determine the WindowId of this terminal. Please set env variable 'WINDOWID' and try again.",
                )?;
                Ok((win_id, Some(name)))
            } else {
                let api = setup()?;
                let win_id = api.get_active_window()
                    .context("Cannot determine the active window. Please set either env variable `WINDOWID` or `TERM_PROGRAM`. Check also `-l` to list all windows.")?;
                Ok((win_id, None))
            }
        }
    }
}

///
/// checks for imagemagick
/// and suggests the installation command if there are issues
fn check_for_imagemagick() -> Result<Output> {
    Command::new("magick")
        .arg("convert")
        .arg("--version")
        .output()
        .context("There is an issue with 'magick', make sure you have it installed: `brew install imagemagick`")
}

///
/// returns a new filename that does not yet exists.
/// like `t-rec.gif` or `t-rec_1.gif`
pub fn target_file() -> String {
    let mut suffix = "".to_string();
    let mut i = 0;
    while std::path::Path::new(format!("t-rec{}.gif", suffix).as_str()).exists() {
        i += 1;
        suffix = format!("_{}", i).to_string();
    }
    format!("t-rec{}.gif", suffix)
}

///
/// encapsulate the file naming convention
pub fn file_name_for(tc: &u128, ext: &str) -> String {
    format!("t-rec-frame-{:09}.{}", tc, ext)
}

///
/// finds the window id for a given terminal / programm by name
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
    println!("Window | Id");
    for (window_owner, window_id) in api.window_list()? {
        match (window_owner, window_id) {
            (Some(window_owner), window_id) => println!("{} | {}", window_owner, window_id),
            (_, _) => {}
        }
    }

    Ok(())
}
