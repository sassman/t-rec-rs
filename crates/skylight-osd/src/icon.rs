//! Icon builder for composing visual elements.
//!
//! Provides a declarative API for building icons from shapes.

use super::color::Color;
use super::drawing::{Canvas, Shape};
use super::geometry::{Point, Rect};

/// A composed icon made of multiple shapes.
#[derive(Debug, Clone)]
pub struct Icon {
    /// The size of the icon canvas.
    pub size: f64,
    /// The shapes that make up the icon.
    pub shapes: Vec<Shape>,
}

impl Icon {
    /// Draw the icon onto a canvas.
    pub fn draw(&self, canvas: &mut Canvas) {
        canvas.clear();
        canvas.flip_vertical();
        canvas.draw_shapes(&self.shapes);
        canvas.flush();
    }
}

/// Builder for creating icons declaratively.
#[derive(Debug, Clone)]
pub struct IconBuilder {
    size: f64,
    shapes: Vec<Shape>,
    padding: f64,
}

impl IconBuilder {
    /// Create a new icon builder with the given size.
    pub fn new(size: f64) -> Self {
        Self {
            size,
            shapes: Vec::new(),
            padding: 0.0,
        }
    }

    /// Set the padding around the icon content.
    pub fn padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    /// Add a background rounded rectangle.
    pub fn background(mut self, color: Color, corner_radius: f64) -> Self {
        let rect = Rect::from_xywh(
            self.padding / 2.0,
            self.padding / 2.0,
            self.size - self.padding,
            self.size - self.padding,
        );
        self.shapes
            .push(Shape::rounded_rect(rect, corner_radius, color));
        self
    }

    /// Add a shape to the icon.
    pub fn add_shape(mut self, shape: Shape) -> Self {
        self.shapes.push(shape);
        self
    }

    /// Add a rounded rectangle.
    pub fn rounded_rect(
        mut self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        corner_radius: f64,
        color: Color,
    ) -> Self {
        self.shapes.push(Shape::rounded_rect_xywh(
            x,
            y,
            width,
            height,
            corner_radius,
            color,
        ));
        self
    }

    /// Add a circle.
    pub fn circle(mut self, x: f64, y: f64, radius: f64, color: Color) -> Self {
        self.shapes.push(Shape::circle_at(x, y, radius, color));
        self
    }

    /// Add a circle centered on the icon.
    pub fn circle_centered(mut self, offset_y: f64, radius: f64, color: Color) -> Self {
        let center_x = self.size / 2.0;
        let center_y = self.size / 2.0 + offset_y;
        self.shapes
            .push(Shape::circle_at(center_x, center_y, radius, color));
        self
    }

    /// Get the center point of the icon.
    pub fn center(&self) -> Point {
        Point::new(self.size / 2.0, self.size / 2.0)
    }

    /// Get the content bounds (accounting for padding).
    pub fn content_bounds(&self) -> Rect {
        Rect::from_xywh(
            self.padding,
            self.padding,
            self.size - 2.0 * self.padding,
            self.size - 2.0 * self.padding,
        )
    }

    /// Build the final icon.
    pub fn build(self) -> Icon {
        Icon {
            size: self.size,
            shapes: self.shapes,
        }
    }
}

/// Create a camera icon for screenshot feedback.
///
/// This creates a clean, modern camera icon with:
/// - Vibrant blue rounded background
/// - White camera body with viewfinder bump
/// - Dark lens with blue reflection and highlight
/// - Yellow flash indicator
pub fn camera_icon(size: f64) -> Icon {
    let padding = 12.0;
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
        .padding(padding)
        // Background rounded rectangle (vibrant blue)
        .background(Color::VIBRANT_BLUE, 16.0)
        // Camera body (white)
        .rounded_rect(
            center_x - cam_w / 2.0,
            center_y - cam_h / 2.0 + 4.0,
            cam_w,
            cam_h,
            8.0,
            Color::WHITE,
        )
        // Viewfinder bump on top
        .rounded_rect(
            center_x - vf_w / 2.0,
            center_y - cam_h / 2.0 - vf_h + 8.0,
            vf_w,
            vf_h + 2.0,
            3.0,
            Color::WHITE,
        )
        // Lens outer ring (dark)
        .circle(center_x, center_y + 4.0, lens_r, Color::DARK_GRAY)
        // Lens inner (blue reflection)
        .circle(center_x, center_y + 4.0, lens_r * 0.65, Color::LIGHT_BLUE)
        // Lens highlight dot
        .circle(
            center_x - lens_r * 0.25,
            center_y + 4.0 - lens_r * 0.25,
            lens_r * 0.2,
            Color::WHITE.with_alpha(0.8),
        )
        // Flash indicator (yellow)
        .circle(
            center_x + cam_w / 2.0 - 12.0,
            center_y - cam_h / 2.0 + 14.0,
            4.0,
            Color::WARM_YELLOW,
        )
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_icon_builder_new() {
        let builder = IconBuilder::new(100.0);
        assert_eq!(builder.size, 100.0);
        assert!(builder.shapes.is_empty());
    }

    #[test]
    fn test_icon_builder_padding() {
        let builder = IconBuilder::new(100.0).padding(10.0);
        let bounds = builder.content_bounds();
        assert_eq!(bounds.origin.x, 10.0);
        assert_eq!(bounds.size.width, 80.0);
    }

    #[test]
    fn test_icon_builder_background() {
        let icon = IconBuilder::new(100.0)
            .padding(10.0)
            .background(Color::RED, 8.0)
            .build();
        assert_eq!(icon.shapes.len(), 1);
    }

    #[test]
    fn test_icon_builder_chain() {
        let icon = IconBuilder::new(100.0)
            .padding(10.0)
            .background(Color::BLUE, 8.0)
            .circle(50.0, 50.0, 20.0, Color::WHITE)
            .rounded_rect(10.0, 10.0, 30.0, 20.0, 5.0, Color::RED)
            .build();
        assert_eq!(icon.shapes.len(), 3);
    }

    #[test]
    fn test_camera_icon() {
        let icon = camera_icon(120.0);
        assert_eq!(icon.size, 120.0);
        // Camera icon has: background + body + viewfinder + 3 lens parts + flash = 7 shapes
        assert_eq!(icon.shapes.len(), 7);
    }

    #[test]
    fn test_icon_builder_center() {
        let builder = IconBuilder::new(100.0);
        let center = builder.center();
        assert_eq!(center.x, 50.0);
        assert_eq!(center.y, 50.0);
    }
}
