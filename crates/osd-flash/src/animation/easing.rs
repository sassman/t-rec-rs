//! Easing functions for animation timing.
//!
//! Easing functions control how animation progress is mapped to visual change,
//! allowing for more natural-feeling motion. The default [`Easing::EaseInOut`]
//! provides smooth acceleration and deceleration.
//!
//! # Example
//!
//! ```
//! use osd_flash::animation::Easing;
//!
//! // Apply easing to linear progress
//! let linear_progress = 0.5;
//! let eased = Easing::EaseInOut.apply(linear_progress);
//! ```

/// Easing functions for animation timing.
///
/// Each variant transforms linear progress `[0, 1]` into eased progress `[0, 1]`.
/// The easing affects how quickly the animation progresses at different points.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Easing {
    /// Constant speed: `f(t) = t`
    ///
    /// Use for mechanical, robotic motion or when linear interpolation is desired.
    Linear,

    /// Slow start, fast end: `f(t) = t²`
    ///
    /// Use for objects accelerating from rest.
    EaseIn,

    /// Fast start, slow end: `f(t) = 1 - (1-t)²`
    ///
    /// Use for objects decelerating to rest.
    EaseOut,

    /// Slow start and end: quadratic S-curve
    ///
    /// This is the default easing, providing natural-feeling motion
    /// that accelerates smoothly and decelerates smoothly.
    #[default]
    EaseInOut,

    /// Custom cubic bezier curve (CSS-compatible).
    ///
    /// The four values represent the control points `(x1, y1, x2, y2)`
    /// of a cubic bezier curve from `(0, 0)` to `(1, 1)`.
    ///
    /// # Example
    ///
    /// ```
    /// use osd_flash::animation::Easing;
    ///
    /// // CSS "ease" equivalent
    /// let ease = Easing::CubicBezier(0.25, 0.1, 0.25, 1.0);
    /// ```
    CubicBezier(f64, f64, f64, f64),
}

impl Easing {
    /// Apply easing function to linear progress.
    ///
    /// # Arguments
    ///
    /// * `t` - Linear progress value, will be clamped to `[0, 1]`
    ///
    /// # Returns
    ///
    /// Eased progress value in `[0, 1]`
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Easing::CubicBezier(x1, y1, x2, y2) => cubic_bezier_at(t, *x1, *y1, *x2, *y2),
        }
    }
}

/// Compute the y-value of a cubic bezier curve at parameter t.
///
/// This uses Newton-Raphson iteration to find the t parameter for a given x,
/// then computes the corresponding y value.
fn cubic_bezier_at(t: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    // For a cubic bezier from (0,0) to (1,1) with control points (x1,y1) and (x2,y2):
    // B(t) = 3(1-t)²t·P1 + 3(1-t)t²·P2 + t³
    //
    // We need to find the t parameter where Bx(t) = input_t, then compute By(t).

    // Newton-Raphson to find t where x(t) = input_t
    let mut guess = t;
    for _ in 0..8 {
        let x = bezier_component(guess, x1, x2) - t;
        if x.abs() < 1e-7 {
            break;
        }
        let dx = bezier_component_derivative(guess, x1, x2);
        if dx.abs() < 1e-7 {
            break;
        }
        guess -= x / dx;
    }

    bezier_component(guess.clamp(0.0, 1.0), y1, y2)
}

/// Compute one component of a cubic bezier at parameter t.
/// B(t) = 3(1-t)²t·p1 + 3(1-t)t²·p2 + t³
#[inline]
fn bezier_component(t: f64, p1: f64, p2: f64) -> f64 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3
}

/// Compute the derivative of one component of a cubic bezier at parameter t.
/// B'(t) = 3(1-t)²·p1 + 6(1-t)t·(p2-p1) + 3t²·(1-p2)
#[inline]
fn bezier_component_derivative(t: f64, p1: f64, p2: f64) -> f64 {
    let mt = 1.0 - t;
    3.0 * mt * mt * p1 + 6.0 * mt * t * (p2 - p1) + 3.0 * t * t * (1.0 - p2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear() {
        assert!((Easing::Linear.apply(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((Easing::Linear.apply(0.5) - 0.5).abs() < f64::EPSILON);
        assert!((Easing::Linear.apply(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ease_in() {
        assert!((Easing::EaseIn.apply(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((Easing::EaseIn.apply(0.5) - 0.25).abs() < f64::EPSILON); // 0.5² = 0.25
        assert!((Easing::EaseIn.apply(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ease_out() {
        assert!((Easing::EaseOut.apply(0.0) - 0.0).abs() < f64::EPSILON);
        assert!((Easing::EaseOut.apply(0.5) - 0.75).abs() < f64::EPSILON); // 1 - 0.5² = 0.75
        assert!((Easing::EaseOut.apply(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ease_in_out() {
        assert!((Easing::EaseInOut.apply(0.0) - 0.0).abs() < f64::EPSILON);
        // At t=0.5, should be exactly 0.5 (inflection point)
        assert!((Easing::EaseInOut.apply(0.5) - 0.5).abs() < f64::EPSILON);
        assert!((Easing::EaseInOut.apply(1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_clamp() {
        // Values outside [0, 1] should be clamped
        assert!((Easing::Linear.apply(-0.5) - 0.0).abs() < f64::EPSILON);
        assert!((Easing::Linear.apply(1.5) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_cubic_bezier_linear() {
        // Linear bezier (0.0, 0.0, 1.0, 1.0) should behave like Linear
        let bezier = Easing::CubicBezier(0.0, 0.0, 1.0, 1.0);
        assert!((bezier.apply(0.0) - 0.0).abs() < 0.01);
        assert!((bezier.apply(0.5) - 0.5).abs() < 0.01);
        assert!((bezier.apply(1.0) - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_cubic_bezier_ease() {
        // CSS "ease" curve (0.25, 0.1, 0.25, 1.0)
        let ease = Easing::CubicBezier(0.25, 0.1, 0.25, 1.0);
        // Just verify it produces reasonable values
        assert!(ease.apply(0.0) >= 0.0);
        assert!(ease.apply(1.0) <= 1.0);
        assert!(ease.apply(0.5) > 0.0 && ease.apply(0.5) < 1.0);
    }

    #[test]
    fn test_default() {
        assert_eq!(Easing::default(), Easing::EaseInOut);
    }
}
