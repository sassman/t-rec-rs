//! Recording indicator icon.

use crate::color::Color;
use crate::icon::{Icon, IconBuilder};

/// A recording indicator icon.
///
/// Creates a classic recording dot icon with:
/// - Dark semi-transparent background
/// - Red glow effect
/// - Pulsing red recording dot
/// - Highlight reflection
///
/// # Example
///
/// ```ignore
/// use osd_flash::icon::RecordingIcon;
///
/// let icon = RecordingIcon::new(80.0).build();
/// ```
#[derive(Debug, Clone)]
pub struct RecordingIcon {
    size: f64,
    padding: f64,
    corner_radius: f64,
    background_color: Color,
    glow_color: Color,
    dot_color: Color,
    highlight_color: Color,
}

impl RecordingIcon {
    /// Create a new recording icon builder with the given size.
    pub fn new(size: f64) -> Self {
        Self {
            size,
            padding: 10.0,
            corner_radius: 14.0,
            background_color: Color::rgba(0.1, 0.1, 0.1, 0.85),
            glow_color: Color::rgba(1.0, 0.2, 0.2, 0.4),
            dot_color: Color::rgba(1.0, 0.15, 0.15, 1.0),
            highlight_color: Color::rgba(1.0, 0.5, 0.5, 0.6),
        }
    }

    /// Set the padding around the icon.
    pub fn padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    /// Set the corner radius of the background.
    pub fn corner_radius(mut self, radius: f64) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Set the background color.
    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    /// Set the outer glow color.
    pub fn glow_color(mut self, color: Color) -> Self {
        self.glow_color = color;
        self
    }

    /// Set the main recording dot color.
    pub fn dot_color(mut self, color: Color) -> Self {
        self.dot_color = color;
        self
    }

    /// Set the highlight color.
    pub fn highlight_color(mut self, color: Color) -> Self {
        self.highlight_color = color;
        self
    }

    /// Build the recording icon.
    pub fn build(self) -> Icon {
        let size = self.size;
        let center = size / 2.0;

        IconBuilder::new(size)
            .padding(self.padding)
            // Dark semi-transparent background
            .background(self.background_color, self.corner_radius)
            // Outer red glow
            .circle(center, center, size * 0.28, self.glow_color)
            // Main red recording dot
            .circle(center, center, size * 0.2, self.dot_color)
            // Highlight
            .circle(
                center - size * 0.06,
                center - size * 0.06,
                size * 0.06,
                self.highlight_color,
            )
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let icon = RecordingIcon::new(80.0).build();
        assert_eq!(icon.size, 80.0);
        // Recording icon has: background + glow + dot + highlight = 4 shapes
        assert_eq!(icon.shapes.len(), 4);
    }

    #[test]
    fn test_custom_colors() {
        let icon = RecordingIcon::new(100.0)
            .dot_color(Color::GREEN)
            .glow_color(Color::rgba(0.0, 1.0, 0.0, 0.4))
            .build();
        assert_eq!(icon.size, 100.0);
        assert_eq!(icon.shapes.len(), 4);
    }

    #[test]
    fn test_custom_padding() {
        let icon = RecordingIcon::new(100.0).padding(20.0).build();
        assert_eq!(icon.shapes.len(), 4);
    }
}
