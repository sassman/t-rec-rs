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

// Core Text for text rendering
#[link(name = "CoreText", kind = "framework")]
extern "C" {
    fn CTFontCreateWithName(name: *const c_void, size: f64, matrix: *const c_void) -> *const c_void;
    fn CTLineCreateWithAttributedString(attr_string: *const c_void) -> *const c_void;
    fn CTLineDraw(line: *const c_void, context: *mut c_void);
}

// Core Foundation for attributed strings
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
    fn CFStringCreateWithBytes(
        alloc: *const c_void,
        bytes: *const u8,
        num_bytes: i64,
        encoding: u32,
        is_external: bool,
    ) -> *const c_void;
    fn CFAttributedStringCreate(
        alloc: *const c_void,
        string: *const c_void,
        attributes: *const c_void,
    ) -> *const c_void;
    fn CFDictionaryCreate(
        alloc: *const c_void,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: i64,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> *const c_void;
    static kCFTypeDictionaryKeyCallBacks: c_void;
    static kCFTypeDictionaryValueCallBacks: c_void;
}

// Core Text attribute keys
#[link(name = "CoreText", kind = "framework")]
extern "C" {
    static kCTFontAttributeName: *const c_void;
    static kCTForegroundColorAttributeName: *const c_void;
}

// Core Graphics color and text
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGColorCreateGenericRGB(r: f64, g: f64, b: f64, a: f64) -> *const c_void;
    fn CGColorRelease(color: *const c_void);
    fn CGContextSetTextPosition(context: *mut c_void, x: f64, y: f64);
    fn CGContextSetTextMatrix(context: *mut c_void, t: CGAffineTransform);
}

#[repr(C)]
#[derive(Copy, Clone)]
struct CGAffineTransform {
    a: f64,
    b: f64,
    c: f64,
    d: f64,
    tx: f64,
    ty: f64,
}

impl CGAffineTransform {
    fn identity() -> Self {
        Self {
            a: 1.0,
            b: 0.0,
            c: 0.0,
            d: 1.0,
            tx: 0.0,
            ty: 0.0,
        }
    }
}

const K_CF_STRING_ENCODING_UTF8: u32 = 0x08000100;

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
    /// Uses top-left origin coordinate system (Y increases downward).
    ///
    /// # Safety
    /// The context pointer must be valid for the lifetime of the Canvas.
    pub unsafe fn new(ctx: *mut c_void, size: Size) -> Self {
        let ctx = CGContext::from_existing_context_ptr(ctx as *mut _);
        // Don't flip the context globally - handle per-shape instead
        // This allows text to render correctly
        Self { ctx, size }
    }

    /// Convert Y from top-left origin to bottom-left origin (CG native)
    fn flip_y(&self, y: f64) -> f64 {
        self.size.height - y
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
            // Flip Y for top-left origin
            let flipped_rect = Rect::from_xywh(
                rect.origin.x,
                self.flip_y(rect.origin.y + rect.size.height),
                rect.size.width,
                rect.size.height,
            );
            let cg_rect: CGRect = flipped_rect.into();
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
            // Flip Y for top-left origin
            let flipped_y = self.flip_y(center.y);
            CGContextAddArc(
                self.ctx.as_ptr() as *mut c_void,
                center.x,
                flipped_y,
                radius,
                0.0,
                2.0 * PI,
                0,
            );
            self.ctx.fill_path();
        }
    }

    /// Draw text at a position.
    fn draw_text(&self, text: &str, position: Point, font_size: f64, color: Color) {
        unsafe {
            // Create font name string
            let font_name = "Helvetica Neue";
            let font_name_cf = CFStringCreateWithBytes(
                std::ptr::null(),
                font_name.as_ptr(),
                font_name.len() as i64,
                K_CF_STRING_ENCODING_UTF8,
                false,
            );
            if font_name_cf.is_null() {
                return;
            }

            // Create font
            let font = CTFontCreateWithName(font_name_cf, font_size, std::ptr::null());
            CFRelease(font_name_cf);
            if font.is_null() {
                return;
            }

            // Create text string
            let text_cf = CFStringCreateWithBytes(
                std::ptr::null(),
                text.as_ptr(),
                text.len() as i64,
                K_CF_STRING_ENCODING_UTF8,
                false,
            );
            if text_cf.is_null() {
                CFRelease(font);
                return;
            }

            // Create color
            let cg_color = CGColorCreateGenericRGB(color.r, color.g, color.b, color.a);

            // Create attributes dictionary
            let keys: [*const c_void; 2] = [kCTFontAttributeName, kCTForegroundColorAttributeName];
            let values: [*const c_void; 2] = [font, cg_color];

            let attributes = CFDictionaryCreate(
                std::ptr::null(),
                keys.as_ptr(),
                values.as_ptr(),
                2,
                &kCFTypeDictionaryKeyCallBacks as *const _ as *const c_void,
                &kCFTypeDictionaryValueCallBacks as *const _ as *const c_void,
            );

            // Create attributed string
            let attr_string = CFAttributedStringCreate(std::ptr::null(), text_cf, attributes);
            CFRelease(attributes);
            CFRelease(text_cf);

            if !attr_string.is_null() {
                // Create line
                let line = CTLineCreateWithAttributedString(attr_string);
                CFRelease(attr_string);

                if !line.is_null() {
                    // Context is in native CG coordinates (bottom-left origin)
                    // Text baseline is at the specified Y position
                    // Flip Y and account for text rendering from baseline
                    let flipped_y = self.flip_y(position.y + font_size * 0.8);

                    // Use identity text matrix - no flipping needed
                    CGContextSetTextMatrix(
                        self.ctx.as_ptr() as *mut c_void,
                        CGAffineTransform::identity(),
                    );

                    CGContextSetTextPosition(
                        self.ctx.as_ptr() as *mut c_void,
                        position.x,
                        flipped_y,
                    );
                    CTLineDraw(line, self.ctx.as_ptr() as *mut c_void);

                    CFRelease(line);
                }
            }

            CGColorRelease(cg_color);
            CFRelease(font);
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
            Shape::Text {
                text,
                position,
                size,
                color,
            } => {
                self.draw_text(text, *position, *size, *color);
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
