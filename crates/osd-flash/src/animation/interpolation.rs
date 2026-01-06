//! Interpolation logic for animations.
//!
//! This module provides functions for smoothly interpolating between
//! keyframe values, including transforms, shapes, and colors.
//!
//! # Note on Easing
//!
//! The functions in this module perform **linear interpolation**. Easing
//! is applied at a higher level (in the animation runner) by transforming
//! the `t` parameter before calling these functions:
//!
//! ```ignore
//! // In AnimationRunner:
//! let eased_t = easing.apply(linear_t);
//! let frame = interpolate(&from, &to, eased_t);
//! ```
//!
//! This separation keeps the interpolation logic simple and composable.

use crate::geometry::Point;
use crate::icon::StyledShape;
use crate::shape::Shape;
use crate::style::Paint;
use crate::Color;

use super::keyframe::Keyframe;
use super::transform::Transform;

/// The computed frame data ready for rendering.
#[derive(Clone, Debug)]
pub struct InterpolatedFrame {
    /// The interpolated transform to apply.
    pub transform: Transform,
    /// The interpolated overlay shapes to render.
    pub shapes: Vec<StyledShape>,
}

/// Linear interpolation between two values.
///
/// # Arguments
///
/// * `a` - Start value (returned when t = 0)
/// * `b` - End value (returned when t = 1)
/// * `t` - Interpolation factor, typically in [0, 1] (but not clamped)
///
/// # Note
///
/// For eased animations, `t` should be pre-transformed via `Easing::apply()`.
#[inline]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Interpolate between two keyframes.
///
/// # Arguments
///
/// * `from` - The starting keyframe
/// * `to` - The ending keyframe
/// * `t` - Interpolation factor (0.0 = from, 1.0 = to)
///
/// # Note
///
/// The `t` parameter should already have easing applied if desired.
/// This function performs linear interpolation only.
///
/// # Returns
///
/// An `InterpolatedFrame` with interpolated transform and shapes.
pub fn interpolate(from: &Keyframe, to: &Keyframe, t: f64) -> InterpolatedFrame {
    InterpolatedFrame {
        transform: interpolate_transform(&from.transform, &to.transform, t),
        shapes: interpolate_shapes(&from.shapes, &to.shapes, t),
    }
}

/// Interpolate between two transforms.
fn interpolate_transform(from: &Transform, to: &Transform, t: f64) -> Transform {
    Transform {
        scale: lerp(from.scale, to.scale, t),
    }
}

/// Interpolate shapes by index and type.
///
/// Shapes are matched by their position in the list. If shapes at the same
/// index have the same type (e.g., both circles), they are interpolated.
/// If types differ, the shape from the "from" keyframe is used.
pub fn interpolate_shapes(from: &[StyledShape], to: &[StyledShape], t: f64) -> Vec<StyledShape> {
    let mut result = Vec::new();
    let max_len = from.len().max(to.len());

    for i in 0..max_len {
        match (from.get(i), to.get(i)) {
            (Some(a), Some(b)) if shapes_compatible(a, b) => {
                result.push(interpolate_shape(a, b, t));
            }
            (Some(a), Some(_)) => {
                // Types don't match: use "from" shape
                result.push(a.clone());
            }
            (Some(a), None) => {
                // No "to" shape at this index: use "from"
                result.push(a.clone());
            }
            (None, Some(_)) => {
                // No "from" shape: skip (shape appears only in "to")
                // Future: could fade in
            }
            (None, None) => {
                // Should not happen
            }
        }
    }

    result
}

/// Check if two shapes can be interpolated.
///
/// Shapes must be of the same variant (Circle <-> Circle, etc.)
fn shapes_compatible(a: &StyledShape, b: &StyledShape) -> bool {
    std::mem::discriminant(&a.shape) == std::mem::discriminant(&b.shape)
}

/// Interpolate a single shape's properties.
fn interpolate_shape(from: &StyledShape, to: &StyledShape, t: f64) -> StyledShape {
    let shape = match (&from.shape, &to.shape) {
        (
            Shape::Circle {
                center: c1,
                radius: r1,
            },
            Shape::Circle {
                center: c2,
                radius: r2,
            },
        ) => Shape::Circle {
            center: Point::new(lerp(c1.x, c2.x, t), lerp(c1.y, c2.y, t)),
            radius: lerp(*r1, *r2, t),
        },

        (
            Shape::RoundedRect {
                rect: rect1,
                corner_radius: cr1,
            },
            Shape::RoundedRect {
                rect: rect2,
                corner_radius: cr2,
            },
        ) => Shape::RoundedRect {
            rect: crate::geometry::Rect::from_xywh(
                lerp(rect1.origin.x, rect2.origin.x, t),
                lerp(rect1.origin.y, rect2.origin.y, t),
                lerp(rect1.size.width, rect2.size.width, t),
                lerp(rect1.size.height, rect2.size.height, t),
            ),
            corner_radius: lerp(*cr1, *cr2, t),
        },

        (Shape::Ellipse { rect: rect1 }, Shape::Ellipse { rect: rect2 }) => Shape::Ellipse {
            rect: crate::geometry::Rect::from_xywh(
                lerp(rect1.origin.x, rect2.origin.x, t),
                lerp(rect1.origin.y, rect2.origin.y, t),
                lerp(rect1.size.width, rect2.size.width, t),
                lerp(rect1.size.height, rect2.size.height, t),
            ),
        },

        // Fallback: return "from" shape unchanged
        _ => from.shape.clone(),
    };

    let paint = interpolate_paint(&from.paint, &to.paint, t);

    StyledShape { shape, paint }
}

/// Interpolate between two paint values.
fn interpolate_paint(from: &Paint, to: &Paint, t: f64) -> Paint {
    Paint {
        color: interpolate_color(&from.color, &to.color, t),
        opacity: lerp(from.opacity, to.opacity, t),
    }
}

/// Interpolate between two colors (per-channel lerp).
pub fn interpolate_color(from: &Color, to: &Color, t: f64) -> Color {
    Color::rgba(
        lerp(from.r, to.r, t),
        lerp(from.g, to.g, t),
        lerp(from.b, to.b, t),
        lerp(from.a, to.a, t),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lerp() {
        assert!((lerp(0.0, 10.0, 0.0) - 0.0).abs() < f64::EPSILON);
        assert!((lerp(0.0, 10.0, 0.5) - 5.0).abs() < f64::EPSILON);
        assert!((lerp(0.0, 10.0, 1.0) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_interpolate_transform() {
        let from = Transform::new(0.95);
        let to = Transform::new(1.05);
        let result = interpolate_transform(&from, &to, 0.5);
        assert!((result.scale - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_interpolate_color() {
        let from = Color::rgba(0.0, 0.0, 0.0, 0.0);
        let to = Color::rgba(1.0, 1.0, 1.0, 1.0);
        let result = interpolate_color(&from, &to, 0.5);

        assert!((result.r - 0.5).abs() < f64::EPSILON);
        assert!((result.g - 0.5).abs() < f64::EPSILON);
        assert!((result.b - 0.5).abs() < f64::EPSILON);
        assert!((result.a - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_interpolate_circle() {
        let from = StyledShape::new(Shape::circle_at(0.0, 0.0, 10.0), Color::BLACK);
        let to = StyledShape::new(Shape::circle_at(100.0, 100.0, 20.0), Color::WHITE);

        let result = interpolate_shape(&from, &to, 0.5);

        if let Shape::Circle { center, radius } = result.shape {
            assert!((center.x - 50.0).abs() < f64::EPSILON);
            assert!((center.y - 50.0).abs() < f64::EPSILON);
            assert!((radius - 15.0).abs() < f64::EPSILON);
        } else {
            panic!("Expected Circle shape");
        }
    }

    #[test]
    fn test_shapes_compatible() {
        let circle1 = StyledShape::new(Shape::circle_at(0.0, 0.0, 10.0), Color::RED);
        let circle2 = StyledShape::new(Shape::circle_at(10.0, 10.0, 20.0), Color::BLUE);
        let rect = StyledShape::new(
            Shape::rounded_rect_xywh(0.0, 0.0, 10.0, 10.0, 2.0),
            Color::GREEN,
        );

        assert!(shapes_compatible(&circle1, &circle2));
        assert!(!shapes_compatible(&circle1, &rect));
    }

    #[test]
    fn test_interpolate_shapes_different_lengths() {
        let from = vec![
            StyledShape::new(Shape::circle_at(0.0, 0.0, 10.0), Color::RED),
            StyledShape::new(Shape::circle_at(10.0, 10.0, 5.0), Color::BLUE),
        ];
        let to = vec![StyledShape::new(
            Shape::circle_at(100.0, 100.0, 20.0),
            Color::WHITE,
        )];

        let result = interpolate_shapes(&from, &to, 0.5);

        // First shape interpolated, second shape from "from" (no match in "to")
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_interpolate_keyframes() {
        let from = Keyframe {
            progress: 0.0,
            transform: Transform::new(0.95),
            shapes: vec![StyledShape::new(
                Shape::circle_at(40.0, 40.0, 20.0),
                Color::rgba(1.0, 0.0, 0.0, 0.3),
            )],
            easing: None,
        };

        let to = Keyframe {
            progress: 0.7,
            transform: Transform::new(1.0),
            shapes: vec![StyledShape::new(
                Shape::circle_at(40.0, 40.0, 30.0),
                Color::rgba(1.0, 0.0, 0.0, 0.7),
            )],
            easing: None,
        };

        let frame = interpolate(&from, &to, 0.5);

        // Transform should be interpolated
        assert!((frame.transform.scale - 0.975).abs() < f64::EPSILON);

        // Shape should be interpolated
        assert_eq!(frame.shapes.len(), 1);
        if let Shape::Circle { radius, .. } = frame.shapes[0].shape {
            assert!((radius - 25.0).abs() < f64::EPSILON);
        }
    }
}
