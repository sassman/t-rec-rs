//! Animation builder for configuring keyframe animations.
//!
//! This module provides a fluent API for defining animations with keyframes,
//! easing functions, and repeat behavior.

use std::time::Duration;

use crate::window::OsdWindow;

use super::animated_window::AnimatedWindow;
use super::easing::Easing;
use super::keyframe::{Keyframe, KeyframeBuilder};
use super::runner::AnimationRunner;

/// How many times the animation should repeat.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Repeat {
    /// Loop forever (until display duration ends).
    #[default]
    Infinite,
    /// Play exactly N times, then hold final frame.
    Count(u32),
}

/// Internal animation configuration.
#[derive(Clone, Debug)]
pub struct Animation {
    /// Animation name (for debugging).
    pub name: String,
    /// Duration of one animation cycle.
    pub duration: Duration,
    /// Keyframes sorted by progress.
    pub keyframes: Vec<Keyframe>,
    /// Default easing for transitions.
    pub default_easing: Easing,
    /// Repeat behavior.
    pub repeat: Repeat,
}

/// Builder for configuring animations.
///
/// # Example
///
/// ```ignore
/// .animate("pulse", 2.seconds())
///     .easing(Easing::EaseInOut)  // optional, this is the default
///     .keyframe(0.0, |k| k.scale(0.95))
///     .keyframe(0.7, |k| k.scale(1.0).easing(Easing::EaseOut))
///     .keyframe(1.0, |k| k.scale(0.95))
///     .repeat(5)  // optional, default is infinite
///     .show_for_seconds(10.0)?;
/// ```
pub struct AnimationBuilder<W: OsdWindow> {
    window: AnimatedWindow<W>,
    name: String,
    duration: Duration,
    keyframes: Vec<Keyframe>,
    easing: Easing,
    repeat: Repeat,
}

impl<W: OsdWindow> AnimationBuilder<W> {
    /// Create a new animation builder.
    pub(crate) fn new(window: AnimatedWindow<W>, name: String, duration: Duration) -> Self {
        Self {
            window,
            name,
            duration,
            keyframes: Vec::new(),
            easing: Easing::default(), // EaseInOut
            repeat: Repeat::default(), // Infinite
        }
    }

    /// Set the default easing for all keyframe transitions.
    ///
    /// Individual keyframes can override this with `.easing()` on the
    /// keyframe builder.
    ///
    /// # Default
    ///
    /// `Easing::EaseInOut`
    ///
    /// # Example
    ///
    /// ```ignore
    /// .animate("bounce", 1.seconds())
    ///     .easing(Easing::Linear)
    /// ```
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Define a keyframe at the given progress.
    ///
    /// Keyframes define the animation state at specific points in the cycle.
    /// The animation interpolates between consecutive keyframes.
    ///
    /// # Arguments
    ///
    /// * `progress` - Position in animation cycle (0.0 to 1.0)
    /// * `f` - Builder function to configure the keyframe
    ///
    /// # Example
    ///
    /// ```ignore
    /// .keyframe(0.0, |k| k.scale(0.95))
    /// .keyframe(0.5, |k| k.scale(1.05).circle(40.0, 40.0, 30.0, Color::RED))
    /// .keyframe(1.0, |k| k.scale(0.95))
    /// ```
    pub fn keyframe<F>(mut self, progress: f64, f: F) -> Self
    where
        F: FnOnce(KeyframeBuilder) -> KeyframeBuilder,
    {
        let builder = KeyframeBuilder::new(progress);
        let keyframe = f(builder).build();
        self.keyframes.push(keyframe);
        self
    }

    /// Set finite repeat count.
    ///
    /// By default, animations loop infinitely until the display duration ends.
    /// Use this to play the animation a specific number of times, then hold
    /// the final keyframe state.
    ///
    /// # Example
    ///
    /// ```ignore
    /// .animate("flash", 500.millis())
    ///     .keyframe(0.0, |k| k.scale(1.0))
    ///     .keyframe(0.5, |k| k.scale(1.2))
    ///     .keyframe(1.0, |k| k.scale(1.0))
    ///     .repeat(3)  // Flash 3 times, then stay at scale 1.0
    ///     .show_for_seconds(5.0)?;
    /// ```
    pub fn repeat(mut self, count: u32) -> Self {
        self.repeat = Repeat::Count(count);
        self
    }

    /// Run the animation for the specified duration.
    ///
    /// The animation will loop according to the repeat setting until
    /// the display duration ends.
    ///
    /// # Errors
    ///
    /// Returns an error if no keyframes were defined.
    pub fn show_for_seconds(self, seconds: f64) -> crate::Result<()> {
        self.run_animation(Some(Duration::from_secs_f64(seconds)))
    }

    /// Run the animation for the specified duration.
    ///
    /// This is a convenience method that accepts a `Duration` directly.
    /// Use with the `DurationExt` trait for ergonomic syntax:
    ///
    /// ```ignore
    /// .animate("pulse", 2.seconds())
    ///     .keyframe(...)
    ///     .show(10.seconds())?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if no keyframes were defined.
    pub fn show(self, duration: Duration) -> crate::Result<()> {
        self.run_animation(Some(duration))
    }

    /// Run the animation indefinitely (until window is closed).
    ///
    /// # Errors
    ///
    /// Returns an error if no keyframes were defined.
    pub fn show_indefinitely(self) -> crate::Result<()> {
        self.run_animation(None)
    }

    /// Internal: validate and run the animation.
    fn run_animation(self, total_duration: Option<Duration>) -> crate::Result<()> {
        // Validate: at least one keyframe required
        if self.keyframes.is_empty() {
            return Err(anyhow::anyhow!(
                "Animation '{}' requires at least one keyframe",
                self.name
            ));
        }

        // Sort keyframes by progress
        let mut keyframes = self.keyframes;
        keyframes.sort_by(|a, b| {
            a.progress
                .partial_cmp(&b.progress)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Build animation config
        let animation = Animation {
            name: self.name,
            duration: self.duration,
            keyframes,
            default_easing: self.easing,
            repeat: self.repeat,
        };

        // Create and run the animation loop
        let runner = AnimationRunner::new(self.window, animation);
        runner.run(total_duration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repeat_default() {
        assert_eq!(Repeat::default(), Repeat::Infinite);
    }

    #[test]
    fn test_repeat_count() {
        let r = Repeat::Count(5);
        assert_eq!(r, Repeat::Count(5));
    }
}
