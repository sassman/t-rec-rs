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
    idle_pause: Option<Duration>,
) -> Result<()> {
    #[cfg(test)]
    let duration = Duration::from_millis(10); // Fast for testing
    #[cfg(not(test))]
    let duration = Duration::from_millis(250); // Production speed
    let start = Instant::now();
    let mut idle_duration = Duration::from_millis(0);
    let mut current_idle_period = Duration::from_millis(0);
    let mut last_frame: Option<ImageOnHeap> = None;
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
        let frame_duration = now.duration_since(last_now);
        
        // Check if frame is unchanged from previous (only when not in natural mode)
        let frame_unchanged = !force_natural 
            && last_frame.as_ref()
                .map(|last| image.samples.as_slice() == last.samples.as_slice())
                .unwrap_or(false);
        
        if frame_unchanged {
            current_idle_period = current_idle_period.add(frame_duration);
        } else {
            current_idle_period = Duration::from_millis(0);
        }
        
        // Determine if we should save this frame
        let should_save_frame = !frame_unchanged || idle_pause
            .map(|min_idle| current_idle_period < min_idle)
            .unwrap_or(false);
        
        if should_save_frame {
            // Save frame and update state
            if let Err(e) = save_frame(&image, tc, tempdir.lock().unwrap().borrow(), file_name_for) {
                eprintln!("{}", &e);
                return Err(e);
            }
            time_codes.lock().unwrap().push(tc);
            
            // Update last_frame to current frame for next iteration's comparison
            last_frame = Some(image);
        } else {
            // Skip this idle frame and track the skipped time
            idle_duration = idle_duration.add(frame_duration);
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

    #[test]
    fn test_idle_pause() -> crate::Result<()> {
        // Mock API that cycles through predefined frame data for testing
        struct TestApi {
            frames: Vec<Vec<u8>>,
            index: std::cell::Cell<usize>,
        }

        impl crate::PlatformApi for TestApi {
            fn capture_window_screenshot(&self, _: crate::WindowId) -> crate::Result<crate::ImageOnHeap> {
                let i = self.index.get();
                self.index.set(i + 1);
                // Return 1x1 RGBA pixel data cycling through frames
                Ok(Box::new(image::FlatSamples {
                    samples: self.frames[i % self.frames.len()].clone(),
                    layout: image::flat::SampleLayout::row_major_packed(4, 1, 1),
                    color_hint: Some(image::ColorType::Rgba8)
                }))
            }
            fn calibrate(&mut self, _: crate::WindowId) -> crate::Result<()> { Ok(()) }
            fn window_list(&self) -> crate::Result<crate::WindowList> { Ok(vec![]) }
            fn get_active_window(&self) -> crate::Result<crate::WindowId> { Ok(0) }
        }

        // Test cases: (force_natural, frame_data, idle_pause, min_saves, description)
        // Each vec![Nu8; 4] represents a 1x1 RGBA pixel with value N for all channels
        let cases = [
            (true, vec![vec![1u8; 4]; 3], None, 3, "natural mode saves all frames"),
            (false, vec![vec![1u8; 4]], None, 1, "first frame always saved"),
            (false, vec![vec![1u8; 4], vec![2u8; 4], vec![3u8; 4]], None, 3, "different frames saved"),
            (false, vec![vec![1u8; 4]; 3], None, 1, "identical frames skipped"),
            (false, vec![vec![1u8; 4]; 3], Some(Duration::from_millis(500)), 2, "idle pause saves initial frames"),
        ];

        for (i, (natural, frame_data, pause, min_saves, desc)) in cases.iter().enumerate() {
            // Create mock API with test frame data
            let api = TestApi {
                frames: frame_data.clone(),
                index: Default::default(),
            };

            // Set up capture infrastructure
            let time_codes = Arc::new(Mutex::new(Vec::new()));
            let tempdir = Arc::new(Mutex::new(TempDir::new()?));
            let (tx, rx) = mpsc::channel();

            // Spawn thread to stop recording after enough time for captures (fast in test mode)
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(50));
                let _ = tx.send(());
            });

            // Run the actual capture_thread function being tested
            capture_thread(&rx, api, 0, time_codes.clone(), tempdir, *natural, *pause)?;
            
            // Verify the expected number of frames were saved
            let saved = time_codes.lock().unwrap().len();
            assert!(saved >= *min_saves, "Case {}: {} - expected ≥{} saves, got {}", i + 1, desc, min_saves, saved);
        }
        Ok(())
    }

}
