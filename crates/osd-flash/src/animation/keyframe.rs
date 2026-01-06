//! Keyframe types for animation definition.
//!
//! Keyframes define the state of an animation at specific points in time.
//! The animation system interpolates between keyframes to create smooth motion.

use crate::icon::StyledShape;
use crate::shape::Shape;
use crate::Color;

use super::easing::Easing;
use super::transform::Transform;

/// A single keyframe in an animation timeline.
///
/// Keyframes define the visual state at a specific point in the animation cycle.
/// The animation system interpolates between consecutive keyframes.
#[derive(Clone, Debug)]
pub struct Keyframe {
    /// Progress point in animation (0.0 to 1.0).
    ///
    /// - `0.0` = start of animation cycle
    /// - `0.5` = halfway through
    /// - `1.0` = end of cycle (loops back to 0.0)
    pub progress: f64,

    /// Transform to apply at this keyframe.
    pub transform: Transform,

    /// Overlay shapes to render at this keyframe.
    ///
    /// These shapes are drawn on top of the base icon content.
    pub shapes: Vec<StyledShape>,

    /// Easing for transition INTO this keyframe.
    ///
    /// If `None`, inherits the animation's default easing.
    pub easing: Option<Easing>,
}

impl Keyframe {
    /// Create a new keyframe at the given progress.
    pub fn new(progress: f64) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            transform: Transform::default(),
            shapes: Vec::new(),
            easing: None,
        }
    }
}

/// Builder for constructing keyframes fluently.
///
/// # Example
///
/// ```ignore
/// .keyframe(0.5, |k| {
///     k.scale(1.1)
///         .circle(40.0, 40.0, 20.0, Color::RED)
///         .easing(Easing::EaseOut)
/// })
/// ```
pub struct KeyframeBuilder {
    progress: f64,
    transform: Transform,
    shapes: Vec<StyledShape>,
    easing: Option<Easing>,
}

impl KeyframeBuilder {
    /// Create a new keyframe builder at the given progress.
    pub(crate) fn new(progress: f64) -> Self {
        Self {
            progress: progress.clamp(0.0, 1.0),
            transform: Transform::default(),
            shapes: Vec::new(),
            easing: None,
        }
    }

    /// Set scale transform (1.0 = original size).
    ///
    /// # Example
    ///
    /// ```ignore
    /// .keyframe(0.5, |k| k.scale(1.1)) // 10% larger
    /// ```
    pub fn scale(mut self, scale: f64) -> Self {
        self.transform.scale = scale;
        self
    }

    /// Add a circle overlay shape.
    ///
    /// # Arguments
    ///
    /// * `x` - Center X coordinate
    /// * `y` - Center Y coordinate
    /// * `radius` - Circle radius
    /// * `color` - Fill color
    pub fn circle(mut self, x: f64, y: f64, radius: f64, color: Color) -> Self {
        self.shapes.push(StyledShape::new(
            Shape::circle_at(x, y, radius),
            color,
        ));
        self
    }

    /// Add a rounded rectangle overlay shape.
    ///
    /// # Arguments
    ///
    /// * `x` - Top-left X coordinate
    /// * `y` - Top-left Y coordinate
    /// * `width` - Rectangle width
    /// * `height` - Rectangle height
    /// * `corner_radius` - Corner rounding radius
    /// * `color` - Fill color
    pub fn rounded_rect(
        mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        corner_radius: f64,
        color: Color,
    ) -> Self {
        self.shapes.push(StyledShape::new(
            Shape::rounded_rect_xywh(x, y, width, height, corner_radius),
            color,
        ));
        self
    }

    /// Add a generic styled shape overlay.
    pub fn shape(mut self, shape: StyledShape) -> Self {
        self.shapes.push(shape);
        self
    }

    /// Override easing for transition into this keyframe.
    ///
    /// The easing controls how the animation progresses from the previous
    /// keyframe to this one. If not set, the animation's default easing is used.
    ///
    /// # Example
    ///
    /// ```ignore
    /// .keyframe(0.7, |k| k.scale(1.0).easing(Easing::EaseOut))
    /// ```
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = Some(easing);
        self
    }

    /// Build the keyframe (internal use).
    pub(crate) fn build(self) -> Keyframe {
        Keyframe {
            progress: self.progress,
            transform: self.transform,
            shapes: self.shapes,
            easing: self.easing,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyframe_new() {
        let kf = Keyframe::new(0.5);
        assert!((kf.progress - 0.5).abs() < f64::EPSILON);
        assert!(kf.transform.is_identity());
        assert!(kf.shapes.is_empty());
        assert!(kf.easing.is_none());
    }

    #[test]
    fn test_keyframe_progress_clamped() {
        let kf1 = Keyframe::new(-0.5);
        assert!((kf1.progress - 0.0).abs() < f64::EPSILON);

        let kf2 = Keyframe::new(1.5);
        assert!((kf2.progress - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builder_scale() {
        let kf = KeyframeBuilder::new(0.0).scale(0.95).build();
        assert!((kf.transform.scale - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_builder_circle() {
        let kf = KeyframeBuilder::new(0.0)
            .circle(40.0, 40.0, 20.0, Color::RED)
            .build();
        assert_eq!(kf.shapes.len(), 1);
    }

    #[test]
    fn test_builder_easing() {
        let kf = KeyframeBuilder::new(0.5)
            .easing(Easing::EaseOut)
            .build();
        assert_eq!(kf.easing, Some(Easing::EaseOut));
    }

    #[test]
    fn test_builder_chain() {
        let kf = KeyframeBuilder::new(0.7)
            .scale(1.1)
            .circle(50.0, 50.0, 25.0, Color::BLUE)
            .circle(50.0, 50.0, 30.0, Color::rgba(0.0, 0.0, 1.0, 0.3))
            .easing(Easing::EaseIn)
            .build();

        assert!((kf.progress - 0.7).abs() < f64::EPSILON);
        assert!((kf.transform.scale - 1.1).abs() < f64::EPSILON);
        assert_eq!(kf.shapes.len(), 2);
        assert_eq!(kf.easing, Some(Easing::EaseIn));
    }
}
