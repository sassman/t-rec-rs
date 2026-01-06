//! Transform types for animation.
//!
//! Transforms modify how the icon is rendered during animation,
//! allowing for effects like scaling, rotation, and translation.

/// Transformations applied to the entire icon during animation.
///
/// Currently supports scaling; additional transforms (rotate, translate, skew)
/// can be added in the future.
#[derive(Clone, Debug, PartialEq)]
pub struct Transform {
    /// Scale factor (1.0 = original size, 0.5 = half size, 2.0 = double size).
    pub scale: f64,
}

impl Transform {
    /// Create a new transform with the given scale.
    pub fn new(scale: f64) -> Self {
        Self { scale }
    }

    /// Create an identity transform (no change).
    pub fn identity() -> Self {
        Self::default()
    }

    /// Check if this is an identity transform (no effect).
    pub fn is_identity(&self) -> bool {
        (self.scale - 1.0).abs() < f64::EPSILON
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let t = Transform::default();
        assert!((t.scale - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_new() {
        let t = Transform::new(0.5);
        assert!((t.scale - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_identity() {
        let t = Transform::identity();
        assert!(t.is_identity());
    }

    #[test]
    fn test_is_identity() {
        assert!(Transform::new(1.0).is_identity());
        assert!(!Transform::new(0.95).is_identity());
        assert!(!Transform::new(1.05).is_identity());
    }
}
