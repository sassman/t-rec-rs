//! Animation presets for layer animations.
//!
//! Provides declarative animation definitions that backends translate to
//! platform-specific implementations.

use std::time::Duration;

use crate::DurationExt;

/// Easing function for animation timing.
///
/// Controls how the animation progresses over time. Mirrors the Core Animation
/// timing model for direct mapping on macOS.
///
/// # Examples
///
/// ```ignore
/// Animation::pulse().easing(Easing::InOut)
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

/// Repeat behavior for animations.
///
/// Controls how many times the animation plays before stopping.
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

/// Animation preset for layer animations.
///
/// Each variant represents a type of animation that can be applied to a layer.
/// Animations are configured with parameters like duration, easing, and range.
///
/// # Examples
///
/// ```ignore
/// // Simple pulse animation with defaults
/// layer.animate(Animation::pulse())
///
/// // Custom pulse range
/// layer.animate(Animation::Pulse {
///     min_scale: 0.8,
///     max_scale: 1.3,
///     duration: 800u64.millis(),
///     easing: Easing::InOut,
/// })
///
/// // Fade animation
/// layer.animate(Animation::fade_in())
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Animation {
    /// Scale pulse animation (heartbeat effect).
    ///
    /// Animates the layer's scale between min and max values, creating
    /// a pulsing effect. Uses autoreverses for smooth oscillation.
    Pulse {
        /// Minimum scale factor (default: 0.9).
        min_scale: f64,
        /// Maximum scale factor (default: 1.1).
        max_scale: f64,
        /// Duration of one pulse cycle (default: 800ms).
        duration: Duration,
        /// Timing function (default: InOut).
        easing: Easing,
    },

    /// Opacity fade animation.
    ///
    /// Animates the layer's opacity between two values.
    Fade {
        /// Starting opacity (0.0 = transparent, 1.0 = opaque).
        from: f32,
        /// Ending opacity.
        to: f32,
        /// Duration of the fade.
        duration: Duration,
        /// Timing function (default: InOut).
        easing: Easing,
        /// Whether to autoreverse (ping-pong).
        autoreverses: bool,
        /// Repeat behavior.
        repeat: Repeat,
    },

    /// Shadow/glow intensity animation.
    ///
    /// Animates the layer's shadow radius to create a glowing effect.
    Glow {
        /// Minimum shadow radius.
        min_radius: f64,
        /// Maximum shadow radius.
        max_radius: f64,
        /// Duration of one glow cycle.
        duration: Duration,
        /// Timing function (default: InOut).
        easing: Easing,
    },

    /// Rotation animation.
    ///
    /// Animates the layer's z-axis rotation.
    Rotate {
        /// Starting angle in radians.
        from: f64,
        /// Ending angle in radians.
        to: f64,
        /// Duration of the rotation.
        duration: Duration,
        /// Timing function (default: Linear).
        easing: Easing,
        /// Repeat behavior.
        repeat: Repeat,
    },

    /// Composed animations (run in parallel).
    ///
    /// Multiple animations applied to the same layer simultaneously.
    Group(Vec<Animation>),
}

impl Animation {
    /// Create a default pulse animation.
    ///
    /// Returns a pulse animation with scale 0.9 to 1.1 over 800ms.
    pub fn pulse() -> Self {
        Self::Pulse {
            min_scale: 0.9,
            max_scale: 1.1,
            duration: 800u64.millis(),
            easing: Easing::InOut,
        }
    }

    /// Create a pulse animation with custom scale range.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum scale factor
    /// * `max` - Maximum scale factor
    pub fn pulse_range(min: f64, max: f64) -> Self {
        Self::Pulse {
            min_scale: min,
            max_scale: max,
            duration: 800u64.millis(),
            easing: Easing::InOut,
        }
    }

    /// Create a fade-in animation (opacity 0 to 1).
    pub fn fade_in() -> Self {
        Self::Fade {
            from: 0.0,
            to: 1.0,
            duration: 300u64.millis(),
            easing: Easing::InOut,
            autoreverses: false,
            repeat: Repeat::Once,
        }
    }

    /// Create a fade-out animation (opacity 1 to 0).
    pub fn fade_out() -> Self {
        Self::Fade {
            from: 1.0,
            to: 0.0,
            duration: 300u64.millis(),
            easing: Easing::InOut,
            autoreverses: false,
            repeat: Repeat::Once,
        }
    }

    /// Create a looping fade animation that oscillates between two opacity values.
    pub fn fade_loop(from: f32, to: f32) -> Self {
        Self::Fade {
            from,
            to,
            duration: 800u64.millis(),
            easing: Easing::InOut,
            autoreverses: true,
            repeat: Repeat::Forever,
        }
    }

    /// Create a default glow animation.
    ///
    /// Returns a glow animation with radius 5 to 15 over 800ms.
    pub fn glow() -> Self {
        Self::Glow {
            min_radius: 5.0,
            max_radius: 15.0,
            duration: 800u64.millis(),
            easing: Easing::InOut,
        }
    }

    /// Create a glow animation with custom radius range.
    ///
    /// # Arguments
    ///
    /// * `min` - Minimum shadow radius
    /// * `max` - Maximum shadow radius
    pub fn glow_range(min: f64, max: f64) -> Self {
        Self::Glow {
            min_radius: min,
            max_radius: max,
            duration: 800u64.millis(),
            easing: Easing::InOut,
        }
    }

    /// Create a continuous rotation animation.
    ///
    /// Rotates the layer 360 degrees over the specified duration.
    pub fn spin(duration: Duration) -> Self {
        Self::Rotate {
            from: 0.0,
            to: std::f64::consts::TAU,
            duration,
            easing: Easing::Linear,
            repeat: Repeat::Forever,
        }
    }

    /// Group multiple animations to run in parallel.
    pub fn group(animations: Vec<Animation>) -> Self {
        Self::Group(animations)
    }

    /// Set the duration of this animation.
    ///
    /// Returns a new animation with the specified duration.
    pub fn duration(self, duration: Duration) -> Self {
        match self {
            Self::Pulse {
                min_scale,
                max_scale,
                easing,
                ..
            } => Self::Pulse {
                min_scale,
                max_scale,
                duration,
                easing,
            },
            Self::Fade {
                from,
                to,
                easing,
                autoreverses,
                repeat,
                ..
            } => Self::Fade {
                from,
                to,
                duration,
                easing,
                autoreverses,
                repeat,
            },
            Self::Glow {
                min_radius,
                max_radius,
                easing,
                ..
            } => Self::Glow {
                min_radius,
                max_radius,
                duration,
                easing,
            },
            Self::Rotate {
                from,
                to,
                easing,
                repeat,
                ..
            } => Self::Rotate {
                from,
                to,
                duration,
                easing,
                repeat,
            },
            Self::Group(animations) => {
                Self::Group(animations.into_iter().map(|a| a.duration(duration)).collect())
            }
        }
    }

    /// Set the easing function of this animation.
    ///
    /// Returns a new animation with the specified easing.
    pub fn easing(self, new_easing: Easing) -> Self {
        match self {
            Self::Pulse {
                min_scale,
                max_scale,
                duration,
                ..
            } => Self::Pulse {
                min_scale,
                max_scale,
                duration,
                easing: new_easing,
            },
            Self::Fade {
                from,
                to,
                duration,
                autoreverses,
                repeat,
                ..
            } => Self::Fade {
                from,
                to,
                duration,
                easing: new_easing,
                autoreverses,
                repeat,
            },
            Self::Glow {
                min_radius,
                max_radius,
                duration,
                ..
            } => Self::Glow {
                min_radius,
                max_radius,
                duration,
                easing: new_easing,
            },
            Self::Rotate {
                from,
                to,
                duration,
                repeat,
                ..
            } => Self::Rotate {
                from,
                to,
                duration,
                easing: new_easing,
                repeat,
            },
            Self::Group(animations) => {
                Self::Group(animations.into_iter().map(|a| a.easing(new_easing)).collect())
            }
        }
    }
}

impl Default for Animation {
    fn default() -> Self {
        Self::pulse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pulse_default() {
        let anim = Animation::pulse();
        if let Animation::Pulse {
            min_scale,
            max_scale,
            duration,
            easing,
        } = anim
        {
            assert_eq!(min_scale, 0.9);
            assert_eq!(max_scale, 1.1);
            assert_eq!(duration, Duration::from_millis(800));
            assert_eq!(easing, Easing::InOut);
        } else {
            panic!("Expected Pulse animation");
        }
    }

    #[test]
    fn test_pulse_range() {
        let anim = Animation::pulse_range(0.5, 2.0);
        if let Animation::Pulse {
            min_scale,
            max_scale,
            ..
        } = anim
        {
            assert_eq!(min_scale, 0.5);
            assert_eq!(max_scale, 2.0);
        } else {
            panic!("Expected Pulse animation");
        }
    }

    #[test]
    fn test_fade_in() {
        let anim = Animation::fade_in();
        if let Animation::Fade { from, to, .. } = anim {
            assert_eq!(from, 0.0);
            assert_eq!(to, 1.0);
        } else {
            panic!("Expected Fade animation");
        }
    }

    #[test]
    fn test_fade_out() {
        let anim = Animation::fade_out();
        if let Animation::Fade { from, to, .. } = anim {
            assert_eq!(from, 1.0);
            assert_eq!(to, 0.0);
        } else {
            panic!("Expected Fade animation");
        }
    }

    #[test]
    fn test_glow_default() {
        let anim = Animation::glow();
        if let Animation::Glow {
            min_radius,
            max_radius,
            ..
        } = anim
        {
            assert_eq!(min_radius, 5.0);
            assert_eq!(max_radius, 15.0);
        } else {
            panic!("Expected Glow animation");
        }
    }

    #[test]
    fn test_duration_modifier() {
        let anim = Animation::pulse().duration(Duration::from_secs(2));
        if let Animation::Pulse { duration, .. } = anim {
            assert_eq!(duration, Duration::from_secs(2));
        } else {
            panic!("Expected Pulse animation");
        }
    }

    #[test]
    fn test_easing_modifier() {
        let anim = Animation::pulse().easing(Easing::Linear);
        if let Animation::Pulse { easing, .. } = anim {
            assert_eq!(easing, Easing::Linear);
        } else {
            panic!("Expected Pulse animation");
        }
    }

    #[test]
    fn test_easing_default() {
        assert_eq!(Easing::default(), Easing::InOut);
    }

    #[test]
    fn test_repeat_default() {
        assert_eq!(Repeat::default(), Repeat::Once);
    }
}
