use anyhow::Context;
use crossbeam_channel::Receiver;
use image::save_buffer;
use image::ColorType::Rgba8;
use rayon::ThreadPoolBuilder;
use std::borrow::Borrow;
use std::ops::Sub;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::TempDir;

use crate::capture::Framerate;
use crate::utils::{file_name_for, IMG_EXT};
use crate::{ImageOnHeap, PlatformApi, Result, WindowId};

#[derive(Eq, PartialEq)]
pub enum FrameDropStrategy {
    DoNotDropAny,
    DropIdenticalFrames,
}

#[derive(Clone)]
struct FrameComparator<'a> {
    last_frame: Option<ImageOnHeap>,
    strategy: &'a FrameDropStrategy,
}

impl<'a> FrameComparator<'a> {
    pub fn drop_frame(&mut self, frame: ImageOnHeap) -> bool {
        if self.last_frame.is_none() {
            self.last_frame = Some(frame);
            false
        } else {
            false
        }
    }
}

/// captures screenshots as file on disk
/// collects also the timecodes when they have been captured
/// stops once receiving something in rx
pub fn capture_thread(
    rx: &Receiver<()>,
    api: impl PlatformApi + Sync,
    win_id: WindowId,
    time_codes: Arc<Mutex<Vec<u128>>>,
    tempdir: Arc<TempDir>,
    frame_drop_strategy: &FrameDropStrategy,
    framerate: &Framerate,
) -> Result<()> {
    let pool = ThreadPoolBuilder::default().build()?;
    let duration = Duration::from_secs(1) / *framerate.as_ref();
    let start = Instant::now();
    let mut idle_duration = Duration::from_millis(0);
    let mut last_frame: Option<ImageOnHeap> = None;
    let mut identical_frames = 0;
    let mut last_time = Instant::now();
    let api = Arc::new(api);
    let comp = Arc::new(FrameComparator {
        last_frame: None,
        strategy: frame_drop_strategy,
    });
    // let rx = Arc::new(rx);
    // let mut results: Arc<Mutex<Vec<Result<()>>>> = Arc::new(Mutex::new(Vec::new()));

    pool.scope(|s| {
        loop {
            let delta = Instant::now().saturating_duration_since(last_time);
            let sleep_time = duration.sub(delta);
            // thread::sleep(sleep_time);
            // blocks for a timeout
            if rx.recv_timeout(sleep_time).is_ok() {
                if pool.current_thread_has_pending_tasks().unwrap_or(false) {
                    println!(
                        "there is a backlog of frames that needs to be persisted, this may take a bit ...",
                    );
                }
                return;
            }
            let now = Instant::now();
            let timecode = now.saturating_duration_since(start).as_millis();
            // let effective_now = now.sub(idle_duration);
            let api = api.clone();
            let tempdir = tempdir.clone();
            time_codes.lock().unwrap().push(timecode);

            s.spawn(move |_| {
                let frame = api.capture_window_screenshot(win_id);

                if let Ok(frame) = frame {
                    save_frame(&frame, timecode, tempdir.borrow(), file_name_for).unwrap();
                    // results.borrow_mut().lock().unwrap().push(result);
                }
            });

            /*
            let image = api.capture_window_screenshot(win_id)?;
            if frame_drop_strategy == &FrameDropStrategy::DropIdenticalFrames {
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
             */

            last_time = now;
        }
    });

    Ok(())
}

fn capture_and_save_frame(
    api: Arc<impl PlatformApi + Sync>,
    win_id: WindowId,
    timecode: u128,
    tempdir: Arc<Mutex<TempDir>>,
    file_name_fn: fn(&u128, &str) -> String,
) -> Result<()> {
    let mut result: Result<()> = Ok(());
    rayon::scope(|s| s.spawn(|_| {}));

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
