use anyhow::{Context, Result};
use image::save_buffer;
use image::ColorType::Rgba8;
use log::{debug, error};
use std::borrow::Borrow;
use std::ops::{Add, Sub};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::broadcast::{error::RecvError, Receiver, Sender};

use crate::screenshot::{screenshot_file_name, ScreenshotInfo};
use crate::utils::{file_name_for, IMG_EXT};
use crate::{ImageOnHeap, PlatformApi, WindowId};

/// Events for the Photographer actor (capture thread).
///
/// The Photographer receives these events via a channel and responds accordingly.
/// Regular frame timing is handled internally via `recv_timeout`.
#[derive(Debug, Clone)]
pub enum CaptureEvent {
    /// Start capturing frames. The thread waits for this before beginning.
    Start,

    /// Take a screenshot (F2-triggered).
    /// The timecode_ms is the elapsed recording time when the screenshot was requested.
    Screenshot { timecode_ms: u128 },

    /// Stop recording and exit the capture loop.
    Stop,
}

/// Events for the Presenter actor (main thread).
///
/// The Presenter receives these events and shows appropriate visual feedback.
/// On macOS, this requires running on the main thread with NSRunLoop.
#[derive(Debug, Clone)]
pub enum FlashEvent {
    /// Show screenshot visual feedback (camera icon).
    ScreenshotTaken,

    #[allow(dead_code)]
    /// Show keystroke overlay
    KeyPressed { key: String },
}

/// Unified event type for routing to different actors.
///
/// This allows callers to send events without worrying about which actor
/// should receive them - the EventRouter handles the routing.
#[derive(Debug, Clone)]
pub enum Event {
    /// Event for the Photographer actor (capture thread).
    Capture(CaptureEvent),
    /// Event for the Presenter actor (main thread visual feedback).
    Flash(FlashEvent),
}

/// Routes events to the appropriate actor channels.
///
/// Simplifies event dispatch by providing a single `send()` method.
/// Silently ignores events if the target channel is `None`.
#[derive(Clone)]
pub struct EventRouter {
    capture_tx: Option<Sender<CaptureEvent>>,
    flash_tx: Option<Sender<FlashEvent>>,
}

impl EventRouter {
    /// Create a new EventRouter with optional channel senders.
    pub fn new(
        capture_tx: Option<Sender<CaptureEvent>>,
        flash_tx: Option<Sender<FlashEvent>>,
    ) -> Self {
        Self {
            capture_tx,
            flash_tx,
        }
    }

    /// Send an event to the appropriate actor.
    ///
    /// Routes `Event::Capture` to the Photographer and `Event::Flash` to the Presenter.
    /// Silently ignores if the target channel is `None`.
    pub fn send(&self, event: Event) {
        match event {
            Event::Capture(e) => {
                if let Some(ref tx) = self.capture_tx {
                    let _ = tx.send(e);
                }
            }
            Event::Flash(e) => {
                if let Some(ref tx) = self.flash_tx {
                    let _ = tx.send(e);
                }
            }
        }
    }
}

/// Configuration and shared state for the capture thread.
///
/// Groups all parameters needed for frame capture, making the API cleaner
/// and easier to extend with new options.
pub struct CaptureContext {
    /// Window ID to capture
    pub win_id: WindowId,
    /// Shared list to store frame timestamps
    pub time_codes: Arc<Mutex<Vec<u128>>>,
    /// Directory for saving frames
    pub tempdir: Arc<Mutex<TempDir>>,
    /// If true, save all frames without idle detection
    pub natural: bool,
    /// Maximum pause duration to preserve (None = skip all identical frames)
    pub idle_pause: Option<Duration>,
    /// Capture framerate (4-15 fps)
    pub fps: u8,
    /// List of captured screenshots
    pub screenshots: Option<Arc<Mutex<Vec<ScreenshotInfo>>>>,
}

impl CaptureContext {
    /// Calculate frame interval from fps, this is not used in tests
    pub fn frame_interval(&self) -> Duration {
        if cfg!(test) {
            Duration::from_millis(10) // Fast for testing
        } else {
            Duration::from_millis(1000 / self.fps as u64)
        }
    }
}

/// Captures screenshots periodically and decides which frames to keep.
///
/// Eliminates long idle periods while preserving brief pauses that aid
/// viewer comprehension. Adjusts timestamps to prevent playback gaps.
///
/// # Parameters
/// * `rx` - Channel to receive capture events (Start, Stop, Screenshot)
/// * `api` - Platform API for taking screenshots
/// * `ctx` - Capture configuration and shared state
///
/// # Behavior
/// - Waits for `CaptureEvent::Start` before beginning capture
/// - When identical frames are detected:
///   - Within threshold: frames are saved (preserves brief pauses)
///   - Beyond threshold: frames are skipped and time is subtracted from timestamps
/// - Exits on `CaptureEvent::Stop`
///
/// Example: 10-second idle with 3-second threshold → saves 3 seconds of pause,
///          skips 7 seconds, playback shows exactly 3 seconds.
pub fn capture_thread(
    mut rx: Receiver<CaptureEvent>,
    api: impl PlatformApi,
    ctx: CaptureContext,
) -> Result<()> {
    // Wait for Start event before beginning capture
    loop {
        match rx.blocking_recv() {
            Ok(CaptureEvent::Start) => break,
            Ok(CaptureEvent::Stop) => return Ok(()), // Stop before even starting
            Ok(_) => continue,                       // Ignore other events while waiting
            Err(_) => return Ok(()),                 // Channel closed
        }
    }

    let duration = ctx.frame_interval();
    let start = Instant::now();

    // Total idle time skipped (subtracted from timestamps to prevent gaps)
    let mut idle_duration = Duration::from_millis(0);

    // How long current identical frames have lasted
    let mut current_idle_period = Duration::from_millis(0);

    let mut last_frame: Option<ImageOnHeap> = None;
    let mut last_now = Instant::now();
    loop {
        // Wait for remaining time to hit target frame interval
        let elapsed = last_now.elapsed();
        if let Some(remaining) = duration.checked_sub(elapsed) {
            std::thread::sleep(remaining);
        }

        let screenshot_event_tc = match rx.try_recv() {
            Ok(CaptureEvent::Stop) => break,
            Ok(CaptureEvent::Start) => continue,
            Ok(CaptureEvent::Screenshot { timecode_ms }) => {
                debug!("Received Screenshot event with timecode {}", timecode_ms);
                Some(timecode_ms)
            }
            Err(TryRecvError::Closed) => break,
            Err(TryRecvError::Empty) => None,
            Err(_) => None,
        };
        let now = Instant::now();

        // Calculate timestamp with skipped idle time removed
        let effective_now = now.sub(idle_duration);
        let tc = effective_now.saturating_duration_since(start).as_millis();

        let image = api.capture_window_screenshot(ctx.win_id)?;
        let frame_duration = now.duration_since(last_now);

        // Handle screenshot if triggered by event
        let should_screenshot = screenshot_event_tc.is_some();

        if should_screenshot {
            let screenshot_tc = screenshot_event_tc.unwrap_or(tc);
            debug!("Taking screenshot at tc={}", screenshot_tc);
            if let Err(e) = save_screenshot(&image, screenshot_tc, &ctx) {
                error!("Failed to save screenshot: {}", e);
            } else {
                debug!("Screenshot saved successfully to tempdir");
            }
        }

        // Check if frame is identical to previous (skip check in natural mode)
        let frame_unchanged = !ctx.natural
            && last_frame
                .as_ref()
                .map(|last| image.samples.as_slice() == last.samples.as_slice())
                .unwrap_or(false);

        // Track duration of identical frames
        if frame_unchanged {
            current_idle_period = current_idle_period.add(frame_duration);
        } else {
            current_idle_period = Duration::from_millis(0);
        }

        // Decide whether to save this frame
        let should_save_frame = if frame_unchanged {
            let should_skip_for_compression = if let Some(threshold) = ctx.idle_pause {
                // Skip if idle exceeds threshold
                current_idle_period >= threshold
            } else {
                // No threshold: skip all identical frames
                true
            };

            if should_skip_for_compression {
                // Add skipped time to idle_duration for timestamp adjustment
                idle_duration = idle_duration.add(frame_duration);
                false
            } else {
                // Save frame (idle within threshold)
                true
            }
        } else {
            // Frame changed: reset idle tracking and save
            current_idle_period = Duration::from_millis(0);
            true
        };

        if should_save_frame {
            // Save frame and update state
            if let Err(e) = save_frame(
                &image,
                tc,
                ctx.tempdir.lock().unwrap().borrow(),
                file_name_for,
            ) {
                eprintln!("{}", &e);
                return Err(e);
            }
            ctx.time_codes.lock().unwrap().push(tc);

            // Store frame for next comparison
            last_frame = Some(image);
        }
        last_now = now;
    }

    Ok(())
}

/// Saves a screenshot to the temp directory.
fn save_screenshot(image: &ImageOnHeap, timecode_ms: u128, ctx: &CaptureContext) -> Result<()> {
    let tempdir = ctx.tempdir.lock().unwrap();
    let path = tempdir
        .path()
        .join(screenshot_file_name(timecode_ms, IMG_EXT));

    save_buffer(
        &path,
        &image.samples,
        image.layout.width,
        image.layout.height,
        image.color_hint.unwrap_or(Rgba8),
    )
    .context("Cannot save screenshot")?;

    debug!("Screenshot saved at timecode {timecode_ms}");

    // Record screenshot info
    if let Some(ref screenshots) = ctx.screenshots {
        screenshots.try_lock().unwrap().push(ScreenshotInfo {
            timecode_ms,
            temp_path: path.clone(),
        });
        debug!("ScreenshotInfo collected for timecode {timecode_ms}");
    } else {
        debug!("ScreenshotInfo collection skipped (no storage) for timecode {timecode_ms}");
    }

    Ok(())
}

/// Saves a frame as a BMP file.
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::sync::broadcast;

    /// Mock PlatformApi that returns predefined 1x1 pixel frames.
    struct TestApi {
        frames: Vec<Vec<u8>>,
        index: std::cell::Cell<usize>,
    }

    impl crate::PlatformApi for TestApi {
        fn capture_window_screenshot(
            &self,
            _: crate::WindowId,
        ) -> crate::Result<crate::ImageOnHeap> {
            let i = self.index.get();
            self.index.set(i + 1);
            let num_channels = 4;
            let pixel_width = 1;
            let pixel_height = 1;
            let frame_index = if i >= self.frames.len() {
                self.frames.len() - 1
            } else {
                i
            };
            Ok(Box::new(image::FlatSamples {
                samples: self.frames[frame_index].clone(),
                layout: image::flat::SampleLayout::row_major_packed(
                    num_channels,
                    pixel_width,
                    pixel_height,
                ),
                color_hint: Some(image::ColorType::Rgba8),
            }))
        }
        fn calibrate(&mut self, _: crate::WindowId) -> crate::Result<()> {
            Ok(())
        }
        fn window_list(&self) -> crate::Result<crate::WindowList> {
            Ok(vec![])
        }
        fn get_active_window(&self) -> crate::Result<crate::WindowId> {
            Ok(0)
        }
    }

    fn frames<T: AsRef<[u8]>>(sequence: T) -> Vec<Vec<u8>> {
        sequence
            .as_ref()
            .iter()
            .map(|&value| vec![value; 4])
            .collect()
    }

    fn run_capture_test(
        test_frames: Vec<Vec<u8>>,
        natural_mode: bool,
        idle_threshold: Option<Duration>,
    ) -> crate::Result<Vec<u128>> {
        let test_api = TestApi {
            frames: test_frames.clone(),
            index: Default::default(),
        };
        let captured_timestamps = Arc::new(Mutex::new(Vec::new()));
        let temp_directory = Arc::new(Mutex::new(TempDir::new()?));
        let (tx, rx) = broadcast::channel(10);

        let frame_interval = 10;
        let capture_duration_ms = (test_frames.len() as u64 * frame_interval) + 15;

        let tx_clone = tx.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(capture_duration_ms));
            let _ = tx_clone.send(CaptureEvent::Stop);
        });

        // Send start event
        tx.send(CaptureEvent::Start).unwrap();

        let ctx = CaptureContext {
            win_id: 0,
            time_codes: captured_timestamps.clone(),
            tempdir: temp_directory,
            natural: natural_mode,
            idle_pause: idle_threshold,
            fps: 4,
            screenshots: None,
        };
        capture_thread(rx, test_api, ctx)?;
        let result = captured_timestamps.lock().unwrap().clone();
        Ok(result)
    }

    #[test]
    fn test_event_router() {
        let (capture_tx, mut capture_rx) = broadcast::channel(16);
        let (flash_tx, mut flash_rx) = broadcast::channel(16);

        let router = EventRouter::new(Some(capture_tx), Some(flash_tx));

        router.send(Event::Capture(CaptureEvent::Start));
        router.send(Event::Flash(FlashEvent::ScreenshotTaken));

        assert!(matches!(
            capture_rx.blocking_recv(),
            Ok(CaptureEvent::Start)
        ));
        assert!(matches!(
            flash_rx.blocking_recv(),
            Ok(FlashEvent::ScreenshotTaken)
        ));
    }

    #[test]
    fn test_event_router_none_channels() {
        let router = EventRouter::new(None, None);
        // Should not panic when channels are None
        router.send(Event::Capture(CaptureEvent::Start));
        router.send(Event::Flash(FlashEvent::ScreenshotTaken));
    }
}
