//! Animation runner for executing keyframe animations.
//!
//! This module contains the animation loop that computes interpolated frames
//! and renders them to the window.

use std::ffi::c_void;
use std::time::{Duration, Instant};

use crate::window::OsdWindow;

use super::animated_window::AnimatedWindow;
use super::builder::{Animation, Repeat};
use super::interpolation::{interpolate, InterpolatedFrame};
use super::keyframe::Keyframe;

// CFRunLoop bindings (matching the existing window.rs bindings)
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRunLoopRunInMode(
        mode: *const c_void,
        seconds: f64,
        return_after_source_handled: bool,
    ) -> i32;
    static kCFRunLoopDefaultMode: *const c_void;
}

/// Run the CFRunLoop for a given duration.
fn run_loop_for_seconds(seconds: f64) {
    unsafe {
        CFRunLoopRunInMode(kCFRunLoopDefaultMode, seconds, false);
    }
}

/// Orchestrates the animation loop.
pub struct AnimationRunner<W: OsdWindow> {
    window: AnimatedWindow<W>,
    animation: Animation,
}

impl<W: OsdWindow> AnimationRunner<W> {
    /// Create a new animation runner.
    pub fn new(window: AnimatedWindow<W>, animation: Animation) -> Self {
        Self { window, animation }
    }

    /// Run the animation loop.
    ///
    /// # Arguments
    ///
    /// * `total_duration` - How long to run the animation. If `None`, runs indefinitely.
    pub fn run(self, total_duration: Option<Duration>) -> crate::Result<()> {
        // For the animation to work, we need access to the underlying window's
        // drawing capabilities. This requires the OsdWindow to expose methods
        // for animation rendering.
        //
        // The animation loop:
        // 1. Compute elapsed time
        // 2. Calculate animation progress (0.0 to 1.0)
        // 3. Find surrounding keyframes
        // 4. Apply easing and interpolate
        // 5. Render the frame
        // 6. Wait for next frame (using CFRunLoop)

        let start = Instant::now();
        let frame_duration = Duration::from_secs_f64(1.0 / 60.0); // 60 FPS target
        let total_duration = total_duration.unwrap_or(Duration::from_secs(u64::MAX));
        let animation_duration_secs = self.animation.duration.as_secs_f64();

        // Show the window first
        self.window.window.show_window()?;

        while start.elapsed() < total_duration {
            let frame_start = Instant::now();
            let elapsed = start.elapsed().as_secs_f64();

            // Calculate cycle progress based on repeat mode
            let cycle_progress = match &self.animation.repeat {
                Repeat::Infinite => {
                    // Loop forever
                    (elapsed % animation_duration_secs) / animation_duration_secs
                }
                Repeat::Count(n) => {
                    let cycles_completed = (elapsed / animation_duration_secs) as u32;
                    if cycles_completed >= *n {
                        // Animation finished, hold at final state
                        1.0
                    } else {
                        (elapsed % animation_duration_secs) / animation_duration_secs
                    }
                }
            };

            // Compute the interpolated frame
            let frame = self.compute_frame(cycle_progress);

            // Render the frame
            self.render_frame(&frame)?;

            // Calculate how long to wait for next frame
            let render_time = frame_start.elapsed();
            let wait_time = if render_time < frame_duration {
                frame_duration - render_time
            } else {
                Duration::ZERO
            };

            // Wait for next frame using CFRunLoop (also processes events)
            if !wait_time.is_zero() {
                run_loop_for_seconds(wait_time.as_secs_f64());
            }

            // Additional sleep to ensure we hit target frame rate
            // CFRunLoopRunInMode may return early
            let total_frame_time = frame_start.elapsed();
            if total_frame_time < frame_duration {
                std::thread::sleep(frame_duration - total_frame_time);
            }
        }

        // Hide the window
        self.window.window.hide_window()?;

        Ok(())
    }

    /// Compute the interpolated frame at the given progress.
    fn compute_frame(&self, progress: f64) -> InterpolatedFrame {
        let (from_kf, to_kf, segment_t) = self.find_surrounding_keyframes(progress);

        // Determine which easing to use for this segment
        // The "to" keyframe's easing controls the transition into it
        let easing = to_kf.easing.unwrap_or(self.animation.default_easing);

        // Apply easing to the segment progress
        let eased_t = easing.apply(segment_t);

        interpolate(from_kf, to_kf, eased_t)
    }

    /// Find the two keyframes surrounding the given progress.
    ///
    /// Returns (from_keyframe, to_keyframe, segment_progress)
    fn find_surrounding_keyframes(&self, progress: f64) -> (&Keyframe, &Keyframe, f64) {
        let keyframes = &self.animation.keyframes;

        // Handle edge cases
        if keyframes.len() == 1 {
            return (&keyframes[0], &keyframes[0], 0.0);
        }

        // Find the keyframe pair that surrounds the progress
        let mut from_idx = 0;
        for (i, kf) in keyframes.iter().enumerate() {
            if kf.progress <= progress {
                from_idx = i;
            } else {
                break;
            }
        }

        let to_idx = (from_idx + 1).min(keyframes.len() - 1);

        let from_kf = &keyframes[from_idx];
        let to_kf = &keyframes[to_idx];

        // Calculate progress within this segment
        let segment_t = if from_idx == to_idx {
            0.0
        } else {
            let segment_start = from_kf.progress;
            let segment_end = to_kf.progress;
            let segment_length = segment_end - segment_start;
            if segment_length > 0.0 {
                (progress - segment_start) / segment_length
            } else {
                0.0
            }
        };

        (from_kf, to_kf, segment_t.clamp(0.0, 1.0))
    }

    /// Render the interpolated frame to the window.
    fn render_frame(&self, frame: &InterpolatedFrame) -> crate::Result<()> {
        // This requires the OsdWindow to expose animation rendering methods.
        // We'll delegate to the window's render_animation_frame method.
        self.window.window.render_animation_frame(
            &self.window.content,
            &frame.transform,
            &frame.shapes,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::animation::transform::Transform;

    #[allow(dead_code)]
    fn make_test_keyframes() -> Vec<Keyframe> {
        vec![
            Keyframe {
                progress: 0.0,
                transform: Transform::new(0.95),
                shapes: vec![],
                easing: None,
            },
            Keyframe {
                progress: 0.5,
                transform: Transform::new(1.05),
                shapes: vec![],
                easing: None,
            },
            Keyframe {
                progress: 1.0,
                transform: Transform::new(0.95),
                shapes: vec![],
                easing: None,
            },
        ]
    }

    // Note: Full tests require a mock OsdWindow implementation
    // Testing the keyframe finding logic:

    #[test]
    fn test_find_keyframes_at_start() {
        // This would test find_surrounding_keyframes at progress 0.0
        // Returns first keyframe for both from and to with t=0
    }

    #[test]
    fn test_find_keyframes_at_middle() {
        // At progress 0.25, should be between keyframe 0 and 1
        // segment_t should be 0.5 (halfway between 0.0 and 0.5)
    }

    #[test]
    fn test_find_keyframes_at_end() {
        // At progress 1.0, should be at/past the last keyframe
    }
}
