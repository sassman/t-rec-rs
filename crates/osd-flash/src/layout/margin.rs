//! Margin type for positioning elements.

/// Margin for positioning elements, similar to CSS margin.
///
/// Supports multiple construction patterns:
/// - Single value: `Margin::all(20.0)` or `20.0.into()`
/// - Vertical/horizontal: `Margin::symmetric(10.0, 20.0)`
/// - Individual sides: `Margin::new(10.0, 20.0, 10.0, 20.0)`
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Margin {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

impl Margin {
    /// Create a margin with individual values for each side.
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

    /// Create a margin with the same value on all sides.
    pub const fn all(value: f64) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    /// Create a margin with symmetric vertical and horizontal values.
    ///
    /// - `vertical`: top and bottom margin
    /// - `horizontal`: left and right margin
    pub const fn symmetric(vertical: f64, horizontal: f64) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    /// Create a zero margin.
    pub const fn zero() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }

    /// Get the total horizontal margin (left + right).
    pub fn horizontal(&self) -> f64 {
        self.left + self.right
    }

    /// Get the total vertical margin (top + bottom).
    pub fn vertical(&self) -> f64 {
        self.top + self.bottom
    }
}

impl From<f64> for Margin {
    /// Create a margin with the same value on all sides.
    fn from(value: f64) -> Self {
        Self::all(value)
    }
}

impl From<(f64, f64)> for Margin {
    /// Create a margin from (vertical, horizontal) values.
    fn from((vertical, horizontal): (f64, f64)) -> Self {
        Self::symmetric(vertical, horizontal)
    }
}

impl From<(f64, f64, f64, f64)> for Margin {
    /// Create a margin from (top, right, bottom, left) values.
    fn from((top, right, bottom, left): (f64, f64, f64, f64)) -> Self {
        Self::new(top, right, bottom, left)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all() {
        let m = Margin::all(20.0);
        assert_eq!(m.top, 20.0);
        assert_eq!(m.right, 20.0);
        assert_eq!(m.bottom, 20.0);
        assert_eq!(m.left, 20.0);
    }

    #[test]
    fn test_symmetric() {
        let m = Margin::symmetric(10.0, 20.0);
        assert_eq!(m.top, 10.0);
        assert_eq!(m.right, 20.0);
        assert_eq!(m.bottom, 10.0);
        assert_eq!(m.left, 20.0);
    }

    #[test]
    fn test_new() {
        let m = Margin::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(m.top, 1.0);
        assert_eq!(m.right, 2.0);
        assert_eq!(m.bottom, 3.0);
        assert_eq!(m.left, 4.0);
    }

    #[test]
    fn test_from_f64() {
        let m: Margin = 15.0.into();
        assert_eq!(m, Margin::all(15.0));
    }

    #[test]
    fn test_from_tuple_2() {
        let m: Margin = (10.0, 20.0).into();
        assert_eq!(m, Margin::symmetric(10.0, 20.0));
    }

    #[test]
    fn test_from_tuple_4() {
        let m: Margin = (1.0, 2.0, 3.0, 4.0).into();
        assert_eq!(m, Margin::new(1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn test_horizontal_vertical() {
        let m = Margin::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(m.horizontal(), 60.0); // left + right
        assert_eq!(m.vertical(), 40.0); // top + bottom
    }
}
