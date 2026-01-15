//! 2D point type.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let p = Point::new(10.0, 20.0);
        assert_eq!(p.x, 10.0);
        assert_eq!(p.y, 20.0);
    }

    #[test]
    fn test_offset() {
        let p = Point::new(10.0, 20.0).offset(5.0, -5.0);
        assert_eq!(p.x, 15.0);
        assert_eq!(p.y, 15.0);
    }
}
