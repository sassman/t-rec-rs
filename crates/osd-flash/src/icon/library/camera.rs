//! Camera icon for screenshot feedback.

use crate::color::Color;
use crate::icon::{Icon, IconBuilder};

/// A camera icon for screenshot feedback.
///
/// Creates a clean, modern camera icon with:
/// - Rounded background
/// - Camera body with viewfinder bump
/// - Lens with reflection and highlight
/// - Flash indicator
///
/// # Example
///
/// ```ignore
/// use osd_flash::icon::CameraIcon;
///
/// let icon = CameraIcon::new(120.0).build();
/// ```
#[derive(Debug, Clone)]
pub struct CameraIcon {
    size: f64,
    padding: f64,
    corner_radius: f64,
    background_color: Color,
    body_color: Color,
    lens_color: Color,
    lens_reflection_color: Color,
    flash_color: Color,
}

impl CameraIcon {
    /// Create a new camera icon builder with the given size.
    pub fn new(size: f64) -> Self {
        Self {
            size,
            padding: 12.0,
            corner_radius: 16.0,
            background_color: Color::VIBRANT_BLUE,
            body_color: Color::WHITE,
            lens_color: Color::DARK_GRAY,
            lens_reflection_color: Color::LIGHT_BLUE,
            flash_color: Color::WARM_YELLOW,
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

    /// Set the camera body color.
    pub fn body_color(mut self, color: Color) -> Self {
        self.body_color = color;
        self
    }

    /// Set the lens color.
    pub fn lens_color(mut self, color: Color) -> Self {
        self.lens_color = color;
        self
    }

    /// Set the lens reflection color.
    pub fn lens_reflection_color(mut self, color: Color) -> Self {
        self.lens_reflection_color = color;
        self
    }

    /// Set the flash indicator color.
    pub fn flash_color(mut self, color: Color) -> Self {
        self.flash_color = color;
        self
    }

    /// Build the camera icon.
    pub fn build(self) -> Icon {
        let size = self.size;
        let center_x = size / 2.0;
        let center_y = size / 2.0;

        // Camera body dimensions
        let cam_w = size * 0.55;
        let cam_h = size * 0.38;

        // Viewfinder dimensions
        let vf_w = size * 0.18;
        let vf_h = size * 0.1;

        // Lens radius
        let lens_r = size * 0.14;

        IconBuilder::new(size)
            .padding(self.padding)
            // Background rounded rectangle
            .background(self.background_color, self.corner_radius)
            // Camera body
            .rounded_rect(
                center_x - cam_w / 2.0,
                center_y - cam_h / 2.0 + 4.0,
                cam_w,
                cam_h,
                8.0,
                self.body_color,
            )
            // Viewfinder bump on top
            .rounded_rect(
                center_x - vf_w / 2.0,
                center_y - cam_h / 2.0 - vf_h + 8.0,
                vf_w,
                vf_h + 2.0,
                3.0,
                self.body_color,
            )
            // Lens outer ring
            .circle(center_x, center_y + 4.0, lens_r, self.lens_color)
            // Lens inner (reflection)
            .circle(
                center_x,
                center_y + 4.0,
                lens_r * 0.65,
                self.lens_reflection_color,
            )
            // Lens highlight dot
            .circle(
                center_x - lens_r * 0.25,
                center_y + 4.0 - lens_r * 0.25,
                lens_r * 0.2,
                Color::WHITE.with_alpha(0.8),
            )
            // Flash indicator
            .circle(
                center_x + cam_w / 2.0 - 12.0,
                center_y - cam_h / 2.0 + 14.0,
                4.0,
                self.flash_color,
            )
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let icon = CameraIcon::new(120.0).build();
        // Camera icon has: background + body + viewfinder + 3 lens parts + flash = 7 shapes
        assert_eq!(icon.shapes.len(), 7);
    }

    #[test]
    fn test_custom_colors() {
        let icon = CameraIcon::new(100.0)
            .background_color(Color::DARK_GRAY)
            .flash_color(Color::RED)
            .build();
        assert_eq!(icon.shapes.len(), 7);
    }

    #[test]
    fn test_custom_padding() {
        let icon = CameraIcon::new(100.0).padding(20.0).build();
        assert_eq!(icon.shapes.len(), 7);
    }

    #[test]
    fn test_custom_corner_radius() {
        let icon = CameraIcon::new(100.0).corner_radius(24.0).build();
        assert_eq!(icon.shapes.len(), 7);
    }
}
