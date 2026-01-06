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
use crate::style::{Paint, TextStyle};

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
    fn CTFontCreateWithName(name: *const c_void, size: f64, matrix: *const c_void)
        -> *const c_void;
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

// Core Graphics color, text, and state management
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGColorCreateGenericRGB(r: f64, g: f64, b: f64, a: f64) -> *const c_void;
    fn CGColorRelease(color: *const c_void);
    fn CGContextSetTextPosition(context: *mut c_void, x: f64, y: f64);
    fn CGContextSetTextMatrix(context: *mut c_void, t: CGAffineTransform);
    fn CGContextSaveGState(context: *mut c_void);
    fn CGContextRestoreGState(context: *mut c_void);
    fn CGContextTranslateCTM(context: *mut c_void, tx: f64, ty: f64);
    fn CGContextScaleCTM(context: *mut c_void, sx: f64, sy: f64);
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
    /// Content size in user coordinates (for size() method)
    size: Size,
    /// Frame height in window coordinates for Y-coordinate flipping
    frame_height: f64,
    /// Offset in window coordinates (for padding)
    offset: Point,
    /// Scale factor to convert user coordinates to window coordinates
    scale: f64,
}

impl SkylightCanvas {
    /// Create a new canvas wrapping a CGContext.
    ///
    /// Uses top-left origin coordinate system (Y increases downward).
    ///
    /// # Safety
    /// The context pointer must be valid for the lifetime of the Canvas.
    pub unsafe fn new(ctx: *mut c_void, size: Size) -> Self {
        Self::with_frame_and_offset(ctx, size, size.height, Point::zero())
    }

    /// Create a new canvas with an offset applied to all drawing operations.
    ///
    /// The offset is in logical coordinates (top-left origin).
    /// This is useful for applying padding to content.
    ///
    /// # Safety
    /// The context pointer must be valid for the lifetime of the Canvas.
    pub unsafe fn with_offset(ctx: *mut c_void, size: Size, offset: Point) -> Self {
        Self::with_frame_and_offset(ctx, size, size.height, offset)
    }

    /// Create a new canvas with frame height for Y-flipping and content offset.
    ///
    /// - `size`: Content size (returned by size() method)
    /// - `frame_height`: Total frame height for Y-coordinate flipping
    /// - `offset`: Offset applied to drawing operations (for padding)
    /// - `scale`: Scale factor to apply to all drawing coordinates (for Retina)
    ///
    /// # Safety
    /// The context pointer must be valid for the lifetime of the Canvas.
    pub unsafe fn with_frame_and_offset(
        ctx: *mut c_void,
        size: Size,
        frame_height: f64,
        offset: Point,
    ) -> Self {
        Self::with_scale(ctx, size, frame_height, offset, 1.0)
    }

    /// Create a new canvas with custom scale factor.
    ///
    /// - `size`: Content size in user coordinates (returned by size() method)
    /// - `frame_height`: Frame height in window coordinates for Y-flipping
    /// - `offset`: Offset in window coordinates (for padding)
    /// - `scale`: Scale factor (e.g., 0.5 for Retina where user coords are 2x window coords)
    ///
    /// # Safety
    /// The context pointer must be valid for the lifetime of the Canvas.
    pub unsafe fn with_scale(
        ctx: *mut c_void,
        size: Size,
        frame_height: f64,
        offset: Point,
        scale: f64,
    ) -> Self {
        let ctx = CGContext::from_existing_context_ptr(ctx as *mut _);
        // Don't flip the context globally - handle per-shape instead
        // This allows text to render correctly
        Self {
            ctx,
            size,
            frame_height,
            offset,
            scale,
        }
    }

    /// Convert Y from top-left origin to bottom-left origin (CG native)
    fn flip_y(&self, y: f64) -> f64 {
        self.frame_height - y
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
    fn fill_rounded_rect(&self, rect: &Rect, corner_radius: f64, color: Color) {
        unsafe {
            self.set_fill_color(color);
            // Scale input coordinates from user space to window space
            let scaled_rect = Rect::from_xywh(
                rect.origin.x * self.scale,
                rect.origin.y * self.scale,
                rect.size.width * self.scale,
                rect.size.height * self.scale,
            );
            let scaled_radius = corner_radius * self.scale;
            // Apply offset and flip Y for top-left origin
            let offset_rect = Rect::from_xywh(
                scaled_rect.origin.x + self.offset.x,
                scaled_rect.origin.y + self.offset.y,
                scaled_rect.size.width,
                scaled_rect.size.height,
            );
            let flipped_rect = Rect::from_xywh(
                offset_rect.origin.x,
                self.flip_y(offset_rect.origin.y + offset_rect.size.height),
                offset_rect.size.width,
                offset_rect.size.height,
            );
            let cg_rect: CGRect = flipped_rect.into();
            let path = CGPathCreateWithRoundedRect(
                cg_rect,
                scaled_radius,
                scaled_radius,
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
            // Scale input coordinates from user space to window space
            let scaled_center = Point::new(center.x * self.scale, center.y * self.scale);
            let scaled_radius = radius * self.scale;
            // Apply offset and flip Y for top-left origin
            let offset_center = Point::new(
                scaled_center.x + self.offset.x,
                scaled_center.y + self.offset.y,
            );
            let flipped_y = self.flip_y(offset_center.y);
            CGContextAddArc(
                self.ctx.as_ptr() as *mut c_void,
                offset_center.x,
                flipped_y,
                scaled_radius,
                0.0,
                2.0 * PI,
                0,
            );
            self.ctx.fill_path();
        }
    }

    /// Draw text at a position (internal implementation).
    fn draw_text_internal(
        &self,
        text: &str,
        position: &Point,
        font_size: f64,
        font_family: &str,
        color: &Color,
    ) {
        // Scale input coordinates from user space to window space
        let scaled_position = Point::new(position.x * self.scale, position.y * self.scale);
        let scaled_font_size = font_size * self.scale;
        // Apply offset to position
        let offset_position = Point::new(
            scaled_position.x + self.offset.x,
            scaled_position.y + self.offset.y,
        );

        unsafe {
            // Create font name string
            let font_name_cf = CFStringCreateWithBytes(
                std::ptr::null(),
                font_family.as_ptr(),
                font_family.len() as i64,
                K_CF_STRING_ENCODING_UTF8,
                false,
            );
            if font_name_cf.is_null() {
                return;
            }

            // Create font with scaled size
            let font = CTFontCreateWithName(font_name_cf, scaled_font_size, std::ptr::null());
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
                &kCFTypeDictionaryKeyCallBacks as *const _,
                &kCFTypeDictionaryValueCallBacks as *const _,
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
                    let flipped_y = self.flip_y(offset_position.y + scaled_font_size * 0.8);

                    // Use identity text matrix - no flipping needed
                    CGContextSetTextMatrix(
                        self.ctx.as_ptr() as *mut c_void,
                        CGAffineTransform::identity(),
                    );

                    CGContextSetTextPosition(
                        self.ctx.as_ptr() as *mut c_void,
                        offset_position.x,
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

    fn draw_rect(&mut self, rect: &Rect, paint: &Paint) {
        self.fill_rounded_rect(rect, 0.0, paint.effective_color());
    }

    fn draw_rounded_rect(&mut self, rect: &Rect, corner_radius: f64, paint: &Paint) {
        self.fill_rounded_rect(rect, corner_radius, paint.effective_color());
    }

    fn draw_circle(&mut self, center: &Point, radius: f64, paint: &Paint) {
        self.fill_circle(*center, radius, paint.effective_color());
    }

    fn draw_ellipse(&mut self, rect: &Rect, paint: &Paint) {
        // Approximate ellipse as circle using smaller dimension
        // TODO: Implement proper ellipse drawing with CGContextAddEllipseInRect
        let center = rect.center();
        let radius = rect.size.width.min(rect.size.height) / 2.0;
        self.fill_circle(center, radius, paint.effective_color());
    }

    fn draw_text(&mut self, text: &str, position: &Point, style: &TextStyle) {
        let color = style.effective_color();
        self.draw_text_internal(text, position, style.size, &style.font_family, &color);
    }

    fn flush(&self) {
        self.ctx.flush();
    }

    fn save_state(&mut self) {
        unsafe {
            CGContextSaveGState(self.ctx.as_ptr() as *mut c_void);
        }
    }

    fn restore_state(&mut self) {
        unsafe {
            CGContextRestoreGState(self.ctx.as_ptr() as *mut c_void);
        }
    }

    fn scale(&mut self, scale: f64, center: &Point) {
        // Scale around the center point:
        // 1. Translate so center is at origin
        // 2. Apply scale
        // 3. Translate back
        //
        // In CG coordinates (bottom-left origin), we need to flip the center Y.
        let scaled_center = Point::new(center.x * self.scale, center.y * self.scale);
        let offset_center = Point::new(
            scaled_center.x + self.offset.x,
            scaled_center.y + self.offset.y,
        );
        let flipped_y = self.flip_y(offset_center.y);

        unsafe {
            // Translate to center
            CGContextTranslateCTM(self.ctx.as_ptr() as *mut c_void, offset_center.x, flipped_y);
            // Apply scale
            CGContextScaleCTM(self.ctx.as_ptr() as *mut c_void, scale, scale);
            // Translate back
            CGContextTranslateCTM(self.ctx.as_ptr() as *mut c_void, -offset_center.x, -flipped_y);
        }
    }
}
