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

/// Captures screenshots periodically and decides which frames to keep.
///
/// Eliminates long idle periods while preserving brief pauses that aid
/// viewer comprehension. Adjusts timestamps to prevent playback gaps.
///
/// # Parameters
/// * `rx` - Channel to receive stop signal
/// * `api` - Platform API for taking screenshots
/// * `win_id` - Window ID to capture
/// * `time_codes` - Shared list to store frame timestamps
/// * `tempdir` - Directory for saving frames
/// * `force_natural` - If true, save all frames (no skipping)
/// * `idle_pause` - Maximum pause duration to preserve for viewer comprehension:
///   - `None`: Skip all identical frames (maximum compression)
///   - `Some(duration)`: Preserve pauses up to this duration, skip beyond
///
/// # Behavior
/// When identical frames are detected:
/// - Within threshold: frames are saved (preserves brief pauses)
/// - Beyond threshold: frames are skipped and time is subtracted from timestamps
///
/// Example: 10-second idle with 3-second threshold → saves 3 seconds of pause,
///          skips 7 seconds, playback shows exactly 3 seconds.
pub fn capture_thread(
    rx: &Receiver<()>,
    api: impl PlatformApi,
    win_id: WindowId,
    time_codes: Arc<Mutex<Vec<u128>>>,
    tempdir: Arc<Mutex<TempDir>>,
    force_natural: bool,
    idle_pause: Option<Duration>,
) -> Result<()> {
    #[cfg(test)]
    let duration = Duration::from_millis(10); // Fast for testing
    #[cfg(not(test))]
    let duration = Duration::from_millis(250); // Production speed
    let start = Instant::now();

    // Total idle time skipped (subtracted from timestamps to prevent gaps)
    let mut idle_duration = Duration::from_millis(0);

    // How long current identical frames have lasted
    let mut current_idle_period = Duration::from_millis(0);

    let mut last_frame: Option<ImageOnHeap> = None;
    let mut last_now = Instant::now();
    loop {
        // blocks for a timeout
        if rx.recv_timeout(duration).is_ok() {
            break;
        }
        let now = Instant::now();

        // Calculate timestamp with skipped idle time removed
        let effective_now = now.sub(idle_duration);
        let tc = effective_now.saturating_duration_since(start).as_millis();

        let image = api.capture_window_screenshot(win_id)?;
        let frame_duration = now.duration_since(last_now);

        // Check if frame is identical to previous (skip check in natural mode)
        let frame_unchanged = !force_natural
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
            let should_skip_for_compression = if let Some(threshold) = idle_pause {
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
            if let Err(e) = save_frame(&image, tc, tempdir.lock().unwrap().borrow(), file_name_for)
            {
                eprintln!("{}", &e);
                return Err(e);
            }
            time_codes.lock().unwrap().push(tc);

            // Store frame for next comparison
            last_frame = Some(image);
        }
        last_now = now;
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
    use std::sync::mpsc;
    use tempfile::TempDir;

    /// Mock PlatformApi that returns predefined 1x1 pixel frames.
    /// After all frames are used, keeps returning the last frame.
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
            // Return 1x1 RGBA pixel data - stop at last frame instead of cycling
            let num_channels = 4; // RGBA
            let pixel_width = 1;
            let pixel_height = 1;
            let frame_index = if i >= self.frames.len() {
                self.frames.len() - 1 // Stay on last frame
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

    /// Converts byte array to frame data for testing.
    /// Each byte becomes all 4 channels of an RGBA pixel.
    /// Same values = identical frames, different values = changed content.
    ///
    /// Example: frames(&[1,2,2,3]) creates 4 frames where frames 1 and 2 are identical
    fn frames<T: AsRef<[u8]>>(sequence: T) -> Vec<Vec<u8>> {
        sequence
            .as_ref()
            .iter()
            .map(|&value| vec![value; 4])
            .collect()
    }

    /// Runs capture_thread with test frames and returns timestamps of saved frames.
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
        let (stop_signal_tx, stop_signal_rx) = mpsc::channel();

        // Run capture for (frame_count * 10ms) + 15ms buffer
        let frame_interval = 10; // ms per frame in test mode
        let capture_duration_ms = (test_frames.len() as u64 * frame_interval) + 15;

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(capture_duration_ms));
            let _ = stop_signal_tx.send(());
        });

        let timestamps_clone = captured_timestamps.clone();
        capture_thread(
            &stop_signal_rx,
            test_api,
            0,
            timestamps_clone,
            temp_directory,
            natural_mode,
            idle_threshold,
        )?;
        let result = captured_timestamps.lock().unwrap().clone();
        Ok(result)
    }

    /// Analyzes captured frame timestamps to verify compression worked correctly.
    ///
    /// Returns a tuple of:
    /// - Frame count: Total number of frames captured
    /// - Total duration: Time span from first to last frame (ms)
    /// - Has gaps: Whether gaps over 25ms exist that indicate compression failure
    ///
    /// Gaps over 25ms between consecutive frames indicate the timeline
    /// compression algorithm failed to maintain continuous playback.
    fn analyze_timeline(timestamps: &[u128]) -> (usize, u128, bool) {
        let max_normal_gap = 25; // Maximum expected gap between consecutive frames (ms)

        let frame_count = timestamps.len();
        let total_duration_ms = if timestamps.len() > 1 {
            timestamps.last().unwrap() - timestamps.first().unwrap()
        } else {
            0
        };

        // Detect gaps over 25ms indicating timeline compression failure
        let has_compression_gaps = timestamps
            .windows(2)
            .any(|window| window[1] - window[0] > max_normal_gap);

        (frame_count, total_duration_ms, has_compression_gaps)
    }

    /// Tests idle frame compression behavior.
    ///
    /// Verifies:
    /// - Correct frame count based on threshold settings
    /// - No timestamp gaps over 25ms after compression (ensures smooth playback)
    /// - Natural mode saves all frames regardless of content
    /// - Threshold boundaries work correctly (e.g., exactly at 30ms)
    #[test]
    fn test_idle_pause() -> crate::Result<()> {
        // Test format: (frames, natural_mode, threshold_ms, expected_count, description)
        // - frames: byte array where same value = identical frame
        // - natural_mode: true = save all, false = skip identical
        // - threshold_ms: None = skip all identical, Some(n) = keep up to n ms
        // - expected_count: range due to timing variations
        // - [..] converts array to slice (required for different array sizes)
        //
        // Example: [1,2,2,2,3] = active frame, 3 idle frames, then active frame
        [
            // Natural mode - saves all frames regardless of content
            (
                vec![1, 1, 1],
                true,
                None,
                3..=4,
                "natural mode preserves all frames",
            ),
            // Basic single frame test
            (vec![1], false, None, 1..=2, "single frame recording"),
            // All different frames - no idle to compress
            (
                vec![1, 2, 3],
                false,
                None,
                3..=3,
                "all different frames saved",
            ),
            // Basic idle compression
            (
                vec![1, 1, 1],
                false,
                None,
                1..=1,
                "3 identical frames → 1 frame",
            ),
            // Long threshold preserves short sequences
            (
                vec![1, 1, 1],
                false,
                Some(500),
                3..=4,
                "500ms threshold preserves 30ms idle",
            ),
            // Multiple idle periods compress independently
            (
                vec![1, 2, 2, 2, 3, 4, 4, 4],
                false,
                None,
                3..=4,
                "two idle periods compress independently",
            ),
            // 20ms threshold behavior
            (
                vec![1, 2, 2, 2, 3, 4, 4, 4],
                false,
                Some(20),
                6..=8,
                "20ms threshold: 2 frames per idle period",
            ),
            // Mixed idle lengths with 30ms threshold
            (
                vec![1, 2, 2, 3, 4, 5, 5, 5, 5],
                false,
                Some(30),
                8..=9,
                "mixed idle: 20ms saved, 40ms partial",
            ),
            // Content change resets idle tracking
            (
                vec![1, 2, 2, 3, 4, 4, 4, 5],
                false,
                Some(25),
                6..=8,
                "content change resets idle tracking",
            ),
            // Exact threshold boundary
            (
                vec![1, 2, 2, 2, 3],
                false,
                Some(30),
                5..=6,
                "exact 30ms boundary test",
            ),
            // Timeline compression verification
            (
                vec![1, 2, 2, 2, 2, 3],
                false,
                Some(20),
                4..=4,
                "40ms idle: 20ms saved, rest compressed",
            ),
            // Maximum compression
            (
                vec![1, 2, 2, 2, 2, 3],
                false,
                None,
                2..=3,
                "max compression: only active frames",
            ),
        ]
        .iter()
        .enumerate()
        .try_for_each(|(i, (frame_seq, natural, threshold_ms, expected, desc))| {
            let threshold = threshold_ms.map(Duration::from_millis);
            let timestamps = run_capture_test(frames(frame_seq), *natural, threshold)?;
            let (count, duration, has_gaps) = analyze_timeline(&timestamps);

            // Check frame count matches expectation
            assert!(
                expected.contains(&count),
                "Test {}: expected {:?} frames, got {}",
                i + 1,
                expected,
                count
            );

            // Check timeline compression (no gaps over 25ms between frames)
            if threshold.is_some() && !natural {
                assert!(!has_gaps, "Test {}: timeline has gaps", i + 1);
            }

            // Check aggressive compression for long idle sequences
            if !natural
                && threshold.is_none()
                && frame_seq.windows(2).filter(|w| w[0] == w[1]).count() >= 3
            {
                assert!(
                    duration < 120,
                    "Test {}: duration {} too long",
                    i + 1,
                    duration
                );
            }

            println!("✓ Test {}: {} - {} frames captured", i + 1, desc, count);
            Ok(())
        })
    }
}
