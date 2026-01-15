//! Color types for the OSD flash module.
//!
//! Provides a platform-agnostic Color type for use across all platforms.

/// RGBA color with components in the range 0.0-1.0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    /// Red component (0.0-1.0).
    pub r: f64,
    /// Green component (0.0-1.0).
    pub g: f64,
    /// Blue component (0.0-1.0).
    pub b: f64,
    /// Alpha component (0.0-1.0), where 1.0 is fully opaque.
    pub a: f64,
}

impl Color {
    // =========================================================================
    // Preset colors
    // =========================================================================

    /// Fully transparent.
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
    /// Cyan.
    pub const CYAN: Self = Self::rgb(0.0, 1.0, 1.0);
    /// Magenta.
    pub const MAGENTA: Self = Self::rgb(1.0, 0.0, 1.0);
    /// Yellow.
    pub const YELLOW: Self = Self::rgb(1.0, 1.0, 0.0);

    // =========================================================================
    // Constructors
    // =========================================================================

    /// Create a color with RGBA components (0.0-1.0).
    pub const fn rgba(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }

    /// Create a fully opaque color with RGB components (0.0-1.0).
    pub const fn rgb(r: f64, g: f64, b: f64) -> Self {
        Self::rgba(r, g, b, 1.0)
    }

    /// Create a grayscale color (0.0 = black, 1.0 = white).
    pub const fn gray(value: f64) -> Self {
        Self::rgb(value, value, value)
    }

    /// Create a grayscale color with alpha.
    pub const fn gray_alpha(value: f64, alpha: f64) -> Self {
        Self::rgba(value, value, value, alpha)
    }

    /// Create a color from a hex string (e.g., "#ff5500" or "ff5500").
    ///
    /// Returns `None` if the string is not a valid hex color.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Self::rgb(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
        ))
    }

    // =========================================================================
    // Modifiers
    // =========================================================================

    /// Return a new color with the specified alpha value.
    pub const fn with_alpha(self, alpha: f64) -> Self {
        Self::rgba(self.r, self.g, self.b, alpha)
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BLACK
    }
}

// OSD-flash specific color presets as standalone constants
// These are library exports for future compositions.

/// Vibrant blue (used for camera icon background).
#[allow(dead_code)]
pub const VIBRANT_BLUE: Color = Color::rgba(0.15, 0.45, 0.9, 0.92);

/// Light blue (used for lens reflection).
#[allow(dead_code)]
pub const LIGHT_BLUE: Color = Color::rgb(0.3, 0.5, 0.8);

/// Warm yellow (used for flash indicator).
#[allow(dead_code)]
pub const WARM_YELLOW: Color = Color::rgb(1.0, 0.85, 0.2);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_rgba() {
        let c = Color::rgba(0.5, 0.25, 0.75, 0.5);
        assert!((c.r - 0.5).abs() < f64::EPSILON);
        assert!((c.g - 0.25).abs() < f64::EPSILON);
        assert!((c.b - 0.75).abs() < f64::EPSILON);
        assert!((c.a - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_color_rgb() {
        let c = Color::rgb(0.5, 0.25, 0.75);
        assert!((c.a - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_osd_colors() {
        assert!((VIBRANT_BLUE.r - 0.15).abs() < f64::EPSILON);
        assert!((VIBRANT_BLUE.a - 0.92).abs() < f64::EPSILON);
    }

    #[test]
    fn test_preset_colors() {
        assert_eq!(Color::BLACK, Color::rgb(0.0, 0.0, 0.0));
        assert_eq!(Color::WHITE, Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(Color::RED, Color::rgb(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_with_alpha() {
        let c = Color::RED.with_alpha(0.5);
        assert!((c.r - 1.0).abs() < f64::EPSILON);
        assert!((c.a - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_gray() {
        let c = Color::gray(0.5);
        assert!((c.r - 0.5).abs() < f64::EPSILON);
        assert!((c.g - 0.5).abs() < f64::EPSILON);
        assert!((c.b - 0.5).abs() < f64::EPSILON);
    }
}
