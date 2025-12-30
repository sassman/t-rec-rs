//! Canvas trait for platform-agnostic drawing.
//!
//! This module defines the interface that all platform-specific
//! canvas implementations must provide.

use crate::geometry::Size;
use crate::shape::Shape;

/// A drawing surface that can render shapes.
///
/// This trait abstracts over platform-specific graphics contexts,
/// allowing icons and other drawable elements to be rendered
/// without knowing the underlying platform.
pub trait Canvas {
    /// Clear the canvas to transparent.
    fn clear(&mut self);

    /// Draw a single shape.
    fn draw_shape(&mut self, shape: &Shape);

    /// Draw multiple shapes.
    fn draw_shapes(&mut self, shapes: &[Shape]) {
        for shape in shapes {
            self.draw_shape(shape);
        }
    }

    /// Flush any buffered drawing operations.
    fn flush(&self);

    /// Get the canvas size.
    fn size(&self) -> Size;
}
