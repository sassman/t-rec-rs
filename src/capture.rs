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

    /// Converts a sequence of numbers into frame data for testing.
    /// 
    /// Each number becomes a 1x1 RGBA pixel where all channels have the same value.
    /// This simulates terminal screenshots where:
    /// - Same numbers = identical frames (idle terminal)
    /// - Different numbers = content changed (terminal activity)
    /// 
    /// Example: frames(&[1,2,2,3]) creates 4 frames where frames 1 and 2 are identical
    fn frames<T: AsRef<[u8]>>(sequence: T) -> Vec<Vec<u8>> {
        sequence.as_ref().iter().map(|&value| vec![value; 4]).collect()
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
        // Frame number explanation:
        // - Each number in arrays like [1,2,2,2,3] represents a pixel value (0-255)
        // - The frames() function converts each number to a 1x1 RGBA pixel where all channels have that value
        // - Identical numbers = identical frames (simulates idle terminal)
        // - Different numbers = content changed (simulates terminal activity)
        // - Example: [1,2,2,2,3] = active frame, 3 idle frames, then active frame
        // - The [..] syntax converts arrays to slices (&[u8]) since we have different array sizes
        //
        // Test data format - each test is a tuple with 5 elements:
        // 1. frames: &[u8] - Array of pixel values representing frame sequence
        // 2. natural mode: bool - If true, saves all frames (no compression)
        // 3. threshold ms: Option<u64> - Idle duration to preserve before compressing
        //    - None = maximum compression (skip all identical frames)
        //    - Some(ms) = preserve idle frames up to ms, then compress
        // 4. expected frames: RangeInclusive - Expected frame count range (handles timing variations)
        // 5. description: &str - Human-readable explanation of what this test verifies
        [
            // Natural mode - saves all frames regardless of content
            (&[1,1,1][..],          true,  None,     3..=4, "natural mode preserves all frames"),
            
            // Basic single frame test
            (&[1][..],              false, None,     1..=2, "single frame recording"),
            
            // All different frames - no idle to compress
            (&[1,2,3][..],          false, None,     3..=3, "all different frames saved"),
            
            // Basic idle compression
            (&[1,1,1][..],          false, None,     1..=1, "3 identical frames → 1 frame"),
            
            // Long threshold preserves short sequences
            (&[1,1,1][..],          false, Some(500), 3..=4, "500ms threshold preserves 30ms idle"),
            
            // Multiple idle periods compress independently
            (&[1,2,2,2,3,4,4,4][..], false, None,     3..=4, "two idle periods compress independently"),
            
            // 20ms threshold behavior
            (&[1,2,2,2,3,4,4,4][..], false, Some(20), 6..=8, "20ms threshold: 2 frames per idle period"),
            
            // Mixed idle lengths with 30ms threshold
            (&[1,2,2,3,4,5,5,5,5][..], false, Some(30), 8..=9, "mixed idle: 20ms saved, 40ms partial"),
            
            // Content change resets idle tracking
            (&[1,2,2,3,4,4,4,5][..], false, Some(25), 6..=8, "content change resets idle tracking"),
            
            // Exact threshold boundary
            (&[1,2,2,2,3][..],      false, Some(30), 5..=6, "exact 30ms boundary test"),
            
            // Timeline compression verification
            (&[1,2,2,2,2,3][..],    false, Some(20), 4..=4, "40ms idle: 20ms saved, rest compressed"),
            
            // Maximum compression
            (&[1,2,2,2,2,3][..],    false, None,     2..=3, "max compression: only active frames"),
        ]
        .iter()
        .enumerate()
        .try_for_each(|(i, (frame_seq, natural, threshold_ms, expected, desc))| {
            let threshold = threshold_ms.map(Duration::from_millis);
            let timestamps = run_capture_test(frames(frame_seq), *natural, threshold)?;
            let (count, duration, has_gaps) = analyze_timeline(&timestamps);
            
            // Check frame count matches expectation
            assert!(expected.contains(&count), 
                "Test {}: expected {:?} frames, got {}", i+1, expected, count);
            
            // Check timeline compression (no large gaps between frames)
            if threshold.is_some() && !natural {
                assert!(!has_gaps, "Test {}: timeline has gaps", i+1);
            }
            
            // Check aggressive compression for long idle sequences
            if !natural && threshold.is_none() && frame_seq.windows(2).filter(|w| w[0] == w[1]).count() >= 3 {
                assert!(duration < 120, "Test {}: duration {} too long", i+1, duration);
            }
            
            println!("✓ Test {}: {} - {} frames captured", i+1, desc, count);
            Ok(())
        })
    }


}
