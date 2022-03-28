use crossbeam_channel::Receiver;
use rayon::ThreadPoolBuilder;
use std::borrow::Borrow;
use std::ops::Sub;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::TempDir;

use crate::capture::frame_comparator::FrameComparator;
use crate::capture::frame_essence::{FrameDropStrategy, FrameEssence};
use crate::capture::{Framerate, Timecode};
use crate::utils::file_name_for;
use crate::{Frame, PlatformApi, Result, WindowId};

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
    let mut last_time = Instant::now();
    let api = Arc::new(api);
    let comparator = Arc::new(Mutex::new(FrameComparator::new(
        frame_drop_strategy.clone(),
    )));

    pool.scope(|s| {
        loop {
            let delta = Instant::now().saturating_duration_since(last_time);
            let sleep_time = duration.sub(delta);
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
                let comp = comparator.clone();
                let time_codes = time_codes.clone();
                move |_| {
                    let tc: Timecode = now.saturating_duration_since(start).as_millis().into();
                    if let Ok(frame) = api.capture_window_screenshot(win_id) {
                        let frame: Frame = frame.into();
                        let frame_essence = FrameEssence::new(&frame, &tc);
                        {
                            let mut lock = comp.try_lock();
                            if let Ok(ref mut mutex) = lock {
                                if mutex.should_drop_frame(frame_essence) {
                                    return;
                                }
                            } else {
                                dbg!(" locking failed...");
                            }
                        }
                        frame.save(&tc, tempdir.borrow(), file_name_for).unwrap();
                        time_codes.lock().unwrap().push(tc);
                    }
                }
            });

            last_time = now;
        }
    });

    Ok(())
}
