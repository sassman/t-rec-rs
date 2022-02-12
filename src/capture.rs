use anyhow::{Context, Result};
use image::save_buffer;
use image::ColorType::Rgba8;
use std::borrow::Borrow;
use std::ops::{Add, Sub};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::TempDir;

use crate::utils::{file_name_for, IMG_EXT};
use crate::{ImageOnHeap, PlatformApi, WindowId};

/// captures screenshots as file on disk
/// collects also the timecodes when they have been captured
/// stops once receiving something in rx
pub fn capture_thread(
    rx: &Receiver<()>,
    api: impl PlatformApi,
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
            if let Err(e) = save_frame(&image, tc, tempdir.lock().unwrap().borrow(), file_name_for)
            {
                eprintln!("{}", &e);
                return Err(e);
            }
            time_codes.lock().unwrap().push(tc);
            last_frame = Some(image);
            identical_frames = 0;
        }
        last_now = now;
    }

    Ok(())
}

/// saves a frame as a tga file
pub fn save_frame(
    image: &ImageOnHeap,
    time_code: u128,
    tempdir: &TempDir,
    file_name_for: fn(&u128, &str) -> String,
) -> Result<()> {
    save_buffer(
        tempdir.path().join(file_name_for(&time_code, IMG_EXT)),
        &image.samples,
        image.layout.width,
        image.layout.height,
        image.color_hint.unwrap_or(Rgba8),
    )
    .context("Cannot save frame")
}
