//! Canvas implementation for macOS using Core Graphics.
//!
//! This module provides the platform-specific rendering of shapes onto a CGContext.

use std::f64::consts::PI;
use std::ffi::c_void;

use core_graphics::context::CGContext;
use core_graphics::geometry::CGRect;
use core_graphics::path::CGPath;
use foreign_types_shared::ForeignType;

use crate::canvas::Canvas as CanvasTrait;
use crate::color::Color;
use crate::geometry::{Point, Rect, Size};
use crate::shape::Shape;

// Import geometry extensions for CG type conversions
#[allow(unused_imports)]
use super::geometry_ext;

// Core Graphics functions not yet available in the core-graphics crate
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGContextAddArc(
        context: *mut c_void,
        x: f64,
        y: f64,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        clockwise: i32,
    );
    fn CGPathCreateWithRoundedRect(
        rect: CGRect,
        corner_width: f64,
        corner_height: f64,
        transform: *const c_void,
    ) -> *const c_void;
    fn CGPathRelease(path: *const c_void);
}

/// A canvas for drawing shapes using Core Graphics.
///
/// Wraps a CGContext and provides rendering for the common Shape types.
/// The coordinate system is automatically flipped to use top-left origin.
pub struct SkylightCanvas {
    ctx: CGContext,
    size: Size,
}

impl SkylightCanvas {
    /// Create a new canvas wrapping a CGContext.
    ///
    /// The coordinate system is automatically transformed to use top-left origin.
    ///
    /// # Safety
    /// The context pointer must be valid for the lifetime of the Canvas.
    pub unsafe fn new(ctx: *mut c_void, size: Size) -> Self {
        let ctx = CGContext::from_existing_context_ptr(ctx as *mut _);
        let this = Self { ctx, size };
        // Transform to top-left origin (CG uses bottom-left)
        this.ctx.translate(0.0, size.height);
        this.ctx.scale(1.0, -1.0);
        this
    }

    /// Get the center point of the canvas.
    pub fn center(&self) -> Point {
        Point::new(self.size.width / 2.0, self.size.height / 2.0)
    }

    /// Set the fill color.
    fn set_fill_color(&self, color: Color) {
        self.ctx
            .set_rgb_fill_color(color.r, color.g, color.b, color.a);
    }

    /// Draw a rounded rectangle.
    fn fill_rounded_rect(&self, rect: Rect, corner_radius: f64, color: Color) {
        unsafe {
            self.set_fill_color(color);
            let cg_rect: CGRect = rect.into();
            let path = CGPathCreateWithRoundedRect(
                cg_rect,
                corner_radius,
                corner_radius,
                std::ptr::null(),
            );
            let cg_path = CGPath::from_ptr(path as *mut _);
            self.ctx.add_path(&cg_path);
            self.ctx.fill_path();
            std::mem::forget(cg_path);
            CGPathRelease(path);
        }
    }

    /// Draw a filled circle.
    fn fill_circle(&self, center: Point, radius: f64, color: Color) {
        unsafe {
            self.set_fill_color(color);
            CGContextAddArc(
                self.ctx.as_ptr() as *mut c_void,
                center.x,
                center.y,
                radius,
                0.0,
                2.0 * PI,
                0,
            );
            self.ctx.fill_path();
        }
    }
}

impl CanvasTrait for SkylightCanvas {
    fn clear(&mut self) {
        let rect = CGRect {
            origin: core_graphics::geometry::CGPoint { x: 0.0, y: 0.0 },
            size: core_graphics::geometry::CGSize {
                width: self.size.width,
                height: self.size.height,
            },
        };
        self.ctx.clear_rect(rect);
    }

    fn draw_shape(&mut self, shape: &Shape) {
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
                let center = rect.center();
                let radius = rect.size.width.min(rect.size.height) / 2.0;
                self.fill_circle(center, radius, *color);
            }
        }
    }

    fn flush(&self) {
        self.ctx.flush();
    }

    fn size(&self) -> Size {
        self.size
    }
}
