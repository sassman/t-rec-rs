//! Padding type for internal spacing.

use crate::geometry::Rect;
use crate::shape::Shape;

/// Padding for internal element spacing, similar to CSS padding.
///
/// Supports multiple construction patterns:
/// - Single value: `Padding::all(20.0)` or `20.0.into()`
/// - Vertical/horizontal: `Padding::symmetric(10.0, 20.0)`
/// - Individual sides: `Padding::new(10.0, 20.0, 10.0, 20.0)`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Padding {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Padding {
    /// Create padding with individual values for each side.
    ///
    /// Order follows CSS convention: top, right, bottom, left.
    pub const fn new(top: f64, right: f64, bottom: f64, left: f64) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    /// Create padding with the same value on all sides.
    pub const fn all(value: f64) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create padding with symmetric vertical and horizontal values.
    ///
    /// - `vertical`: top and bottom padding
    /// - `horizontal`: left and right padding
    pub const fn symmetric(vertical: f64, horizontal: f64) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create zero padding.
    pub const fn zero() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }

    /// Get the total horizontal padding (left + right).
    pub fn horizontal(&self) -> f64 {
        self.left + self.right
    }

    /// Get the total vertical padding (top + bottom).
    pub fn vertical(&self) -> f64 {
        self.top + self.bottom
    }

    /// Expand a shape by this padding amount.
    ///
    /// The shape's bounds are extended outward by the padding on each side.
    /// The shape's background/fill will cover the entire padded area.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // A 120x120 rect expanded by 40px horizontal padding becomes 200x120
    /// let padded = Padding::symmetric(0.0, 40.0).around(
    ///     Shape::rounded_rect(Rect::from_xywh(0.0, 0.0, 120.0, 120.0), 16.0)
    /// );
    /// ```
    pub fn around(&self, shape: Shape) -> Shape {
        match shape {
            Shape::RoundedRect { rect, corner_radius } => {
                let expanded = Rect::from_xywh(
                    rect.origin.x - self.left,
                    rect.origin.y - self.top,
                    rect.size.width + self.horizontal(),
                    rect.size.height + self.vertical(),
                );
                Shape::RoundedRect {
                    rect: expanded,
                    corner_radius,
                }
            }
            Shape::Circle { center, radius } => {
                // For circles with symmetric padding, expand the radius
                // For asymmetric, convert to ellipse
                if (self.left - self.right).abs() < f64::EPSILON
                    && (self.top - self.bottom).abs() < f64::EPSILON
                {
                    // Symmetric - expand radius
                    Shape::Circle {
                        center,
                        radius: radius + self.left, // left == right in symmetric case
                    }
                } else {
                    // Asymmetric - convert to ellipse
                    let bounds = Rect::from_xywh(
                        center.x - radius - self.left,
                        center.y - radius - self.top,
                        radius * 2.0 + self.horizontal(),
                        radius * 2.0 + self.vertical(),
                    );
                    Shape::Ellipse { rect: bounds }
                }
            }
            Shape::Ellipse { rect } => {
                let expanded = Rect::from_xywh(
                    rect.origin.x - self.left,
                    rect.origin.y - self.top,
                    rect.size.width + self.horizontal(),
                    rect.size.height + self.vertical(),
                );
                Shape::Ellipse { rect: expanded }
            }
        }
    }
}

impl From<f64> for Padding {
    /// Create padding with the same value on all sides.
    fn from(value: f64) -> Self {
        Self::all(value)
    }
}

impl From<(f64, f64)> for Padding {
    /// Create padding from (vertical, horizontal) values.
    fn from((vertical, horizontal): (f64, f64)) -> Self {
        Self::symmetric(vertical, horizontal)
    }
}

impl From<(f64, f64, f64, f64)> for Padding {
    /// Create padding from (top, right, bottom, left) values.
    fn from((top, right, bottom, left): (f64, f64, f64, f64)) -> Self {
        Self::new(top, right, bottom, left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all() {
        let p = Padding::all(20.0);
        assert_eq!(p.top, 20.0);
        assert_eq!(p.right, 20.0);
        assert_eq!(p.bottom, 20.0);
        assert_eq!(p.left, 20.0);
    }

    #[test]
    fn test_symmetric() {
        let p = Padding::symmetric(10.0, 20.0);
        assert_eq!(p.top, 10.0);
        assert_eq!(p.right, 20.0);
        assert_eq!(p.bottom, 10.0);
        assert_eq!(p.left, 20.0);
    }

    #[test]
    fn test_new() {
        let p = Padding::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(p.top, 1.0);
        assert_eq!(p.right, 2.0);
        assert_eq!(p.bottom, 3.0);
        assert_eq!(p.left, 4.0);
    }

    #[test]
    fn test_from_f64() {
        let p: Padding = 15.0.into();
        assert_eq!(p, Padding::all(15.0));
    }

    #[test]
    fn test_from_tuple_2() {
        let p: Padding = (10.0, 20.0).into();
        assert_eq!(p, Padding::symmetric(10.0, 20.0));
    }

    #[test]
    fn test_from_tuple_4() {
        let p: Padding = (1.0, 2.0, 3.0, 4.0).into();
        assert_eq!(p, Padding::new(1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn test_horizontal_vertical() {
        let p = Padding::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(p.horizontal(), 60.0); // left + right
        assert_eq!(p.vertical(), 40.0); // top + bottom
    }
}
