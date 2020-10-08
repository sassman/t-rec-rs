mod window_id;

use crate::window_id::get_window_id_for;
use anyhow::Context;
use core_foundation_sys::base::CFShow;
use core_graphics::base::{kCGImageAlphaNone, kCGImageAlphaNoneSkipFirst};
use core_graphics::color_space::kCGColorSpaceGenericRGB;
use core_graphics::display::*;
use core_graphics::geometry::CG_ZERO_RECT;
use core_graphics::image::CGImageRef;
use core_graphics::window::kCGWindowIsOnscreen;
use glob::glob;
use image::flat::SampleLayout;
use image::imageops::crop;
use image::{
    load_from_memory, load_from_memory_with_format, Bgra, DynamicImage, FlatSamples, GenericImage,
    GenericImageView, ImageBuffer, ImageFormat, Rgba,
};
use image::{save_buffer, ColorType};
use std::borrow::Borrow;
use std::env::{args, temp_dir};
use std::ffi::OsStr;
use std::process::{exit, Command};
use std::sync::mpsc::Receiver;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, thread};
use tempdir::TempDir;

fn main() -> Result<(), std::io::Error> {
    let program = {
        let default = "/bin/sh".to_owned();
        if args().len() > 1 {
            args().skip(1).next().unwrap_or(default)
        } else {
            env::var("SHELL").unwrap_or(default)
        }
    };
    // the nice thing is the cleanup on drop
    let tempdir = Arc::new(Mutex::new(
        TempDir::new(format!("trec-{}", std::process::id()).as_str())
            .expect("Failed to create tempdir."),
    ));
    clear_screen();
    println!("tmp path: {:?}", tempdir.lock().unwrap().path());
    let time_codes = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = mpsc::channel();
    let photograph = {
        let tempdir = tempdir.clone();
        let time_codes = time_codes.clone();
        thread::spawn(move || capture_thread(&rx, time_codes, tempdir))
    };
    let interact = thread::spawn(move || sub_shell_thread(&program));

    let _ = interact.join();
    tx.send(());
    let _ = photograph.join();
    generate_gif_with_convert(
        &time_codes.lock().unwrap(),
        tempdir.lock().unwrap().borrow(),
    );

    Ok(())
}

fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H\n", esc = 27 as char);
}

///
/// captures screenshots as file on disk
/// collects also the timecodes when they have been captured
/// stops once receiving something in rx
fn capture_thread(
    rx: &Receiver<()>,
    time_codes: Arc<Mutex<Vec<u128>>>,
    tempdir: Arc<Mutex<TempDir>>,
) {
    let win_id = current_win_id();
    let duration = Duration::from_millis(250);
    let start = Instant::now();
    loop {
        // blocks for a timeout
        if let Ok(_) = rx.recv_timeout(duration) {
            break;
        }
        let tc = Instant::now().saturating_duration_since(start).as_millis();
        time_codes.lock().unwrap().push(tc);
        screenshot_and_save(win_id, tc, tempdir.lock().unwrap().borrow());
    }
}

///
/// starts the main program and keeps interacting with the user
/// blocks until termination
fn sub_shell_thread<T: AsRef<OsStr> + Clone>(program: T) {
    println!("Press Ctrl+D to end recording");
    Command::new(program.clone())
        .spawn()
        .with_context(move || format!("failed to start {:?}", program.as_ref()))
        .unwrap()
        .wait();
}

///
/// determines the WindowId either by env var 'WINDOWID'
/// or by the env var 'TERM_PROGRAM' and then asking the window manager for all visible windows
/// and finding the Terminal in that list
/// panics if WindowId was not was not there
fn current_win_id() -> CGWindowID {
    use core_foundation::base::*;
    use core_foundation::number::*;
    use core_foundation::string::*;
    use std::ffi::CStr;
    use std::os::raw::c_void;

    let win_id = env::var("WINDOWID");
    if win_id.is_ok() {
        let win_id = win_id
            .unwrap()
            .parse::<u32>()
            .expect("Env variable 'WINDOWID' was not a valid number");
        return win_id;
    }
    let terminal = env::var("TERM_PROGRAM")
        .expect("Env variable 'TERM_PROGRAM' was empty but is needed for figure out the WindowId");

    let win_id = get_window_id_for(terminal).expect(
        "Cannot determine the WindowId of this terminal. Please set env variable 'WINDOWID' and try again.",
    );

    win_id.into()
}

///
/// generating the final gif with help of convert
fn generate_gif_with_convert(time_codes: &Vec<u128>, tempdir: &TempDir) {
    let target = "t-rec.gif";
    println!(
        "\n\n🎉 🚀 Generating {:?} out of {} frames!",
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
        .expect("failed to execute process");
}

/// TODO implement a image native gif creation
fn generate_gif(time_codes: &Vec<i128>) {}

fn file_name_for(tc: &u128, ext: &str) -> String {
    format!("t-rec-frame-{:09}.{}", tc, ext)
}

///
/// grabs a screenshot by window id and
/// saves it as a tga file
fn screenshot_and_save(win_id: CGWindowID, time_code: u128, tempdir: &TempDir) {
    let (w, h, channels, raw_data) = {
        let image = unsafe {
            CGDisplay::screenshot(
                CGRectNull,
                kCGWindowListOptionIncludingWindow | kCGWindowListExcludeDesktopElements,
                win_id,
                kCGWindowImageNominalResolution
                    | kCGWindowImageBoundsIgnoreFraming
                    | kCGWindowImageShouldBeOpaque,
            )
        }
        .expect("failed to get a screenshot");

        let img_ref: &CGImageRef = &image;
        let (_wrong_width, h) = (img_ref.width() as u32, img_ref.height() as u32);
        let raw_data: Vec<_> = img_ref.data().to_vec();
        let byte_per_row = img_ref.bytes_per_row() as u32;
        let bit_per_pixel = img_ref.bits_per_pixel() as u32;
        let channels = img_ref.bits_per_component() as u32 / 8;
        // the buffer must be as long as the row length x height
        assert_eq!(byte_per_row * h, raw_data.len() as u32);
        // CAUTION this took me hours of my life time to figure out,
        // the width is not trust worthy, only the buffer dimensions are real
        // actual width, based on the buffer dimensions
        let w = byte_per_row / ((bit_per_pixel / 8) * channels);
        assert_eq!(bit_per_pixel / 8 * w * channels, byte_per_row);

        (w, h, channels as u8, raw_data)
    };

    let color = ColorType::Bgra8;
    let buffer = FlatSamples {
        samples: raw_data,
        layout: SampleLayout::row_major_packed(channels, w, h),
        color_hint: Some(color),
    };

    save_buffer(
        tempdir.path().join(file_name_for(&time_code, "tga")),
        &buffer.samples,
        w,
        h,
        color,
    )
    .expect("failed to save a frame");
}
