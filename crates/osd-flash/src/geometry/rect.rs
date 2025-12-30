//! Rectangle type.

use super::{Point, Size};

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

    /// Round all coordinates to integers to avoid subpixel rendering artifacts.
    pub fn rounded(self) -> Self {
        Self {
            origin: Point::new(self.origin.x.round(), self.origin.y.round()),
            size: Size::new(self.size.width.round(), self.size.height.round()),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_center() {
        let r = Rect::from_xywh(10.0, 20.0, 100.0, 50.0);
        let c = r.center();
        assert_eq!(c.x, 60.0);
        assert_eq!(c.y, 45.0);
    }

    #[test]
    fn test_centered() {
        let center = Point::new(50.0, 50.0);
        let r = Rect::centered(center, Size::new(20.0, 10.0));
        assert_eq!(r.origin.x, 40.0);
        assert_eq!(r.origin.y, 45.0);
    }

    #[test]
    fn test_inset() {
        let r = Rect::from_xywh(0.0, 0.0, 100.0, 100.0).inset(10.0, 10.0);
        assert_eq!(r.origin.x, 10.0);
        assert_eq!(r.origin.y, 10.0);
        assert_eq!(r.size.width, 80.0);
        assert_eq!(r.size.height, 80.0);
    }
}
