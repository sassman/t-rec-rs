//! Position types for OSD window placement.

use crate::geometry::Point;

/// Position for OSD window relative to screen.
///
/// Determines where the OSD window is placed on the screen.
/// Each position can be combined with a margin to add spacing from screen edges.
///
/// # Examples
///
/// ```ignore
/// use osd_flash::prelude::*;
///
/// OsdBuilder::new()
///     .position(Position::TopRight)
///     .margin(20.0)  // 20px from screen edge
///     .show_for(3.seconds())?;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Position {
    /// Top-right corner of the screen (default).
    #[default]
    TopRight,
    /// Top-left corner of the screen.
    TopLeft,
    /// Bottom-right corner of the screen.
    BottomRight,
    /// Bottom-left corner of the screen.
    BottomLeft,
    /// Center of the screen.
    Center,
    /// Custom position with absolute coordinates.
    Custom { x: f64, y: f64 },
}

impl Position {
    /// Create a custom position from coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position in screen coordinates
    /// * `y` - Vertical position in screen coordinates
    pub const fn custom(x: f64, y: f64) -> Self {
        Self::Custom { x, y }
    }

    /// Create a custom position from a Point.
    pub const fn from_point(point: Point) -> Self {
        Self::Custom {
            x: point.x,
            y: point.y,
        }
    }
}

impl From<Point> for Position {
    fn from(point: Point) -> Self {
        Self::from_point(point)
    }
}

impl From<(f64, f64)> for Position {
    fn from((x, y): (f64, f64)) -> Self {
        Self::custom(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        assert_eq!(Position::default(), Position::TopRight);
    }

    #[test]
    fn test_custom() {
        let pos = Position::custom(100.0, 200.0);
        if let Position::Custom { x, y } = pos {
            assert_eq!(x, 100.0);
            assert_eq!(y, 200.0);
        } else {
            panic!("Expected Custom position");
        }
    }

    #[test]
    fn test_from_point() {
        let point = Point::new(50.0, 75.0);
        let pos: Position = point.into();
        if let Position::Custom { x, y } = pos {
            assert_eq!(x, 50.0);
            assert_eq!(y, 75.0);
        } else {
            panic!("Expected Custom position");
        }
    }

    #[test]
    fn test_from_tuple() {
        let pos: Position = (150.0, 250.0).into();
        if let Position::Custom { x, y } = pos {
            assert_eq!(x, 150.0);
            assert_eq!(y, 250.0);
        } else {
            panic!("Expected Custom position");
        }
    }
}
