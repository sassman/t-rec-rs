//! 2D size type.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square() {
        let s = Size::square(100.0);
        assert_eq!(s.width, 100.0);
        assert_eq!(s.height, 100.0);
    }

    #[test]
    fn test_scale() {
        let s = Size::new(100.0, 50.0).scale(2.0);
        assert_eq!(s.width, 200.0);
        assert_eq!(s.height, 100.0);
    }
}
