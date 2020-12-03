#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::*;

#[cfg(not(target_os = "macos"))]
mod any;
#[cfg(not(target_os = "macos"))]
use any::*;

mod cli;
mod decor_effect;

use crate::cli::launch;

use crate::decor_effect::{apply_big_sur_corner_effect, apply_shadow_effect};
use crate::macos::capture_window_screenshot;
use anyhow::Context;
use anyhow::Result;
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

#[cfg(target_os = "linux")]
fn main() -> Result<(), std::io::Error> {
    unimplemented!("We're super sorry, right now t-rec is only supporting MacOS.\nIf you'd like to contribute checkout:\n\nhttps://github.com/sassman/t-rec-rs/issues/1\n")
}

#[cfg(target_os = "windows")]
fn main() -> Result<(), std::io::Error> {
    unimplemented!("We're super sorry, right now t-rec is only supporting MacOS.\nIf you'd like to contribute checkout:\n\nhttps://github.com/sassman/t-rec-rs/issues/2\n")
}

#[cfg(target_os = "macos")]
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
    capture_window_screenshot(win_id)?;

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
            capture_thread(&rx, win_id, time_codes, tempdir, force_natural)
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

    generate_gif_with_convert(
        &time_codes.lock().unwrap(),
        tempdir.lock().unwrap().borrow(),
    )
    .map(|_| ())?;

    Ok(())
}

///
/// escape sequences that clears the screen
fn clear_screen() {
    println!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

///
/// captures screenshots as file on disk
/// collects also the timecodes when they have been captured
/// stops once receiving something in rx
fn capture_thread(
    rx: &Receiver<()>,
    win_id: u32,
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
        let image = capture_window_screenshot(win_id)?;
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
fn current_win_id() -> Result<(u32, Option<String>)> {
    if env::var("WINDOWID").is_ok() {
        let win_id = env::var("WINDOWID")
            .unwrap()
            .parse::<u32>()
            .context("Cannot parse env variable 'WINDOWID' as number")?;
        Ok((win_id, None))
    } else {
        let terminal = env::var("TERM_PROGRAM").context(
            "Env variable 'TERM_PROGRAM' was empty but it is needed for determine the window id",
        )?;
        let (win_id, name) = get_window_id_for(terminal.to_owned()).context(
            format!(
            "Cannot determine the window id of {}. Please set env variable 'WINDOWID' and try again.", terminal),
        )?;
        Ok((win_id, Some(name)))
    }
}

///
/// checks for imagemagick
/// and suggests the installation command if there are issues
fn check_for_imagemagick() -> Result<Output> {
    Command::new("convert")
        .arg("--version")
        .output()
        .context("There is an issue with 'convert', make sure you have it installed: `brew install imagemagick`")
}

///
/// generating the final gif with help of convert
fn generate_gif_with_convert(time_codes: &[u128], tempdir: &TempDir) -> Result<()> {
    let target = target_file();
    println!("🎉 🚀 Generating {}", target);
    let mut cmd = Command::new("convert");
    cmd.arg("-loop").arg("0");
    let mut delay = 0;
    for tc in time_codes.iter() {
        delay = *tc - delay;
        cmd.arg("-delay")
            .arg(format!("{}", (delay as f64 * 0.1) as u64))
            .arg(tempdir.path().join(file_name_for(tc, "tga")));
        delay = *tc;
    }
    cmd.arg("-layers")
        .arg("Optimize")
        .arg(target)
        .output()
        .context("Cannot start 'convert' to generate the final gif")?;

    Ok(())
}

///
/// returns a new filename that does not yet exists.
/// like `t-rec.gif` or `t-rec_1.gif`
fn target_file() -> String {
    let mut suffix = "".to_string();
    let mut i = 0;
    while std::path::Path::new(format!("t-rec{}.gif", suffix).as_str()).exists() {
        i += 1;
        suffix = format!("_{}", i).to_string();
    }
    format!("t-rec{}.gif", suffix)
}

/// TODO implement a image native gif creation
// fn generate_gif(time_codes: &Vec<i128>) {}

///
/// encapsulate the file naming convention
fn file_name_for(tc: &u128, ext: &str) -> String {
    format!("t-rec-frame-{:09}.{}", tc, ext)
}
