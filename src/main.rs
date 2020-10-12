#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos::{get_window_id_for, ls_win, screenshot_and_save};

#[cfg(not(target_os = "macos"))]
mod any;
#[cfg(not(target_os = "macos"))]
use any::{get_window_id_for, ls_win, screenshot_and_save};

mod cli;
use crate::cli::launch;

use anyhow::Context;
use anyhow::Result;
use std::borrow::Borrow;
use std::ffi::OsStr;
use std::process::{Command, ExitStatus, Output};
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, thread};
use tempfile::TempDir;

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
        thread::spawn(move || -> Result<()> { capture_thread(&rx, time_codes, tempdir) })
    };
    let interact = thread::spawn(move || -> Result<()> { sub_shell_thread(&program).map(|_| ()) });

    clear_screen();
    println!(
        "Frame cache dir: {:?}",
        tempdir.lock().expect("Cannot lock tempdir resource").path()
    );
    println!("Press Ctrl+D to end recording");

    interact
        .join()
        .unwrap()
        .context("Cannot launch the sub shell")?;
    tx.send(()).context("Cannot stop the recording thread")?;
    photograph
        .join()
        .unwrap()
        .context("Cannot launch the recording thread")?;
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
    time_codes: Arc<Mutex<Vec<u128>>>,
    tempdir: Arc<Mutex<TempDir>>,
) -> Result<()> {
    let win_id = current_win_id()?;
    let duration = Duration::from_millis(250);
    let start = Instant::now();
    loop {
        // blocks for a timeout
        if rx.recv_timeout(duration).is_ok() {
            break;
        }
        let tc = Instant::now().saturating_duration_since(start).as_millis();
        time_codes.lock().unwrap().push(tc);
        screenshot_and_save(win_id, tc, tempdir.lock().unwrap().borrow(), file_name_for)?
    }

    Ok(())
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
fn current_win_id() -> Result<u32> {
    if env::var("WINDOWID").is_ok() {
        env::var("WINDOWID")
            .unwrap()
            .parse::<u32>()
            .context("Cannot parse env variable 'WINDOWID' as number")
    } else {
        let terminal = env::var("TERM_PROGRAM").context(
            "Env variable 'TERM_PROGRAM' was empty but is needed for figure out the WindowId",
        )?;
        get_window_id_for(terminal).context(
            "Cannot determine the WindowId of this terminal. Please set env variable 'WINDOWID' and try again.",
        )
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
    let target = "t-rec.gif";
    println!(
        "\n🎉 🚀 Generating {:?} out of {} frames!",
        target,
        time_codes.len()
    );
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

/// TODO implement a image native gif creation
// fn generate_gif(time_codes: &Vec<i128>) {}

///
/// encapsulate the file naming convention
fn file_name_for(tc: &u128, ext: &str) -> String {
    format!("t-rec-frame-{:09}.{}", tc, ext)
}
