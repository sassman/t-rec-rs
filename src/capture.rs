use std::borrow::Borrow;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use image::save_buffer;
use image::ColorType::Rgba8;
use smol::stream::block_on;
use tempfile::TempDir;

use crate::common::{Frame, Recorder};
use crate::utils::{file_name_for, IMG_EXT};
use crate::{Image, PlatformApi, WindowId};

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
    let recorder = Recorder::new(api, win_id, 4);
    let mut last_frame: Option<Frame> = None;
    let mut identical_frames = 0;
    let start = Instant::now();

    for frame in block_on(recorder) {
        if !force_natural {
            let image: &Image = frame.as_ref();
            if let Some(last_image) = last_frame.as_ref() {
                let last_image: &Image = last_image.as_ref();
                if image.samples.as_slice().eq(last_image.samples.as_slice()) {
                    identical_frames += 1;
                } else {
                    identical_frames = 0;
                }
            }
        }

        if identical_frames == 0 {
            let tc: &Instant = frame.as_ref();
            let tc = tc.duration_since(start).as_millis();
            save_image(&frame, tc, tempdir.lock().unwrap().borrow(), file_name_for)?;
            time_codes.lock().unwrap().push(tc);
            last_frame = Some(frame);
        }

        // when there is a message we should just stop
        if rx.recv_timeout(Duration::from_millis(1)).is_ok() {
            break;
        }
    }

    Ok(())
}

/// saves a frame as a tga file
pub fn save_image(
    image: impl AsRef<Image>,
    time_code: u128,
    tempdir: &TempDir,
    file_name_for: fn(&u128, &str) -> String,
) -> Result<()> {
    let image = image.as_ref();
    save_buffer(
        tempdir.path().join(file_name_for(&time_code, IMG_EXT)),
        &image.samples,
        image.layout.width,
        image.layout.height,
        image.color_hint.unwrap_or(Rgba8),
    )
    .context("Cannot save frame")
}
