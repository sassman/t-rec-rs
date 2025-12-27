//! Drawing primitives and canvas abstraction.
//!
//! This module provides a high-level API for drawing shapes onto a CGContext.

use std::f64::consts::PI;
use std::ffi::c_void;

use core_graphics::geometry::CGRect;

use super::color::Color;
use super::geometry::{Point, Rect, Size};

// Core Graphics functions not available in the core-graphics crate
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGContextSetRGBFillColor(context: *mut c_void, red: f64, green: f64, blue: f64, alpha: f64);
    fn CGContextFillPath(context: *mut c_void);
    fn CGContextAddPath(context: *mut c_void, path: *const c_void);
    fn CGContextAddArc(
        context: *mut c_void,
        x: f64,
        y: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        clockwise: i32,
    );
    fn CGContextFlush(context: *mut c_void);
    fn CGContextClearRect(context: *mut c_void, rect: CGRect);
    fn CGContextTranslateCTM(context: *mut c_void, tx: f64, ty: f64);
    fn CGContextScaleCTM(context: *mut c_void, sx: f64, sy: f64);
    fn CGPathCreateWithRoundedRect(
        rect: CGRect,
        corner_width: f64,
        corner_height: f64,
        transform: *const c_void,
    ) -> *const c_void;
    fn CGPathRelease(path: *const c_void);
}

/// A drawable shape.
#[derive(Debug, Clone)]
pub enum Shape {
    /// A rectangle with rounded corners.
    RoundedRect {
        rect: Rect,
        corner_radius: f64,
        color: Color,
    },
    /// A filled circle.
    Circle {
        center: Point,
        radius: f64,
        color: Color,
    },
    /// A filled ellipse.
    Ellipse { rect: Rect, color: Color },
}

impl Shape {
    /// Create a rounded rectangle shape.
    pub fn rounded_rect(rect: Rect, corner_radius: f64, color: Color) -> Self {
        Self::RoundedRect {
            rect,
            corner_radius,
            color,
        }
    }

    /// Create a rounded rectangle from position and size.
    pub fn rounded_rect_xywh(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        corner_radius: f64,
        color: Color,
    ) -> Self {
        Self::RoundedRect {
            rect: Rect::from_xywh(x, y, width, height),
            corner_radius,
            color,
        }
    }

    /// Create a circle shape.
    pub fn circle(center: Point, radius: f64, color: Color) -> Self {
        Self::Circle {
            center,
            radius,
            color,
        }
    }

    /// Create a circle at x, y coordinates.
    pub fn circle_at(x: f64, y: f64, radius: f64, color: Color) -> Self {
        Self::Circle {
            center: Point::new(x, y),
            radius,
            color,
        }
    }

    /// Create an ellipse shape.
    pub fn ellipse(rect: Rect, color: Color) -> Self {
        Self::Ellipse { rect, color }
    }
}

/// A canvas for drawing shapes.
///
/// Wraps a CGContext and provides a high-level drawing API.
pub struct Canvas {
    ctx: *mut c_void,
    size: Size,
}

impl Canvas {
    /// Create a new canvas wrapping a CGContext.
    ///
    /// # Safety
    /// The context pointer must be valid for the lifetime of the Canvas.
    pub unsafe fn new(ctx: *mut c_void, size: Size) -> Self {
        Self { ctx, size }
    }

    /// Get the canvas size.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Get the center point of the canvas.
    pub fn center(&self) -> Point {
        Point::new(self.size.width / 2.0, self.size.height / 2.0)
    }

    /// Clear the canvas to transparent.
    pub fn clear(&mut self) {
        unsafe {
            CGContextClearRect(
                self.ctx,
                CGRect {
                    origin: core_graphics::geometry::CGPoint { x: 0.0, y: 0.0 },
                    size: core_graphics::geometry::CGSize {
                        width: self.size.width,
                        height: self.size.height,
                    },
                },
            );
        }
    }

    /// Apply a vertical flip transformation.
    ///
    /// CG coordinate system has origin at bottom-left; this flips to top-left.
    pub fn flip_vertical(&mut self) {
        unsafe {
            CGContextTranslateCTM(self.ctx, self.size.width, self.size.height);
            CGContextScaleCTM(self.ctx, -1.0, -1.0);
        }
    }

    /// Set the fill color.
    fn set_fill_color(&mut self, color: Color) {
        unsafe {
            CGContextSetRGBFillColor(self.ctx, color.r, color.g, color.b, color.a);
        }
    }

    /// Draw a rounded rectangle.
    pub fn fill_rounded_rect(&mut self, rect: Rect, corner_radius: f64, color: Color) {
        unsafe {
            self.set_fill_color(color);
            let cg_rect: CGRect = rect.into();
            let path = CGPathCreateWithRoundedRect(
                cg_rect,
                corner_radius,
                corner_radius,
                std::ptr::null(),
            );
            CGContextAddPath(self.ctx, path);
            CGContextFillPath(self.ctx);
            CGPathRelease(path);
        }
    }

    /// Draw a filled circle.
    pub fn fill_circle(&mut self, center: Point, radius: f64, color: Color) {
        unsafe {
            self.set_fill_color(color);
            CGContextAddArc(self.ctx, center.x, center.y, radius, 0.0, 2.0 * PI, 0);
            CGContextFillPath(self.ctx);
        }
    }

    /// Draw a shape.
    pub fn draw_shape(&mut self, shape: &Shape) {
        match shape {
            Shape::RoundedRect {
                rect,
                corner_radius,
                color,
            } => {
                self.fill_rounded_rect(*rect, *corner_radius, *color);
            }
            Shape::Circle {
                center,
                radius,
                color,
            } => {
                self.fill_circle(*center, *radius, *color);
            }
            Shape::Ellipse { rect, color } => {
                // Approximate ellipse with arc
                let center = rect.center();
                let radius = rect.size.width.min(rect.size.height) / 2.0;
                self.fill_circle(center, radius, *color);
            }
        }
    }

    /// Draw multiple shapes.
    pub fn draw_shapes(&mut self, shapes: &[Shape]) {
        for shape in shapes {
            self.draw_shape(shape);
        }
    }

    /// Flush the context to display the drawn content.
    pub fn flush(&mut self) {
        unsafe {
            CGContextFlush(self.ctx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_rounded_rect() {
        let shape = Shape::rounded_rect(Rect::from_xywh(0.0, 0.0, 100.0, 50.0), 10.0, Color::RED);
        match shape {
            Shape::RoundedRect {
                rect,
                corner_radius,
                color,
            } => {
                assert_eq!(rect.size.width, 100.0);
                assert_eq!(corner_radius, 10.0);
                assert_eq!(color, Color::RED);
            }
            _ => panic!("Expected RoundedRect"),
        }
    }

    #[test]
    fn test_shape_circle() {
        let shape = Shape::circle(Point::new(50.0, 50.0), 25.0, Color::BLUE);
        match shape {
            Shape::Circle {
                center,
                radius,
                color,
            } => {
                assert_eq!(center.x, 50.0);
                assert_eq!(radius, 25.0);
                assert_eq!(color, Color::BLUE);
            }
            _ => panic!("Expected Circle"),
        }
    }

    #[test]
    fn test_shape_circle_at() {
        let shape = Shape::circle_at(10.0, 20.0, 5.0, Color::GREEN);
        match shape {
            Shape::Circle { center, radius, .. } => {
                assert_eq!(center.x, 10.0);
                assert_eq!(center.y, 20.0);
                assert_eq!(radius, 5.0);
            }
            _ => panic!("Expected Circle"),
        }
    }
}
