//! Core Graphics geometry type conversions.
//!
//! Provides conversions between our platform-agnostic geometry types
//! and the macOS Core Graphics types (CGPoint, CGSize, CGRect).

use core_graphics::geometry::{CGPoint, CGRect, CGSize};

use crate::geometry::{Point, Rect, Size};

// Point <-> CGPoint

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

// Size <-> CGSize

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

// Rect <-> CGRect

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
    fn test_point_cg_conversion() {
        let point = Point::new(10.0, 20.0);
        let cg_point: CGPoint = point.into();
        let back: Point = cg_point.into();
        assert_eq!(point, back);
    }

    #[test]
    fn test_size_cg_conversion() {
        let size = Size::new(100.0, 50.0);
        let cg_size: CGSize = size.into();
        let back: Size = cg_size.into();
        assert_eq!(size, back);
    }

    #[test]
    fn test_rect_cg_conversion() {
        let rect = Rect::from_xywh(10.0, 20.0, 100.0, 50.0);
        let cg_rect: CGRect = rect.into();
        let back: Rect = cg_rect.into();
        assert_eq!(rect, back);
    }
}
