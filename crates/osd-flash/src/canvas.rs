//! Canvas trait for platform-agnostic drawing.
//!
//! This module defines the interface that all platform-specific
//! canvas implementations must provide.

use crate::geometry::{Point, Rect};
use crate::style::{Paint, TextStyle};
use crate::Drawable;

/// A drawing surface that can render shapes.
///
/// This trait abstracts over platform-specific graphics contexts,
/// providing primitive drawing operations. Application code uses
/// these primitives to compose complex visuals.
///
/// # Example
///
/// ```ignore
/// impl Drawable for MyIcon {
///     fn draw(&self, canvas: &mut dyn Canvas) {
///         // Draw background
///         canvas.draw_rounded_rect(self.bounds, 12.0, &Paint::new(Color::BLUE));
///         // Draw circle
///         canvas.draw_circle(self.center, 20.0, &Paint::new(Color::WHITE));
///     }
/// }
/// ```
pub trait Canvas {
    /// Clear the canvas to transparent.
    fn clear(&mut self);

    /// Draw a rectangle.
    fn draw_rect(&mut self, rect: &Rect, paint: &Paint);

    /// Draw a rectangle with rounded corners.
    fn draw_rounded_rect(&mut self, rect: &Rect, corner_radius: f64, paint: &Paint);

    /// Draw a circle.
    fn draw_circle(&mut self, center: &Point, radius: f64, paint: &Paint);

    /// Draw an ellipse inscribed in the given rectangle.
    fn draw_ellipse(&mut self, rect: &Rect, paint: &Paint);

    /// Draw text at the specified position with the given style.
    fn draw_text(&mut self, text: &str, position: &Point, style: &TextStyle);

    /// Flush any buffered drawing operations.
    fn flush(&self);

    // Draw another drawable (composition)
    fn draw(&mut self, drawable: &dyn Drawable, bounds: &Rect)
    where
        Self: Sized,
    {
        drawable.draw(self, bounds);
    }
}
