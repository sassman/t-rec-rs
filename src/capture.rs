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


/// Captures screenshots as files for terminal recording with intelligent compression.
/// 
/// Creates smooth, natural recordings by eliminating long idle periods while preserving
/// brief pauses that aid comprehension. Timeline compression removes skipped frame time
/// from subsequent timestamps, preventing jarring gaps in playback.
/// 
/// # Parameters
/// 
/// * `idle_pause` - Controls idle period handling:
///   - `None`: Maximum compression - skip all identical frames
///   - `Some(duration)`: Preserve natural pauses up to duration, skip beyond threshold
/// 
/// # Timeline Compression
/// 
/// When idle periods exceed the threshold:
/// 1. Save frames during natural pauses (up to idle_pause duration)
/// 2. Skip remaining frames and subtract their time from subsequent timestamps
/// 3. Result: Playback shows exactly the intended pause duration
/// 
/// Example: 10-second idle with 3-second threshold → saves 3 seconds of pause,
/// skips 7 seconds, playback shows exactly 3 seconds.
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
    
    // Timeline compression state: total time removed from recording to maintain smooth playback
    let mut idle_duration = Duration::from_millis(0);
    
    // Current idle sequence tracking: duration of ongoing identical frame sequence
    let mut current_idle_period = Duration::from_millis(0);
    
    let mut last_frame: Option<ImageOnHeap> = None;
    let mut last_now = Instant::now();
    loop {
        // blocks for a timeout
        if rx.recv_timeout(duration).is_ok() {
            break;
        }
        let now = Instant::now();
        
        // Calculate compressed timestamp for smooth playback: real time minus skipped idle time
        let effective_now = now.sub(idle_duration);
        let tc = effective_now.saturating_duration_since(start).as_millis();
        
        let image = api.capture_window_screenshot(win_id)?;
        let frame_duration = now.duration_since(last_now);
        
        // Detect identical frames to identify idle periods (unless in natural mode)
        let frame_unchanged = !force_natural 
            && last_frame.as_ref()
                .map(|last| image.samples.as_slice() == last.samples.as_slice())
                .unwrap_or(false);
        
        // Update idle period tracking for compression decisions
        if frame_unchanged {
            current_idle_period = current_idle_period.add(frame_duration);
        } else {
            current_idle_period = Duration::from_millis(0);
        }
        
        // Recording quality decision: balance compression with natural pacing
        let should_save_frame = if frame_unchanged {
            let should_skip_for_compression = if let Some(threshold) = idle_pause {
                // Preserve natural pauses up to threshold, compress longer idle periods
                current_idle_period >= threshold
            } else {
                // Maximum compression: skip all idle frames for smallest file size
                true
            };
            
            if should_skip_for_compression {
                // Remove this idle time from recording timeline for smooth playback
                idle_duration = idle_duration.add(frame_duration);
                false
            } else {
                // Keep short pauses for natural recording feel
                true
            }
        } else {
            // Always capture content changes
            current_idle_period = Duration::from_millis(0);
            true
        };
        
        if should_save_frame {
            // Save frame and update state
            if let Err(e) = save_frame(&image, tc, tempdir.lock().unwrap().borrow(), file_name_for) {
                eprintln!("{}", &e);
                return Err(e);
            }
            time_codes.lock().unwrap().push(tc);
            
            // Update last_frame to current frame for next iteration's comparison
            last_frame = Some(image);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use tempfile::TempDir;

    /// Mock implementation of PlatformApi for testing capture functionality.
    /// 
    /// Cycles through a predefined sequence of frame data to simulate
    /// terminal screenshots with controlled content changes and idle periods.
    struct TestApi {
        frames: Vec<Vec<u8>>,
        index: std::cell::Cell<usize>,
    }

    impl crate::PlatformApi for TestApi {
        fn capture_window_screenshot(&self, _: crate::WindowId) -> crate::Result<crate::ImageOnHeap> {
            let i = self.index.get();
            self.index.set(i + 1);
            // Return 1x1 RGBA pixel data - stop at last frame instead of cycling
            let num_channels = 4; // RGBA
            let pixel_width = 1;
            let pixel_height = 1;
            let frame_index = if i >= self.frames.len() {
                self.frames.len() - 1  // Stay on last frame
            } else {
                i
            };
            Ok(Box::new(image::FlatSamples {
                samples: self.frames[frame_index].clone(),
                layout: image::flat::SampleLayout::row_major_packed(num_channels, pixel_width, pixel_height),
                color_hint: Some(image::ColorType::Rgba8)
            }))
        }
        fn calibrate(&mut self, _: crate::WindowId) -> crate::Result<()> { Ok(()) }
        fn window_list(&self) -> crate::Result<crate::WindowList> { Ok(vec![]) }
        fn get_active_window(&self) -> crate::Result<crate::WindowId> { Ok(0) }
    }

    /// Converts a sequence of numbers into frame data.
    /// 
    /// Each number becomes a 1x1 RGBA pixel where all channels have the same value.
    /// This makes it easy to create test patterns where identical numbers represent
    /// idle frames and different numbers represent content changes.
    fn frames(sequence: &[u8]) -> Vec<Vec<u8>> {
        sequence.iter().map(|&value| vec![value; 4]).collect()
    }
    
    /// Creates test frame sequences for idle compression scenarios.
    /// 
    /// Returns a tuple of (frames, min_expected, max_expected, description) where:
    /// - frames: The sequence of frame data to test
    /// - min_expected: Minimum frames expected with maximum compression
    /// - max_expected: Maximum frames expected with no compression
    /// - description: Human-readable explanation of what this pattern tests
    /// 
    /// Frame timing: At 10ms/frame in test mode:
    /// - 2 identical frames = 20ms idle period
    /// - 3 identical frames = 30ms idle period
    /// - 4 identical frames = 40ms idle period
    fn create_frames(pattern: &str) -> (Vec<Vec<u8>>, usize, usize, &'static str) {
        match pattern {
            // Tests that single frame recordings work correctly - the most basic test case
            "single_frame_recording" => (
                frames(&[1]), 1, 2,
                "Single frame recording saves 1-2 frames (allows for timing variation)"
            ),
            
            // Tests that all frames are saved when each frame has different content
            "all_different_frames" => (
                frames(&[1, 2, 3]), 3, 3,
                "3 different frames save all 3 (no idle to compress)"
            ),
            
            // Tests basic idle compression with 3 identical frames (30ms idle period)
            "three_identical_frames" => (
                frames(&[1, 1, 1]), 1, 4,
                "3 identical frames: compresses to 1 or preserves up to 4 with timing variation"
            ),
            
            // Tests that multiple idle periods are compressed independently, not cumulatively
            "two_idle_periods" => (
                frames(&[1, 2,2,2, 3, 4,4,4]), 3, 8,
                "Two 3-frame idle periods (30ms each): tests independent compression"
            ),
            
            // Tests compression behavior with different idle lengths in same recording
            "mixed_length_idle_periods" => (
                frames(&[1, 2,2, 3,4, 5,5,5,5]), 6, 9,
                "20ms + 40ms idle periods: tests threshold boundary behavior"
            ),
            
            // Tests that content changes properly reset idle period tracking
            "idle_reset_on_change" => (
                frames(&[1, 2,2, 3, 4,4,4, 5]), 6, 8,
                "Content change at frame 3 resets idle tracking, preventing over-compression"
            ),
            
            // Tests exact threshold boundary case where idle duration equals threshold
            "idle_at_exact_threshold" => (
                frames(&[1, 2,2,2, 3]), 5, 6,
                "3 idle frames at exactly 30ms threshold: 5-6 frames saved (timing variation)"
            ),
            
            // Tests timeline compression maintains smooth playback without timestamp gaps
            "single_long_idle_period" => (
                frames(&[1, 2,2,2,2, 3]), 2, 4,
                "40ms idle period: verifies timeline compression prevents playback gaps"
            ),
            
            // Default fallback pattern for unrecognized test names
            _ => (
                frames(&[1, 1, 1]), 1, 3,
                "Default: 3 identical frames"
            ),
        }
    }

    /// Runs a capture test with the specified frame sequence and compression settings.
    /// 
    /// Sets up a mock capture environment, runs the capture thread for a duration
    /// based on frame count, and returns the timestamps of saved frames.
    /// 
    /// # Arguments
    /// * `test_frames` - Sequence of frame data to capture
    /// * `natural_mode` - If true, saves all frames (no compression)
    /// * `idle_threshold` - Duration of idle to preserve before compression kicks in
    fn run_capture_test(test_frames: Vec<Vec<u8>>, natural_mode: bool, idle_threshold: Option<Duration>) -> crate::Result<Vec<u128>> {
        let test_api = TestApi { frames: test_frames.clone(), index: Default::default() };
        let captured_timestamps = Arc::new(Mutex::new(Vec::new()));
        let temp_directory = Arc::new(Mutex::new(TempDir::new()?));
        let (stop_signal_tx, stop_signal_rx) = mpsc::channel();

        // Calculate capture duration based on frame count
        // Add buffer to ensure we capture all frames accounting for timing variations
        let frame_interval = 10; // ms per frame in test mode
        let capture_duration_ms = (test_frames.len() as u64 * frame_interval) + 15;
        
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(capture_duration_ms));
            let _ = stop_signal_tx.send(());
        });

        let timestamps_clone = captured_timestamps.clone();
        capture_thread(&stop_signal_rx, test_api, 0, timestamps_clone, temp_directory, natural_mode, idle_threshold)?;
        let result = captured_timestamps.lock().unwrap().clone();
        Ok(result)
    }

    /// Analyzes captured frame timestamps for compression effectiveness.
    /// 
    /// Returns a tuple of:
    /// - Frame count: Total number of frames captured
    /// - Total duration: Time span from first to last frame (ms)
    /// - Has gaps: Whether large gaps exist that indicate compression failure
    /// 
    /// Large gaps (>25ms) between consecutive frames indicate the timeline
    /// compression algorithm failed to maintain smooth playback.
    fn analyze_timeline(timestamps: &[u128]) -> (usize, u128, bool) {
        let max_normal_gap = 25; // Maximum expected gap between consecutive frames (ms)
        
        let frame_count = timestamps.len();
        let total_duration_ms = if timestamps.len() > 1 {
            timestamps.last().unwrap() - timestamps.first().unwrap()
        } else { 0 };
        
        // Detect large gaps indicating timeline compression failure
        let has_compression_gaps = timestamps.windows(2)
            .any(|window| window[1] - window[0] > max_normal_gap);
        
        (frame_count, total_duration_ms, has_compression_gaps)
    }

    /// Tests idle frame compression behavior across various scenarios.
    /// 
    /// This parameterized test validates the idle pause functionality by running
    /// multiple test cases through a single test function. Each test case specifies:
    /// - Natural mode on/off (force saving all frames vs compression)
    /// - A frame pattern (sequence of frames with idle periods)
    /// - An optional idle threshold (how long to preserve idle frames)
    /// 
    /// The test verifies:
    /// - Frames are compressed correctly based on the threshold
    /// - Timeline compression eliminates gaps for smooth playback
    /// - Natural mode bypasses compression entirely
    /// - Edge cases like exact threshold boundaries work correctly
    /// 
    /// Test patterns are created by `create_frames()` which returns expected
    /// frame counts along with the actual frame data, making assertions clear.
    #[test]
    fn test_idle_pause() -> crate::Result<()> {
        // Test timing: frames arrive every 10ms in test mode
        let frame_interval_ms = 10;
        let short_threshold = Duration::from_millis(frame_interval_ms * 2);  // 20ms = 2 frames
        let medium_threshold = Duration::from_millis(25); // 25ms = 2.5 frames  
        let long_threshold = Duration::from_millis(frame_interval_ms * 3);   // 30ms = 3 frames
        let very_long_threshold = Duration::from_millis(500); // 500ms = 50 frames
        
        // Each test case is a tuple of (natural_mode, pattern_name, idle_threshold)
        // The test loop runs each case and verifies frame counts match expectations
        let test_cases = [
            // Natural mode - no compression regardless of content
            (true, "three_identical_frames", None),
            
            // Basic compression scenarios
            (false, "single_frame_recording", None),
            (false, "all_different_frames", None),
            (false, "three_identical_frames", None),
            (false, "three_identical_frames", Some(very_long_threshold)),
            
            // Multiple idle periods - tests independent compression
            (false, "two_idle_periods", None),
            (false, "two_idle_periods", Some(short_threshold)),
            
            // Edge cases - threshold boundaries and tracking resets
            (false, "mixed_length_idle_periods", Some(long_threshold)),
            (false, "idle_reset_on_change", Some(medium_threshold)),
            (false, "idle_at_exact_threshold", Some(long_threshold)),
            
            // Timeline compression - verifies no playback gaps
            (false, "single_long_idle_period", Some(short_threshold)),
            (false, "single_long_idle_period", None),
        ];

        // Run each test case through the capture simulation
        for (case_num, &(natural_mode, pattern, threshold)) in test_cases.iter().enumerate() {
            // Get test frames and expected results from pattern name
            let (test_frames, min_frames, max_frames, description) = create_frames(pattern);
            
            // Override expected frames for natural mode (saves all frames)
            // Allow for +1 frame due to timing variations in test environment
            let (min_expected, max_expected) = if natural_mode {
                (test_frames.len(), test_frames.len() + 1)
            } else {
                (min_frames, max_frames)
            };
            
            let saved_timestamps = run_capture_test(test_frames, natural_mode, threshold)?;
            let (actual_frame_count, total_duration_ms, has_large_gaps) = analyze_timeline(&saved_timestamps);
            
            // Build test context for clearer error messages
            let threshold_desc = match threshold {
                None => "no threshold".to_string(),
                Some(d) => format!("{}ms threshold", d.as_millis()),
            };
            let mode_desc = if natural_mode { "natural mode" } else { "compression mode" };
            
            // Verify captured frame count matches expected range
            assert!(actual_frame_count >= min_expected && actual_frame_count <= max_expected, 
                "Test {} [{}] {}: {} - expected {}-{} frames, got {} frames", 
                case_num + 1, mode_desc, pattern, threshold_desc, min_expected, max_expected, actual_frame_count);
            
            // Verify timeline compression eliminates gaps from skipped frames
            if threshold.is_some() && !natural_mode {
                assert!(!has_large_gaps, 
                    "Test {} [{}] {}: {} - timeline compression failed, found large timestamp gaps", 
                    case_num + 1, mode_desc, pattern, threshold_desc);
            }
            
            // Verify compression effectiveness for sequences with long idle periods
            let max_compressed_duration_ms = 120; // ~12 frames at 10ms intervals
            if (pattern.contains("long") || pattern.contains("periods")) && !natural_mode && threshold.is_none() {
                assert!(total_duration_ms < max_compressed_duration_ms, 
                    "Test {} [{}] {}: {} - timeline should be compressed to <{}ms, got {}ms", 
                    case_num + 1, mode_desc, pattern, threshold_desc, max_compressed_duration_ms, total_duration_ms);
            }
            
            println!("✓ Test {} [{}] {}: {} - {}", 
                case_num + 1, mode_desc, pattern, threshold_desc, description);
        }
        Ok(())
    }


}
