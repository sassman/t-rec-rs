//! Recording helper for examples.
//!
//! This module provides a clean API for recording window animations to GIF and MP4.

use core_animation::Window;
use std::path::Path;
use std::time::Duration;
use t_rec::HeadlessRecorder;

/// Show the window and record the animation to GIF and MP4 files.
///
/// This is useful for generating demo recordings for examples automatically.
/// The output files are saved with the appropriate extensions (`.gif` and `.mp4`).
///
/// Uses sensible defaults for demo recordings:
/// - 15 FPS
/// - No decoration (clean output)
/// - Ventura wallpaper with 60px padding
///
/// # Arguments
///
/// * `window` - The window to record.
/// * `base_path` - The base path for output files (without extension).
///   For example, `"screenshots/my_example"` will create:
///   - `screenshots/my_example.gif`
///   - `screenshots/my_example.mp4`
/// * `duration` - How long to record the animation.
///
/// # Example
///
/// ```ignore
/// use core_animation::prelude::*;
/// mod common;
///
/// common::show_with_recording(
///     &window,
///     "crates/core-animation/examples/screenshots/my_example",
///     5.seconds(),
/// );
/// ```
pub fn show_with_recording(window: &Window, base_path: impl AsRef<Path>, duration: Duration) {
    let base_path = base_path.as_ref();

    // Create parent directories if needed
    if let Some(parent) = base_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("Failed to create directory: {}", e);
            // Fall back to just showing the window
            window.show_for(duration);
            return;
        }
    }

    // Show window and pump event loop to ensure visibility
    window.show();
    for _ in 0..30 {
        window.run_loop_tick();
    }

    // Create recorder with defaults for demo recordings
    let recorder_result = HeadlessRecorder::builder()
        .window_id(window.window_id())
        .with_defaults_for_demo()
        .output_both(base_path)
        .build();

    let mut recorder = match recorder_result {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to create recorder: {}", e);
            eprintln!("Recording requires ImageMagick and ffmpeg to be installed.");
            // Fall back to just showing the window
            let start = std::time::Instant::now();
            while start.elapsed() < duration {
                window.run_loop_tick();
            }
            return;
        }
    };

    // Start recording
    if let Err(e) = recorder.start() {
        eprintln!("Failed to start recording: {}", e);
        let start = std::time::Instant::now();
        while start.elapsed() < duration {
            window.run_loop_tick();
        }
        return;
    }

    // Run the animation for the specified duration
    let start = std::time::Instant::now();
    while start.elapsed() < duration {
        window.run_loop_tick();
    }

    // Stop and generate outputs
    match recorder.stop_and_generate() {
        Ok(output) => {
            println!("Recording complete! Frames: {}", output.frame_count);
            if let Some(ref gif_path) = output.gif_path {
                println!("  GIF: {}", gif_path.display());
            }
            if let Some(ref mp4_path) = output.mp4_path {
                println!("  MP4: {}", mp4_path.display());
            }
        }
        Err(e) => {
            eprintln!("Failed to generate recording: {}", e);
        }
    }
}
