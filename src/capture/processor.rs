use blockhash::{blockhash256, Blockhash256};
use crossbeam_channel::Receiver;
use rayon::ThreadPoolBuilder;
use std::borrow::Borrow;
use std::ops::Sub;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::TempDir;

use crate::capture::{Framerate, Timecode};
use crate::utils::file_name_for;
use crate::{Frame, PlatformApi, Result, WindowId};

#[derive(Eq, PartialEq, Clone)]
pub enum FrameDropStrategy {
    DoNotDropAny,
    DropIdenticalFrames,
}

#[derive(Clone)]
struct FrameComparator {
    last_frames: Vec<(Timecode, Timecode, Blockhash256)>,
    _strategy: FrameDropStrategy,
}

impl FrameComparator {
    pub fn should_drop_frame(&mut self, timecode: &Timecode, frame: &Frame) -> bool {
        let hash = blockhash256(frame);
        if let Some((_last_time_code, _other_time_code, last_hash)) = self.last_frames.last() {
            let last_eq = last_hash == &hash;
            if !last_eq {
                self.last_frames.pop();
                self.last_frames.push((timecode.clone(), hash));
            }
            last_eq
        } else {
            self.last_frames.push((timecode.clone(), hash));
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
    time_codes: Arc<Mutex<Vec<Timecode>>>,
    tempdir: Arc<TempDir>,
    frame_drop_strategy: &FrameDropStrategy,
    framerate: &Framerate,
) -> Result<()> {
    let pool = ThreadPoolBuilder::default().build()?;
    let duration = Duration::from_secs(1) / *framerate.as_ref();
    let start = Instant::now();
    // let mut idle_duration = Duration::from_millis(0);
    // let mut last_frame: Option<ImageOnHeap> = None;
    // let mut identical_frames = 0;
    let mut last_time = Instant::now();
    let api = Arc::new(api);
    let comp = Arc::new(Mutex::new(FrameComparator {
        last_frames: Vec::new(),
        _strategy: frame_drop_strategy.clone(),
    }));
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
                        "there is a backlog of frames that needs to be stored, this may take a bit ...",
                    );
                }
                return;
            }
            let now = Instant::now();
            s.spawn({
                let api = api.clone();
                let tempdir = tempdir.clone();
                let comp = comp.clone();
                let time_codes = time_codes.clone();
                move |_| {
                    let tc: Timecode = now.saturating_duration_since(start).as_millis().into();

                    let frame = api.capture_window_screenshot(win_id);
                    if let Ok(frame) = frame {
                        let frame: Frame = frame.into();
                        if comp.lock().unwrap().should_drop_frame(&tc, &frame) {
                            return;
                        }
                        frame.save(&tc, tempdir.borrow(), file_name_for).unwrap();
                        time_codes.lock().unwrap().push(tc);
                        // results.borrow_mut().lock().unwrap().push(result);
                    }
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
