//! Color types for the screen flash module.
//!
//! Re-exports `Color` from `core-animation` and adds osd-flash specific color presets.

// Re-export the base Color type from core-animation
pub use core_animation::Color;

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
    fn test_core_animation_color() {
        // Test that core-animation Color works
        let c = Color::rgba(0.5, 0.25, 0.75, 0.5);
        assert!((c.r - 0.5).abs() < f64::EPSILON);
        assert!((c.g - 0.25).abs() < f64::EPSILON);
        assert!((c.b - 0.75).abs() < f64::EPSILON);
        assert!((c.a - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_osd_colors() {
        // Test osd-flash specific colors
        assert!((VIBRANT_BLUE.r - 0.15).abs() < f64::EPSILON);
        assert!((VIBRANT_BLUE.a - 0.92).abs() < f64::EPSILON);
    }

    #[test]
    fn test_preset_colors() {
        assert_eq!(Color::BLACK, Color::rgb(0.0, 0.0, 0.0));
        assert_eq!(Color::WHITE, Color::rgb(1.0, 1.0, 1.0));
        assert_eq!(Color::RED, Color::rgb(1.0, 0.0, 0.0));
    }
}
