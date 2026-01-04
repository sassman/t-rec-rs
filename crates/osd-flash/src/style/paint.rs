//! Paint specification for how to render shapes.

use crate::Color;

/// Paint defines how a shape should be rendered.
///
/// Currently supports solid color fills with optional opacity.
/// Can be extended later with stroke, gradients, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Paint {
    /// Fill color.
    pub color: Color,
    /// Overall opacity (0.0 to 1.0).
    pub opacity: f64,
}

impl Paint {
    /// Create a new paint with color and full opacity.
    pub const fn fill(color: Color) -> Self {
        Self {
            color,
            opacity: 1.0,
        }
    }

    /// Create a paint with specified color and opacity.
    pub const fn with_opacity(color: Color, opacity: f64) -> Self {
        Self { color, opacity }
    }

    /// Set the opacity of this paint.
    pub const fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    /// Set the color of this paint.
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Get the effective color with opacity applied.
    pub fn effective_color(&self) -> Color {
        self.color.with_alpha(self.color.a * self.opacity)
    }
}

impl Default for Paint {
    fn default() -> Self {
        Self::fill(Color::BLACK)
    }
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Self::fill(color)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let p = Paint::fill(Color::RED);
        assert_eq!(p.color, Color::RED);
        assert_eq!(p.opacity, 1.0);
    }

    #[test]
    fn test_with_opacity() {
        let p = Paint::with_opacity(Color::BLUE, 0.5);
        assert_eq!(p.color, Color::BLUE);
        assert_eq!(p.opacity, 0.5);
    }

    #[test]
    fn test_builder() {
        let p = Paint::fill(Color::GREEN).opacity(0.7).color(Color::YELLOW);
        assert_eq!(p.color, Color::YELLOW);
        assert_eq!(p.opacity, 0.7);
    }

    #[test]
    fn test_effective_color() {
        let p = Paint::with_opacity(Color::rgba(1.0, 0.0, 0.0, 0.8), 0.5);
        let c = p.effective_color();
        assert!((c.a - 0.4).abs() < f64::EPSILON); // 0.8 * 0.5
    }

    #[test]
    fn test_from_color() {
        let p: Paint = Color::WHITE.into();
        assert_eq!(p.color, Color::WHITE);
        assert_eq!(p.opacity, 1.0);
    }
}
