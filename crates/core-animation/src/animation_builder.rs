//! GPU-accelerated animations using `CABasicAnimation`.
//!
//! Animations run on the compositor thread, not the main thread.
//! No manual loop required - just configure and show.
//!
//! # Quick Start
//!
//! ```ignore
//! let layer = CAShapeLayerBuilder::new()
//!     .path(circle_path)
//!     .fill_color(Color::RED)
//!     .animate("pulse", KeyPath::TransformScale, |a| {
//!         a.values(0.85, 1.15)
//!             .duration(800.millis())
//!             .easing(Easing::InOut)
//!             .autoreverses()
//!             .repeat(Repeat::Forever)
//!     })
//!     .build();
//!
//! window.container().add_sublayer(&layer);
//! window.show_for(10.seconds());  // No animation loop needed
//! ```
//!
//! # Builder API
//!
//! ## `CABasicAnimationBuilder`
//!
//! Configures a from→to animation on a single property.
//!
//! | Method | Description |
//! |--------|-------------|
//! | `.values(from, to)` | Numeric from/to values (f64) |
//! | `.values_point(from, to)` | CGPoint from/to |
//! | `.values_color(from, to)` | Color from/to |
//! | `.duration(Duration)` | Animation cycle duration |
//! | `.easing(Easing)` | Timing curve (default: `InOut`) |
//! | `.autoreverses()` | Ping-pong animation |
//! | `.repeat(Repeat)` | Repeat behavior (default: `Once`) |
//! | `.phase_offset(f64)` | Start at fraction of cycle (0.0-1.0) |
//!
//! ## `KeyPath`
//!
//! Property to animate. Common paths:
//!
//! | Constant | Animates |
//! |----------|----------|
//! | `TransformScale` | Uniform scale (0.0 = invisible, 1.0 = normal) |
//! | `TransformScaleX` | Horizontal scale |
//! | `TransformScaleY` | Vertical scale |
//! | `TransformRotation` | Z-axis rotation (radians) |
//! | `Opacity` | Alpha (0.0 = transparent, 1.0 = opaque) |
//! | `Position` | Layer center point (CGPoint) |
//! | `PositionX` | Horizontal position |
//! | `PositionY` | Vertical position |
//! | `BackgroundColor` | Fill color (CGColor) |
//! | `CornerRadius` | Corner rounding |
//! | `BorderWidth` | Border thickness |
//!
//! ## `Easing`
//!
//! Timing curves for animation interpolation:
//!
//! | Variant | Behavior |
//! |---------|----------|
//! | `Linear` | Constant speed |
//! | `In` | Slow start, fast end |
//! | `Out` | Fast start, slow end |
//! | `InOut` | Slow start and end (default) |
//!
//! ## `Repeat`
//!
//! How many times the animation plays:
//!
//! | Variant | Behavior |
//! |---------|----------|
//! | `Once` | Play once, hold final value (default) |
//! | `Times(n)` | Play n times, hold final value |
//! | `Forever` | Loop indefinitely |
//!
//! # Default Behaviors
//!
//! **Values persist after animation ends.** Unlike raw `CABasicAnimation`,
//! this builder defaults to `fillMode = forwards` and `removedOnCompletion = false`.
//! The layer stays at the final animated value.
//!
//! To opt-in to snap-back behavior:
//!
//! ```ignore
//! a.values(0.0, 1.0)
//!     .remove_on_completion()  // Snap back to original value
//! ```
//!
//! # Integration with Layer Builders
//!
//! The `.animate()` method is available on:
//! - `CALayerBuilder`
//! - `CAShapeLayerBuilder`
//!
//! Multiple animations can be added to a single layer:
//!
//! ```ignore
//! CAShapeLayerBuilder::new()
//!     .fill_color(Color::RED)
//!     .animate("pulse", KeyPath::TransformScale, |a| {
//!         a.values(0.9, 1.1).duration(500.millis()).repeat(Repeat::Forever)
//!     })
//!     .animate("fade", KeyPath::Opacity, |a| {
//!         a.values(1.0, 0.7).duration(1.seconds()).repeat(Repeat::Forever)
//!     })
//!     .build()
//! ```
//!
//! # Standalone Usage
//!
//! Animations can also be created and added manually:
//!
//! ```ignore
//! let anim = CABasicAnimationBuilder::new(KeyPath::TransformScale)
//!     .values(0.85, 1.15)
//!     .duration(800.millis())
//!     .autoreverses()
//!     .repeat(Repeat::Forever)
//!     .build();
//!
//! layer.add_animation(&anim, "pulse");
//! ```
//!
//! # Future: Other Animation Types
//!
//! The closure pattern allows different builders for different animation types:
//!
//! ```ignore
//! // Keyframe animation (multiple values)
//! .animate_keyframes("bounce", KeyPath::PositionY, |a| {
//!     a.values([100.0, 50.0, 80.0, 60.0, 70.0])
//!         .key_times([0.0, 0.3, 0.5, 0.7, 1.0])
//! })
//!
//! // Spring animation (physics-based)
//! .animate_spring("snap", KeyPath::Position, |a| {
//!     a.to_point(200.0, 200.0)
//!         .damping(10.0)
//!         .stiffness(100.0)
//! })
//! ```
//!
//! The user-facing API pattern remains consistent; only the builder inside
//! the closure changes.

use std::time::Duration;

use objc2::rc::Retained;
use objc2_foundation::{NSNumber, NSString};
use objc2_quartz_core::{
    kCAFillModeForwards, kCAMediaTimingFunctionEaseIn, kCAMediaTimingFunctionEaseInEaseOut,
    kCAMediaTimingFunctionEaseOut, kCAMediaTimingFunctionLinear, CABasicAnimation, CAMediaTiming,
    CAMediaTimingFunction,
};

/// Property key path for animation targets.
///
/// Each variant maps to a Core Animation key path string that identifies
/// which property of a `CALayer` to animate.
///
/// # Examples
///
/// ```ignore
/// // Animate scale
/// let anim = CABasicAnimationBuilder::new(KeyPath::TransformScale)
///     .values(0.5, 1.0)
///     .build();
///
/// // Animate position
/// let anim = CABasicAnimationBuilder::new(KeyPath::PositionX)
///     .values(0.0, 100.0)
///     .build();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyPath {
    /// Uniform scale transform (`transform.scale`).
    /// Value range: 0.0 = invisible, 1.0 = normal size.
    TransformScale,
    /// Horizontal scale transform (`transform.scale.x`).
    TransformScaleX,
    /// Vertical scale transform (`transform.scale.y`).
    TransformScaleY,
    /// Z-axis rotation transform (`transform.rotation.z`).
    /// Value is in radians.
    TransformRotation,
    /// Layer opacity (`opacity`).
    /// Value range: 0.0 = transparent, 1.0 = opaque.
    Opacity,
    /// Layer center position (`position`).
    /// Value type: CGPoint.
    Position,
    /// Horizontal position (`position.x`).
    PositionX,
    /// Vertical position (`position.y`).
    PositionY,
    /// Layer background color (`backgroundColor`).
    /// Value type: CGColor.
    BackgroundColor,
    /// Corner radius (`cornerRadius`).
    CornerRadius,
    /// Border width (`borderWidth`).
    BorderWidth,
    /// Border color (`borderColor`).
    /// Value type: CGColor.
    BorderColor,
    /// Shadow opacity (`shadowOpacity`).
    ShadowOpacity,
    /// Shadow radius (`shadowRadius`).
    ShadowRadius,
    /// Shadow offset (`shadowOffset`).
    /// Value type: CGSize.
    ShadowOffset,
    /// Bounds rectangle (`bounds`).
    /// Value type: CGRect.
    Bounds,
    /// Custom key path string.
    Custom(&'static str),
}

impl KeyPath {
    /// Returns the Core Animation key path string for this property.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// assert_eq!(KeyPath::TransformScale.as_str(), "transform.scale");
    /// assert_eq!(KeyPath::Opacity.as_str(), "opacity");
    /// ```
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            KeyPath::TransformScale => "transform.scale",
            KeyPath::TransformScaleX => "transform.scale.x",
            KeyPath::TransformScaleY => "transform.scale.y",
            KeyPath::TransformRotation => "transform.rotation.z",
            KeyPath::Opacity => "opacity",
            KeyPath::Position => "position",
            KeyPath::PositionX => "position.x",
            KeyPath::PositionY => "position.y",
            KeyPath::BackgroundColor => "backgroundColor",
            KeyPath::CornerRadius => "cornerRadius",
            KeyPath::BorderWidth => "borderWidth",
            KeyPath::BorderColor => "borderColor",
            KeyPath::ShadowOpacity => "shadowOpacity",
            KeyPath::ShadowRadius => "shadowRadius",
            KeyPath::ShadowOffset => "shadowOffset",
            KeyPath::Bounds => "bounds",
            KeyPath::Custom(s) => s,
        }
    }

    /// Creates an `NSString` for this key path.
    ///
    /// This is used internally when constructing `CABasicAnimation`.
    fn to_nsstring(self) -> Retained<NSString> {
        NSString::from_str(self.as_str())
    }
}

/// Timing curve for animation interpolation.
///
/// Controls how the animation progresses over time. The default is `InOut`
/// which provides smooth acceleration and deceleration.
///
/// # Examples
///
/// ```ignore
/// // Linear motion (constant speed)
/// builder.easing(Easing::Linear)
///
/// // Smooth start and end (default)
/// builder.easing(Easing::InOut)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Easing {
    /// Constant speed throughout the animation.
    Linear,
    /// Slow start, accelerating to full speed.
    In,
    /// Fast start, decelerating to a stop.
    Out,
    /// Slow start and end with acceleration in the middle (default).
    #[default]
    InOut,
}

impl Easing {
    /// Creates the corresponding `CAMediaTimingFunction` for this easing curve.
    fn to_timing_function(self) -> Retained<CAMediaTimingFunction> {
        // SAFETY: The timing function name constants are valid extern statics
        // that are always available on macOS.
        let name = unsafe {
            match self {
                Easing::Linear => kCAMediaTimingFunctionLinear,
                Easing::In => kCAMediaTimingFunctionEaseIn,
                Easing::Out => kCAMediaTimingFunctionEaseOut,
                Easing::InOut => kCAMediaTimingFunctionEaseInEaseOut,
            }
        };
        CAMediaTimingFunction::functionWithName(name)
    }
}

/// Repeat behavior for animations.
///
/// Controls how many times the animation plays before stopping.
///
/// # Examples
///
/// ```ignore
/// // Play once and hold (default)
/// builder.repeat(Repeat::Once)
///
/// // Play 3 times
/// builder.repeat(Repeat::Times(3))
///
/// // Loop forever
/// builder.repeat(Repeat::Forever)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Repeat {
    /// Play the animation once and hold the final value (default).
    #[default]
    Once,
    /// Play the animation a specific number of times.
    Times(u32),
    /// Loop the animation indefinitely.
    Forever,
}

impl Repeat {
    /// Returns the `repeatCount` value for Core Animation.
    ///
    /// - `Once` returns 1.0 (play once)
    /// - `Times(n)` returns n as f32
    /// - `Forever` returns `f32::INFINITY`
    fn to_repeat_count(self) -> f32 {
        match self {
            Repeat::Once => 1.0,
            Repeat::Times(n) => n as f32,
            Repeat::Forever => f32::INFINITY,
        }
    }
}

/// Builder for configuring `CABasicAnimation` instances.
///
/// Creates from→to animations on a single property. The builder uses
/// sensible defaults:
/// - `fillMode = forwards` (values persist after animation)
/// - `removedOnCompletion = false` (animation stays attached)
/// - `easing = InOut` (smooth acceleration/deceleration)
/// - `repeat = Once` (play once)
///
/// # Examples
///
/// ```ignore
/// // Simple pulse animation
/// let anim = CABasicAnimationBuilder::new(KeyPath::TransformScale)
///     .values(0.85, 1.15)
///     .duration(800.millis())
///     .autoreverses()
///     .repeat(Repeat::Forever)
///     .build();
///
/// layer.addAnimation_forKey(&anim, Some(ns_string!("pulse")));
/// ```
///
/// # Default Behaviors
///
/// By default, the animation's final value persists on the layer after
/// completion. This differs from raw `CABasicAnimation` which snaps back
/// to the original value. Call `.remove_on_completion()` to opt-in to
/// snap-back behavior.
pub struct CABasicAnimationBuilder {
    key_path: KeyPath,
    from_value: Option<f64>,
    to_value: Option<f64>,
    duration: Duration,
    easing: Easing,
    autoreverses: bool,
    repeat: Repeat,
    phase_offset: f64,
    remove_on_completion: bool,
}

impl CABasicAnimationBuilder {
    /// Creates a new animation builder for the specified property.
    ///
    /// # Arguments
    ///
    /// * `key_path` - The property to animate (e.g., `KeyPath::TransformScale`)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let builder = CABasicAnimationBuilder::new(KeyPath::Opacity);
    /// ```
    #[must_use]
    pub fn new(key_path: KeyPath) -> Self {
        Self {
            key_path,
            from_value: None,
            to_value: None,
            duration: Duration::from_millis(250),
            easing: Easing::default(),
            autoreverses: false,
            repeat: Repeat::default(),
            phase_offset: 0.0,
            remove_on_completion: false,
        }
    }

    /// Sets the from and to values for the animation.
    ///
    /// The values are interpreted based on the key path:
    /// - Scale properties: 0.0 = invisible, 1.0 = normal
    /// - Opacity: 0.0 = transparent, 1.0 = opaque
    /// - Rotation: radians
    /// - Position: points
    ///
    /// # Arguments
    ///
    /// * `from` - Starting value
    /// * `to` - Ending value
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Scale from 85% to 115%
    /// builder.values(0.85, 1.15)
    ///
    /// // Fade from fully opaque to 70% opacity
    /// builder.values(1.0, 0.7)
    /// ```
    #[must_use]
    pub fn values(mut self, from: f64, to: f64) -> Self {
        self.from_value = Some(from);
        self.to_value = Some(to);
        self
    }

    /// Sets the duration of one animation cycle.
    ///
    /// # Arguments
    ///
    /// * `duration` - The time for one complete animation cycle
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::time::Duration;
    ///
    /// builder.duration(Duration::from_millis(800))
    ///
    /// // Or with DurationExt:
    /// builder.duration(800.millis())
    /// builder.duration(1.5.seconds())
    /// ```
    #[must_use]
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the easing curve for the animation.
    ///
    /// The default is `Easing::InOut` which provides smooth acceleration
    /// at the start and deceleration at the end.
    ///
    /// # Arguments
    ///
    /// * `easing` - The timing curve to use
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Constant speed
    /// builder.easing(Easing::Linear)
    ///
    /// // Quick start, slow end
    /// builder.easing(Easing::Out)
    /// ```
    #[must_use]
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Enables ping-pong animation (play forward then backward).
    ///
    /// When combined with `repeat(Repeat::Forever)`, creates a smooth
    /// oscillating animation.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Pulse that smoothly grows and shrinks
    /// builder
    ///     .values(0.9, 1.1)
    ///     .autoreverses()
    ///     .repeat(Repeat::Forever)
    /// ```
    #[must_use]
    pub fn autoreverses(mut self) -> Self {
        self.autoreverses = true;
        self
    }

    /// Sets the repeat behavior for the animation.
    ///
    /// # Arguments
    ///
    /// * `repeat` - How many times to play the animation
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Play 3 times
    /// builder.repeat(Repeat::Times(3))
    ///
    /// // Loop forever
    /// builder.repeat(Repeat::Forever)
    /// ```
    #[must_use]
    pub fn repeat(mut self, repeat: Repeat) -> Self {
        self.repeat = repeat;
        self
    }

    /// Sets the phase offset (starting point within the animation cycle).
    ///
    /// This allows multiple animations to be out of phase with each other,
    /// creating interesting visual effects.
    ///
    /// # Arguments
    ///
    /// * `offset` - Fraction of the cycle to skip (0.0 to 1.0).
    ///   - 0.0 = start at beginning (default)
    ///   - 0.5 = start at midpoint
    ///   - 1.0 = start at end (same as beginning for looping animations)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Two circles pulsing out of phase
    /// let anim1 = builder.clone().phase_offset(0.0).build();
    /// let anim2 = builder.clone().phase_offset(0.5).build();
    /// ```
    #[must_use]
    pub fn phase_offset(mut self, offset: f64) -> Self {
        self.phase_offset = offset;
        self
    }

    /// Opts in to snap-back behavior (remove animation on completion).
    ///
    /// By default, the animation's final value persists on the layer.
    /// Calling this method causes the layer to snap back to its original
    /// value when the animation completes.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Animation snaps back to original value when done
    /// builder
    ///     .values(0.0, 1.0)
    ///     .repeat(Repeat::Times(3))
    ///     .remove_on_completion()
    /// ```
    #[must_use]
    pub fn remove_on_completion(mut self) -> Self {
        self.remove_on_completion = true;
        self
    }

    /// Builds and returns the configured `CABasicAnimation`.
    ///
    /// # Returns
    ///
    /// A retained `CABasicAnimation` ready to be added to a layer.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let anim = CABasicAnimationBuilder::new(KeyPath::TransformScale)
    ///     .values(0.85, 1.15)
    ///     .duration(800.millis())
    ///     .autoreverses()
    ///     .repeat(Repeat::Forever)
    ///     .build();
    ///
    /// layer.addAnimation_forKey(&anim, Some(ns_string!("pulse")));
    /// ```
    #[must_use]
    pub fn build(self) -> Retained<CABasicAnimation> {
        let key_path_str = self.key_path.to_nsstring();
        let anim = CABasicAnimation::animationWithKeyPath(Some(&key_path_str));

        // Set from/to values if provided
        if let Some(from) = self.from_value {
            let from_number = NSNumber::new_f64(from);
            // SAFETY: NSNumber is a valid object type for fromValue
            unsafe {
                anim.setFromValue(Some(&from_number));
            }
        }
        if let Some(to) = self.to_value {
            let to_number = NSNumber::new_f64(to);
            // SAFETY: NSNumber is a valid object type for toValue
            unsafe {
                anim.setToValue(Some(&to_number));
            }
        }

        // Set timing properties (from CAMediaTiming trait)
        let duration_secs = self.duration.as_secs_f64();
        anim.setDuration(duration_secs);
        anim.setAutoreverses(self.autoreverses);
        anim.setRepeatCount(self.repeat.to_repeat_count());

        // Set phase offset as timeOffset
        // For autoreverses, multiply by 2 because the full cycle is forward + backward
        if self.phase_offset > 0.0 {
            let cycle_duration = if self.autoreverses {
                duration_secs * 2.0
            } else {
                duration_secs
            };
            anim.setTimeOffset(self.phase_offset * cycle_duration);
        }

        // Set timing function (easing)
        let timing_function = self.easing.to_timing_function();
        anim.setTimingFunction(Some(&timing_function));

        // Set fill mode and removedOnCompletion for value persistence
        if self.remove_on_completion {
            anim.setRemovedOnCompletion(true);
            // Default fill mode is fine for removal
        } else {
            anim.setRemovedOnCompletion(false);
            // SAFETY: kCAFillModeForwards is a valid extern static
            anim.setFillMode(unsafe { kCAFillModeForwards });
        }

        anim
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_path_as_str() {
        assert_eq!(KeyPath::TransformScale.as_str(), "transform.scale");
        assert_eq!(KeyPath::TransformScaleX.as_str(), "transform.scale.x");
        assert_eq!(KeyPath::TransformScaleY.as_str(), "transform.scale.y");
        assert_eq!(KeyPath::TransformRotation.as_str(), "transform.rotation.z");
        assert_eq!(KeyPath::Opacity.as_str(), "opacity");
        assert_eq!(KeyPath::Position.as_str(), "position");
        assert_eq!(KeyPath::PositionX.as_str(), "position.x");
        assert_eq!(KeyPath::PositionY.as_str(), "position.y");
        assert_eq!(KeyPath::BackgroundColor.as_str(), "backgroundColor");
        assert_eq!(KeyPath::CornerRadius.as_str(), "cornerRadius");
        assert_eq!(KeyPath::BorderWidth.as_str(), "borderWidth");
        assert_eq!(KeyPath::BorderColor.as_str(), "borderColor");
        assert_eq!(KeyPath::ShadowOpacity.as_str(), "shadowOpacity");
        assert_eq!(KeyPath::ShadowRadius.as_str(), "shadowRadius");
        assert_eq!(KeyPath::ShadowOffset.as_str(), "shadowOffset");
        assert_eq!(KeyPath::Bounds.as_str(), "bounds");
        assert_eq!(KeyPath::Custom("custom.path").as_str(), "custom.path");
    }

    #[test]
    fn test_easing_default() {
        assert_eq!(Easing::default(), Easing::InOut);
    }

    #[test]
    fn test_repeat_default() {
        assert_eq!(Repeat::default(), Repeat::Once);
    }

    #[test]
    fn test_repeat_to_count() {
        assert_eq!(Repeat::Once.to_repeat_count(), 1.0);
        assert_eq!(Repeat::Times(5).to_repeat_count(), 5.0);
        assert!(Repeat::Forever.to_repeat_count().is_infinite());
    }

    #[test]
    fn test_builder_defaults() {
        let builder = CABasicAnimationBuilder::new(KeyPath::Opacity);
        assert_eq!(builder.key_path, KeyPath::Opacity);
        assert_eq!(builder.from_value, None);
        assert_eq!(builder.to_value, None);
        assert_eq!(builder.duration, Duration::from_millis(250));
        assert_eq!(builder.easing, Easing::InOut);
        assert!(!builder.autoreverses);
        assert_eq!(builder.repeat, Repeat::Once);
        assert_eq!(builder.phase_offset, 0.0);
        assert!(!builder.remove_on_completion);
    }

    #[test]
    fn test_builder_chaining() {
        let builder = CABasicAnimationBuilder::new(KeyPath::TransformScale)
            .values(0.5, 1.5)
            .duration(Duration::from_secs(1))
            .easing(Easing::Linear)
            .autoreverses()
            .repeat(Repeat::Forever)
            .phase_offset(0.25)
            .remove_on_completion();

        assert_eq!(builder.key_path, KeyPath::TransformScale);
        assert_eq!(builder.from_value, Some(0.5));
        assert_eq!(builder.to_value, Some(1.5));
        assert_eq!(builder.duration, Duration::from_secs(1));
        assert_eq!(builder.easing, Easing::Linear);
        assert!(builder.autoreverses);
        assert_eq!(builder.repeat, Repeat::Forever);
        assert_eq!(builder.phase_offset, 0.25);
        assert!(builder.remove_on_completion);
    }
}
