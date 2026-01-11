//! Camera flash composition.
//!
//! A camera icon indicator commonly used for screenshot feedback.

use crate::color::Color;
use crate::composition::{CompositionBuilder, LayerComposition};

/// A camera flash indicator.
///
/// Displays a camera icon, commonly used as visual feedback when
/// a screenshot is taken (similar to macOS screenshot indicator).
///
/// # Examples
///
/// ```ignore
/// use osd_flash::prelude::*;
///
/// // Default camera flash
/// OsdBuilder::new()
///     .composition(CameraFlash::new())
///     .show_for(1500.millis())?;
///
/// // Customized camera flash
/// OsdBuilder::new()
///     .composition(
///         CameraFlash::new()
///             .size(150.0)
///             .background_color(Color::rgba(0.2, 0.5, 0.8, 0.95))
///     )
///     .show_for(2.seconds())?;
/// ```
#[derive(Debug, Clone)]
pub struct CameraFlash {
    size: f64,
    background_color: Color,
    body_color: Color,
    lens_outer_color: Color,
    lens_inner_color: Color,
    lens_highlight_color: Color,
    flash_color: Color,
    corner_radius: f64,
}

impl CameraFlash {
    /// Create a new camera flash indicator with default settings.
    ///
    /// Defaults:
    /// - Size: 120x120
    /// - Background: Vibrant blue (#2673E6)
    /// - Body: White
    /// - Corner radius: 20
    pub fn new() -> Self {
        Self {
            size: 120.0,
            background_color: Color::rgba(0.15, 0.45, 0.9, 0.92),
            body_color: Color::WHITE,
            lens_outer_color: Color::rgba(0.2, 0.3, 0.5, 1.0),
            lens_inner_color: Color::rgba(0.1, 0.15, 0.3, 1.0),
            lens_highlight_color: Color::rgba(1.0, 1.0, 1.0, 0.4),
            flash_color: Color::rgba(1.0, 0.85, 0.2, 1.0),
            corner_radius: 20.0,
        }
    }

    /// Set the size of the indicator (width and height).
    ///
    /// The camera icon scales proportionally.
    pub fn size(mut self, size: f64) -> Self {
        self.size = size;
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

    /// Set the lens colors.
    ///
    /// The inner color is automatically derived darker if not set explicitly.
    pub fn lens_color(mut self, color: Color) -> Self {
        self.lens_outer_color = color;
        self.lens_inner_color = Color::rgba(color.r * 0.5, color.g * 0.5, color.b * 0.5, color.a);
        self
    }

    /// Set the flash indicator color (the yellow dot).
    pub fn flash_color(mut self, color: Color) -> Self {
        self.flash_color = color;
        self
    }

    /// Set the corner radius.
    pub fn corner_radius(mut self, radius: f64) -> Self {
        self.corner_radius = radius;
        self
    }

    /// Get the background color (for use in OsdBuilder).
    pub fn get_background_color(&self) -> Color {
        self.background_color
    }

    /// Get the corner radius (for use in OsdBuilder).
    pub fn get_corner_radius(&self) -> f64 {
        self.corner_radius
    }
}

impl Default for CameraFlash {
    fn default() -> Self {
        Self::new()
    }
}

impl From<CameraFlash> for LayerComposition {
    fn from(cf: CameraFlash) -> Self {
        // Calculate proportional sizes based on 120.0 reference size
        let scale = cf.size / 120.0;

        let body_width = 70.0 * scale;
        let body_height = 45.0 * scale;
        let body_radius = 8.0 * scale;

        let viewfinder_width = 20.0 * scale;
        let viewfinder_height = 10.0 * scale;
        let viewfinder_radius = 3.0 * scale;
        let viewfinder_offset_y = 22.0 * scale;

        let lens_outer_diameter = 32.0 * scale;
        let lens_inner_diameter = 22.0 * scale;
        let lens_highlight_diameter = 8.0 * scale;
        let lens_highlight_offset = 4.0 * scale;

        let flash_diameter = 10.0 * scale;
        let flash_offset_x = 22.0 * scale;
        let flash_offset_y = 12.0 * scale;

        CompositionBuilder::new(cf.size)
            // Camera body (rounded rectangle)
            .layer("body", |l| {
                l.rounded_rect(body_width, body_height, body_radius)
                    .center()
                    .fill(cf.body_color)
            })
            // Viewfinder bump (small rounded rect at top)
            .layer("viewfinder", |l| {
                l.rounded_rect(viewfinder_width, viewfinder_height, viewfinder_radius)
                    .center_offset(0.0, viewfinder_offset_y)
                    .fill(cf.body_color)
            })
            // Lens outer ring
            .layer("lens_outer", |l| {
                l.circle(lens_outer_diameter)
                    .center()
                    .fill(cf.lens_outer_color)
            })
            // Lens inner
            .layer("lens_inner", |l| {
                l.circle(lens_inner_diameter)
                    .center()
                    .fill(cf.lens_inner_color)
            })
            // Lens highlight (top-left of lens)
            .layer("lens_highlight", |l| {
                l.circle(lens_highlight_diameter)
                    .center_offset(-lens_highlight_offset, lens_highlight_offset)
                    .fill(cf.lens_highlight_color)
            })
            // Flash indicator (top right of camera)
            .layer("flash", |l| {
                l.circle(flash_diameter)
                    .center_offset(flash_offset_x, flash_offset_y)
                    .fill(cf.flash_color)
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let cf = CameraFlash::new();
        assert_eq!(cf.size, 120.0);
        assert_eq!(cf.corner_radius, 20.0);
    }

    #[test]
    fn test_size() {
        let cf = CameraFlash::new().size(150.0);
        assert_eq!(cf.size, 150.0);
    }

    #[test]
    fn test_background_color() {
        let cf = CameraFlash::new().background_color(Color::RED);
        assert_eq!(cf.background_color, Color::RED);
    }

    #[test]
    fn test_into_composition() {
        let comp: LayerComposition = CameraFlash::new().into();
        assert_eq!(comp.layers.len(), 6);
        assert_eq!(comp.layers[0].name, "body");
        assert_eq!(comp.layers[1].name, "viewfinder");
        assert_eq!(comp.layers[2].name, "lens_outer");
        assert_eq!(comp.layers[3].name, "lens_inner");
        assert_eq!(comp.layers[4].name, "lens_highlight");
        assert_eq!(comp.layers[5].name, "flash");
    }

    #[test]
    fn test_scaling() {
        // Test that a larger size produces a larger composition
        let small: LayerComposition = CameraFlash::new().size(60.0).into();
        let large: LayerComposition = CameraFlash::new().size(120.0).into();

        assert_eq!(small.size.width, 60.0);
        assert_eq!(large.size.width, 120.0);
    }
}
