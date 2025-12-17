//! Color types for the screen flash module.
//!
//! Provides a simple RGBA color type with common color presets.

/// An RGBA color with components in the range 0.0 to 1.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    /// Create a new color from RGBA components (0.0 to 1.0).
    pub const fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    /// Create a new opaque color from RGB components (0.0 to 1.0).
    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// Create a color from 8-bit RGBA components (0 to 255).
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: a as f64 / 255.0,
        }
    }

    /// Create an opaque color from 8-bit RGB components (0 to 255).
    pub fn rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::rgba8(r, g, b, 255)
    }

    /// Create a color from a hex string (e.g., "#FF0000" or "FF0000").
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        let len = hex.len();

        match len {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::rgb8(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::rgba8(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Create a grayscale color with the given intensity (0.0 to 1.0).
    pub const fn gray(intensity: f64) -> Self {
        Self::rgb(intensity, intensity, intensity)
    }

    /// Create a grayscale color with alpha.
    pub const fn gray_alpha(intensity: f64, alpha: f64) -> Self {
        Self::rgba(intensity, intensity, intensity, alpha)
    }

    /// Set the alpha component.
    pub const fn with_alpha(mut self, alpha: f64) -> Self {
        self.a = alpha;
        self
    }

    /// Transparent (fully transparent black).
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);

    /// Black.
    pub const BLACK: Self = Self::rgb(0.0, 0.0, 0.0);

    /// White.
    pub const WHITE: Self = Self::rgb(1.0, 1.0, 1.0);

    /// Red.
    pub const RED: Self = Self::rgb(1.0, 0.0, 0.0);

    /// Green.
    pub const GREEN: Self = Self::rgb(0.0, 1.0, 0.0);

    /// Blue.
    pub const BLUE: Self = Self::rgb(0.0, 0.0, 1.0);

    /// Yellow.
    pub const YELLOW: Self = Self::rgb(1.0, 1.0, 0.0);

    /// Vibrant blue (used for camera icon background).
    pub const VIBRANT_BLUE: Self = Self::rgba(0.15, 0.45, 0.9, 0.92);

    /// Dark gray (used for lens).
    pub const DARK_GRAY: Self = Self::rgb(0.2, 0.2, 0.25);

    /// Light blue (used for lens reflection).
    pub const LIGHT_BLUE: Self = Self::rgb(0.3, 0.5, 0.8);

    /// Warm yellow (used for flash indicator).
    pub const WARM_YELLOW: Self = Self::rgb(1.0, 0.85, 0.2);
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgba() {
        let c = Color::rgba(0.5, 0.25, 0.75, 0.5);
        assert!((c.r - 0.5).abs() < f64::EPSILON);
        assert!((c.g - 0.25).abs() < f64::EPSILON);
        assert!((c.b - 0.75).abs() < f64::EPSILON);
        assert!((c.a - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_rgb8() {
        let c = Color::rgb8(255, 128, 0);
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.5).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);
        assert!((c.a - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_from_hex() {
        let c = Color::from_hex("#FF8000").unwrap();
        assert!((c.r - 1.0).abs() < 0.01);
        assert!((c.g - 0.5).abs() < 0.01);
        assert!((c.b - 0.0).abs() < 0.01);

        let c = Color::from_hex("FF800080").unwrap();
        assert!((c.a - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_from_hex_invalid() {
        assert!(Color::from_hex("invalid").is_none());
        assert!(Color::from_hex("#FFF").is_none());
    }

    #[test]
    fn test_gray() {
        let c = Color::gray(0.5);
        assert_eq!(c.r, c.g);
        assert_eq!(c.g, c.b);
        assert!((c.r - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_with_alpha() {
        let c = Color::WHITE.with_alpha(0.5);
        assert!((c.a - 0.5).abs() < f64::EPSILON);
        assert!((c.r - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_preset_colors() {
        assert_eq!(Color::BLACK, Color::rgb(0.0, 0.0, 0.0));
        assert_eq!(Color::WHITE, Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(Color::RED, Color::rgb(1.0, 0.0, 0.0));
    }
}
