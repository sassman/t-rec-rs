//! Drawable shape primitives.
//!
//! Platform-agnostic shape definitions that can be rendered by any backend.
//! Shapes are pure geometry - styling is handled separately by [`Paint`](crate::style::Paint).

use crate::geometry::{Point, Rect};

/// A geometric shape (pure geometry, no styling).
///
/// Shapes describe the geometry of visual elements. Styling (color, opacity)
/// is applied separately via [`Paint`](crate::style::Paint) when drawing.
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Shape {
    /// A rectangle with rounded corners.
    RoundedRect { rect: Rect, corner_radius: f64 },
    /// A filled circle.
    Circle { center: Point, radius: f64 },
    /// A filled ellipse.
    Ellipse { rect: Rect },
}

impl Shape {
    /// Create a rounded rectangle shape.
    pub const fn rounded_rect(rect: Rect, corner_radius: f64) -> Self {
        Self::RoundedRect {
            rect,
            corner_radius,
        }
    }

    /// Create a rounded rectangle from position and size.
    pub const fn rounded_rect_xywh(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        corner_radius: f64,
    ) -> Self {
        Self::RoundedRect {
            rect: Rect::from_xywh(x, y, width, height),
            corner_radius,
        }
    }

    /// Create a circle shape.
    pub const fn circle(center: Point, radius: f64) -> Self {
        Self::Circle { center, radius }
    }

    /// Create a circle at x, y coordinates.
    pub const fn circle_at(x: f64, y: f64, radius: f64) -> Self {
        Self::Circle {
            center: Point::new(x, y),
            radius,
        }
    }

    /// Create an ellipse shape.
    pub const fn ellipse(rect: Rect) -> Self {
        Self::Ellipse { rect }
    }

    /// Create an ellipse from position and size.
    pub const fn ellipse_xywh(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self::Ellipse {
            rect: Rect::from_xywh(x, y, width, height),
        }
    }

    /// Get the bounding rect of this shape.
    pub fn bounds(&self) -> Rect {
        match self {
            Self::RoundedRect { rect, .. } => *rect,
            Self::Circle { center, radius } => {
                Rect::centered(*center, crate::geometry::Size::square(*radius * 2.0))
            }
            Self::Ellipse { rect } => *rect,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_rounded_rect() {
        let shape = Shape::rounded_rect(Rect::from_xywh(0.0, 0.0, 100.0, 50.0), 10.0);
        match shape {
            Shape::RoundedRect {
                rect,
                corner_radius,
            } => {
                assert_eq!(rect.size.width, 100.0);
                assert_eq!(corner_radius, 10.0);
            }
            _ => panic!("Expected RoundedRect"),
        }
    }

    #[test]
    fn test_shape_circle() {
        let shape = Shape::circle(Point::new(50.0, 50.0), 25.0);
        match shape {
            Shape::Circle { center, radius } => {
                assert_eq!(center.x, 50.0);
                assert_eq!(radius, 25.0);
            }
            _ => panic!("Expected Circle"),
        }
    }

    #[test]
    fn test_shape_circle_at() {
        let shape = Shape::circle_at(10.0, 20.0, 5.0);
        match shape {
            Shape::Circle { center, radius } => {
                assert_eq!(center.x, 10.0);
                assert_eq!(center.y, 20.0);
                assert_eq!(radius, 5.0);
            }
            _ => panic!("Expected Circle"),
        }
    }

    #[test]
    fn test_bounds() {
        let rect = Shape::rounded_rect_xywh(10.0, 20.0, 100.0, 50.0, 5.0);
        assert_eq!(rect.bounds().origin.x, 10.0);
        assert_eq!(rect.bounds().size.width, 100.0);

        let circle = Shape::circle_at(50.0, 50.0, 25.0);
        let bounds = circle.bounds();
        assert_eq!(bounds.origin.x, 25.0); // center.x - radius
        assert_eq!(bounds.size.width, 50.0); // diameter
    }
}
