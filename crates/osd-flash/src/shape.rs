//! Drawable shape primitives.
//!
//! Platform-agnostic shape definitions that can be rendered by any backend.

use crate::color::Color;
use crate::geometry::{Point, Rect};

/// A drawable shape.
///
/// Shapes are platform-agnostic descriptions of visual elements.
/// They are rendered by backend-specific canvas implementations.
#[derive(Debug, Clone)]
pub enum Shape {
    /// A rectangle with rounded corners.
    RoundedRect {
        rect: Rect,
        corner_radius: f64,
        color: Color,
    },
    /// A filled circle.
    Circle {
        center: Point,
        radius: f64,
        color: Color,
    },
    /// A filled ellipse.
    Ellipse { rect: Rect, color: Color },
    /// Text label.
    Text {
        text: String,
        position: Point,
        size: f64,
        color: Color,
    },
}

impl Shape {
    /// Create a rounded rectangle shape.
    pub fn rounded_rect(rect: Rect, corner_radius: f64, color: Color) -> Self {
        Self::RoundedRect {
            rect,
            corner_radius,
            color,
        }
    }

    /// Create a rounded rectangle from position and size.
    pub fn rounded_rect_xywh(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        corner_radius: f64,
        color: Color,
    ) -> Self {
        Self::RoundedRect {
            rect: Rect::from_xywh(x, y, width, height),
            corner_radius,
            color,
        }
    }

    /// Create a circle shape.
    pub fn circle(center: Point, radius: f64, color: Color) -> Self {
        Self::Circle {
            center,
            radius,
            color,
        }
    }

    /// Create a circle at x, y coordinates.
    pub fn circle_at(x: f64, y: f64, radius: f64, color: Color) -> Self {
        Self::Circle {
            center: Point::new(x, y),
            radius,
            color,
        }
    }

    /// Create an ellipse shape.
    pub fn ellipse(rect: Rect, color: Color) -> Self {
        Self::Ellipse { rect, color }
    }

    /// Create a text shape.
    pub fn text(text: impl Into<String>, position: Point, size: f64, color: Color) -> Self {
        Self::Text {
            text: text.into(),
            position,
            size,
            color,
        }
    }

    /// Create a text shape at x, y coordinates.
    pub fn text_at(text: impl Into<String>, x: f64, y: f64, size: f64, color: Color) -> Self {
        Self::Text {
            text: text.into(),
            position: Point::new(x, y),
            size,
            color,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_rounded_rect() {
        let shape = Shape::rounded_rect(Rect::from_xywh(0.0, 0.0, 100.0, 50.0), 10.0, Color::RED);
        match shape {
            Shape::RoundedRect {
                rect,
                corner_radius,
                color,
            } => {
                assert_eq!(rect.size.width, 100.0);
                assert_eq!(corner_radius, 10.0);
                assert_eq!(color, Color::RED);
            }
            _ => panic!("Expected RoundedRect"),
        }
    }

    #[test]
    fn test_shape_circle() {
        let shape = Shape::circle(Point::new(50.0, 50.0), 25.0, Color::BLUE);
        match shape {
            Shape::Circle {
                center,
                radius,
                color,
            } => {
                assert_eq!(center.x, 50.0);
                assert_eq!(radius, 25.0);
                assert_eq!(color, Color::BLUE);
            }
            _ => panic!("Expected Circle"),
        }
    }

    #[test]
    fn test_shape_circle_at() {
        let shape = Shape::circle_at(10.0, 20.0, 5.0, Color::GREEN);
        match shape {
            Shape::Circle { center, radius, .. } => {
                assert_eq!(center.x, 10.0);
                assert_eq!(center.y, 20.0);
                assert_eq!(radius, 5.0);
            }
            _ => panic!("Expected Circle"),
        }
    }
}
