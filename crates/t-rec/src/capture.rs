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
use tokio::sync::broadcast::Receiver;

use crate::event_router::{CaptureEvent, Event, LifecycleEvent};
use crate::screenshot::{screenshot_file_name, ScreenshotInfo};
use crate::utils::{file_name_for, IMG_EXT};
use crate::{ImageOnHeap, PlatformApi, WindowId};

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

/// Photographer actor: captures frames periodically, handles idle detection.
pub fn capture_thread(
    mut rx: Receiver<Event>,
    api: impl PlatformApi,
    ctx: CaptureContext,
) -> Result<()> {
    // Wait for Start event before beginning capture
    loop {
        match rx.blocking_recv() {
            Ok(Event::Capture(CaptureEvent::Start)) => break,
            Ok(Event::Capture(CaptureEvent::Stop))
            | Ok(Event::Lifecycle(LifecycleEvent::Shutdown)) => return Ok(()),
            Ok(_) => continue,
            Err(_) => return Ok(()),
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
            Ok(Event::Capture(CaptureEvent::Stop))
            | Ok(Event::Lifecycle(LifecycleEvent::Shutdown)) => break,
            Ok(Event::Capture(CaptureEvent::Start)) => continue,
            Ok(Event::Capture(CaptureEvent::Screenshot { timecode_ms })) => {
                debug!("Received Screenshot event with timecode {}", timecode_ms);
                Some(timecode_ms)
            }
            Ok(_) => None, // Ignore Flash events
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
            Ok(crate::WindowId::default())
        }
    }

    /// Convert a byte sequence into RGBA pixel frames (each byte becomes one 1x1 frame).
    fn frames<T: AsRef<[u8]>>(sequence: T) -> Vec<Vec<u8>> {
        sequence
            .as_ref()
            .iter()
            .map(|&value| vec![value; 4])
            .collect()
    }

    /// Run a capture test with the given frames and settings, returning captured timestamps.
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
        let (tx, rx) = broadcast::channel::<Event>(10);

        let frame_interval = 10;
        let capture_duration_ms = (test_frames.len() as u64 * frame_interval) + 15;

        let tx_clone = tx.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(capture_duration_ms));
            let _ = tx_clone.send(Event::Capture(CaptureEvent::Stop));
        });

        // Send start event
        tx.send(Event::Capture(CaptureEvent::Start)).unwrap();

        let ctx = CaptureContext {
            win_id: crate::WindowId::default(),
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
    fn test_all_unique_frames_are_captured() {
        // Each frame is different, all should be saved
        let test_frames = frames([1, 2, 3, 4, 5]);
        let timestamps = run_capture_test(test_frames, false, None).unwrap();

        assert_eq!(timestamps.len(), 5, "All unique frames should be captured");
    }

    #[test]
    fn test_identical_frames_are_skipped_by_default() {
        // Frames: A, A, A, B, B, C (3 unique values)
        let test_frames = frames([1, 1, 1, 2, 2, 3]);
        let timestamps = run_capture_test(test_frames, false, None).unwrap();

        // Only first occurrence of each unique frame should be saved
        assert_eq!(
            timestamps.len(),
            3,
            "Identical consecutive frames should be skipped"
        );
    }

    #[test]
    fn test_natural_mode_preserves_all_frames() {
        // Same sequence but natural mode keeps everything
        let test_frames = frames([1, 1, 1, 2, 2, 3]);
        let timestamps = run_capture_test(test_frames, true, None).unwrap();

        assert_eq!(timestamps.len(), 6, "Natural mode should keep all frames");
    }

    #[test]
    fn test_idle_threshold_preserves_short_pauses() {
        // With a generous threshold, some identical frames should be preserved
        // Frame interval is 10ms in test mode, so 50ms threshold allows ~5 identical frames
        let test_frames = frames([1, 1, 1, 2]); // 3 identical then change
        let timestamps =
            run_capture_test(test_frames, false, Some(Duration::from_millis(50))).unwrap();

        // All frames within threshold should be kept
        assert!(
            timestamps.len() >= 3,
            "Frames within idle threshold should be preserved"
        );
    }

    #[test]
    fn test_idle_threshold_skips_long_pauses() {
        // With a short threshold, long identical sequences get truncated
        let test_frames = frames([1, 1, 1, 1, 1, 1, 1, 1, 2]); // 8 identical then change
        let timestamps =
            run_capture_test(test_frames, false, Some(Duration::from_millis(25))).unwrap();

        // Should have fewer than 9 frames (some idle skipped)
        assert!(
            timestamps.len() < 9,
            "Long idle periods beyond threshold should be skipped"
        );
        // But should still have the unique frames
        assert!(
            timestamps.len() >= 2,
            "Unique frames should still be captured"
        );
    }

    #[test]
    fn test_alternating_frames_all_captured() {
        // Rapidly changing content: A, B, A, B, A
        let test_frames = frames([1, 2, 1, 2, 1]);
        let timestamps = run_capture_test(test_frames, false, None).unwrap();

        assert_eq!(
            timestamps.len(),
            5,
            "Alternating frames should all be captured"
        );
    }

    #[test]
    fn test_timestamps_are_monotonically_increasing() {
        let test_frames = frames([1, 2, 3, 4, 5]);
        let timestamps = run_capture_test(test_frames, false, None).unwrap();

        for window in timestamps.windows(2) {
            assert!(
                window[1] > window[0],
                "Timestamps should be strictly increasing"
            );
        }
    }

    #[test]
    fn test_stop_event_terminates_capture() {
        let test_api = TestApi {
            frames: frames([1, 2, 3]),
            index: Default::default(),
        };
        let captured_timestamps = Arc::new(Mutex::new(Vec::new()));
        let temp_directory = Arc::new(Mutex::new(TempDir::new().unwrap()));
        let (tx, rx) = broadcast::channel::<Event>(10);

        // Send start then immediate stop
        tx.send(Event::Capture(CaptureEvent::Start)).unwrap();
        tx.send(Event::Capture(CaptureEvent::Stop)).unwrap();

        let ctx = CaptureContext {
            win_id: crate::WindowId::default(),
            time_codes: captured_timestamps.clone(),
            tempdir: temp_directory,
            natural: false,
            idle_pause: None,
            fps: 4,
            screenshots: None,
        };

        capture_thread(rx, test_api, ctx).unwrap();
        // Should terminate quickly without error
    }

    #[test]
    fn test_shutdown_event_terminates_capture() {
        let test_api = TestApi {
            frames: frames([1, 2, 3]),
            index: Default::default(),
        };
        let captured_timestamps = Arc::new(Mutex::new(Vec::new()));
        let temp_directory = Arc::new(Mutex::new(TempDir::new().unwrap()));
        let (tx, rx) = broadcast::channel::<Event>(10);

        // Shutdown before start should exit cleanly
        tx.send(Event::Lifecycle(LifecycleEvent::Shutdown)).unwrap();

        let ctx = CaptureContext {
            win_id: crate::WindowId::default(),
            time_codes: captured_timestamps.clone(),
            tempdir: temp_directory,
            natural: false,
            idle_pause: None,
            fps: 4,
            screenshots: None,
        };

        capture_thread(rx, test_api, ctx).unwrap();
    }

    #[test]
    fn test_frames_saved_to_tempdir() {
        let test_frames = frames([1, 2, 3]);
        let test_api = TestApi {
            frames: test_frames,
            index: Default::default(),
        };
        let captured_timestamps = Arc::new(Mutex::new(Vec::new()));
        let temp_directory = Arc::new(Mutex::new(TempDir::new().unwrap()));
        let (tx, rx) = broadcast::channel::<Event>(10);

        // Clone the Arc to keep tempdir alive for file check
        let temp_directory_check = temp_directory.clone();

        let tx_clone = tx.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(45));
            let _ = tx_clone.send(Event::Capture(CaptureEvent::Stop));
        });

        tx.send(Event::Capture(CaptureEvent::Start)).unwrap();

        let ctx = CaptureContext {
            win_id: crate::WindowId::default(),
            time_codes: captured_timestamps.clone(),
            tempdir: temp_directory,
            natural: false,
            idle_pause: None,
            fps: 4,
            screenshots: None,
        };

        capture_thread(rx, test_api, ctx).unwrap();

        // Check that files were actually created (tempdir still alive via temp_directory_check)
        let temp_guard = temp_directory_check.lock().unwrap();
        let files: Vec<_> = std::fs::read_dir(temp_guard.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        let num_timestamps = captured_timestamps.lock().unwrap().len();
        assert_eq!(
            files.len(),
            num_timestamps,
            "Number of saved files should match timestamps"
        );
    }
}
