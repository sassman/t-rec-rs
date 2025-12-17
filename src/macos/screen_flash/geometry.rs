//! Geometry types for the screen flash module.
//!
//! Provides simple, ergonomic types for working with positions, sizes, and rectangles.

use core_graphics::geometry::{CGPoint, CGRect, CGSize};

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    /// Create a new point.
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Create a point at the origin (0, 0).
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Offset the point by the given amounts.
    pub fn offset(self, dx: f64, dy: f64) -> Self {
        Self {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

impl From<Point> for CGPoint {
    fn from(p: Point) -> Self {
        CGPoint { x: p.x, y: p.y }
    }
}

impl From<CGPoint> for Point {
    fn from(p: CGPoint) -> Self {
        Self { x: p.x, y: p.y }
    }
}

/// A 2D size.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

impl Size {
    /// Create a new size.
    pub const fn new(width: f64, height: f64) -> Self {
        Self { width, height }
    }

    /// Create a square size.
    pub const fn square(side: f64) -> Self {
        Self {
            width: side,
            height: side,
        }
    }

    /// Create a zero size.
    pub const fn zero() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }

    /// Scale the size by a factor.
    pub fn scale(self, factor: f64) -> Self {
        Self {
            width: self.width * factor,
            height: self.height * factor,
        }
    }
}

impl From<Size> for CGSize {
    fn from(s: Size) -> Self {
        CGSize {
            width: s.width,
            height: s.height,
        }
    }
}

impl From<CGSize> for Size {
    fn from(s: CGSize) -> Self {
        Self {
            width: s.width,
            height: s.height,
        }
    }
}

/// A rectangle defined by origin and size.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    /// Create a new rectangle.
    pub const fn new(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    /// Create a rectangle from x, y, width, height.
    pub const fn from_xywh(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    /// Create a square rectangle centered at a point.
    pub fn square_centered(center: Point, side: f64) -> Self {
        Self {
            origin: Point::new(center.x - side / 2.0, center.y - side / 2.0),
            size: Size::square(side),
        }
    }

    /// Create a rectangle centered at a point.
    pub fn centered(center: Point, size: Size) -> Self {
        Self {
            origin: Point::new(center.x - size.width / 2.0, center.y - size.height / 2.0),
            size,
        }
    }

    /// Get the center point of the rectangle.
    pub fn center(&self) -> Point {
        Point::new(
            self.origin.x + self.size.width / 2.0,
            self.origin.y + self.size.height / 2.0,
        )
    }

    /// Get the minimum X coordinate.
    pub fn min_x(&self) -> f64 {
        self.origin.x
    }

    /// Get the maximum X coordinate.
    pub fn max_x(&self) -> f64 {
        self.origin.x + self.size.width
    }

    /// Get the minimum Y coordinate.
    pub fn min_y(&self) -> f64 {
        self.origin.y
    }

    /// Get the maximum Y coordinate.
    pub fn max_y(&self) -> f64 {
        self.origin.y + self.size.height
    }

    /// Inset the rectangle by the given amounts.
    pub fn inset(self, dx: f64, dy: f64) -> Self {
        Self {
            origin: Point::new(self.origin.x + dx, self.origin.y + dy),
            size: Size::new(self.size.width - 2.0 * dx, self.size.height - 2.0 * dy),
        }
    }

    /// Offset the rectangle by the given amounts.
    pub fn offset(self, dx: f64, dy: f64) -> Self {
        Self {
            origin: self.origin.offset(dx, dy),
            size: self.size,
        }
    }
}

impl From<Rect> for CGRect {
    fn from(r: Rect) -> Self {
        CGRect {
            origin: r.origin.into(),
            size: r.size.into(),
        }
    }
}

impl From<CGRect> for Rect {
    fn from(r: CGRect) -> Self {
        Self {
            origin: r.origin.into(),
            size: r.size.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_new() {
        let p = Point::new(10.0, 20.0);
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
    }

    #[test]
    fn test_point_offset() {
        let p = Point::new(10.0, 20.0).offset(5.0, -5.0);
        assert_eq!(p.x, 15.0);
        assert_eq!(p.y, 15.0);
    }

    #[test]
    fn test_size_square() {
        let s = Size::square(100.0);
        assert_eq!(s.width, 100.0);
        assert_eq!(s.height, 100.0);
    }

    #[test]
    fn test_size_scale() {
        let s = Size::new(100.0, 50.0).scale(2.0);
        assert_eq!(s.width, 200.0);
        assert_eq!(s.height, 100.0);
    }

    #[test]
    fn test_rect_center() {
        let r = Rect::from_xywh(10.0, 20.0, 100.0, 50.0);
        let c = r.center();
        assert_eq!(c.x, 60.0);
        assert_eq!(c.y, 45.0);
    }

    #[test]
    fn test_rect_centered() {
        let center = Point::new(50.0, 50.0);
        let r = Rect::centered(center, Size::new(20.0, 10.0));
        assert_eq!(r.origin.x, 40.0);
        assert_eq!(r.origin.y, 45.0);
    }

    #[test]
    fn test_rect_inset() {
        let r = Rect::from_xywh(0.0, 0.0, 100.0, 100.0).inset(10.0, 10.0);
        assert_eq!(r.origin.x, 10.0);
        assert_eq!(r.origin.y, 10.0);
        assert_eq!(r.size.width, 80.0);
        assert_eq!(r.size.height, 80.0);
    }

    #[test]
    fn test_cg_conversions() {
        let rect = Rect::from_xywh(10.0, 20.0, 100.0, 50.0);
        let cg_rect: CGRect = rect.into();
        let back: Rect = cg_rect.into();
        assert_eq!(rect, back);
    }
}
