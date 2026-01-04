//! Border type for element borders.

use crate::Color;

/// Border specification for elements.
///
/// Border is a layout concern - its width affects content bounds.
/// A border of width 2.0 means content is inset by 2.0 on each side.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Border {
    /// Width of the border in points.
    pub width: f64,
    /// Color of the border.
    pub color: Color,
    /// Corner radius for rounded borders.
    pub radius: f64,
}

impl Border {
    /// Create a new border with specified width, color, and corner radius.
    pub const fn new(width: f64, color: Color, radius: f64) -> Self {
        Self {
            width,
            color,
            radius,
        }
    }

    /// Create a simple border with width and color, no rounding.
    pub const fn solid(width: f64, color: Color) -> Self {
        Self {
            width,
            color,
            radius: 0.0,
        }
    }

    /// Create a rounded border with width, color, and corner radius.
    pub const fn rounded(width: f64, color: Color, radius: f64) -> Self {
        Self {
            width,
            color,
            radius,
        }
    }

    /// Create a border with no width (invisible border).
    pub const fn none() -> Self {
        Self {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: 0.0,
        }
    }

    /// Check if this border is visible (has width > 0).
    pub fn is_visible(&self) -> bool {
        self.width > 0.0 && self.color.a > 0.0
    }

    /// Get the total horizontal space taken by the border (left + right).
    pub fn horizontal(&self) -> f64 {
        self.width * 2.0
    }

    /// Get the total vertical space taken by the border (top + bottom).
    pub fn vertical(&self) -> f64 {
        self.width * 2.0
    }

    /// Create a new border with a different width.
    pub const fn with_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Create a new border with a different color.
    pub const fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Create a new border with a different corner radius.
    pub const fn with_radius(mut self, radius: f64) -> Self {
        self.radius = radius;
        self
    }
}

impl Default for Border {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let b = Border::new(2.0, Color::RED, 8.0);
        assert_eq!(b.width, 2.0);
        assert_eq!(b.color, Color::RED);
        assert_eq!(b.radius, 8.0);
    }

    #[test]
    fn test_solid() {
        let b = Border::solid(1.0, Color::BLACK);
        assert_eq!(b.width, 1.0);
        assert_eq!(b.radius, 0.0);
    }

    #[test]
    fn test_rounded() {
        let b = Border::rounded(2.0, Color::WHITE, 12.0);
        assert_eq!(b.width, 2.0);
        assert_eq!(b.radius, 12.0);
    }

    #[test]
    fn test_none() {
        let b = Border::none();
        assert_eq!(b.width, 0.0);
        assert!(!b.is_visible());
    }

    #[test]
    fn test_is_visible() {
        assert!(Border::solid(1.0, Color::BLACK).is_visible());
        assert!(!Border::solid(0.0, Color::BLACK).is_visible());
        assert!(!Border::solid(1.0, Color::TRANSPARENT).is_visible());
    }

    #[test]
    fn test_horizontal_vertical() {
        let b = Border::solid(3.0, Color::BLACK);
        assert_eq!(b.horizontal(), 6.0);
        assert_eq!(b.vertical(), 6.0);
    }

    #[test]
    fn test_with_methods() {
        let b = Border::none()
            .with_width(2.0)
            .with_color(Color::BLUE)
            .with_radius(10.0);
        assert_eq!(b.width, 2.0);
        assert_eq!(b.color, Color::BLUE);
        assert_eq!(b.radius, 10.0);
    }
}
