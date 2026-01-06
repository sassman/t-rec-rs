//! SkyLight implementation of the OsdWindow trait.

use crate::animation::animated_window::AnimatedWindow;
use crate::animation::transform::Transform;
use crate::canvas::Canvas;
use crate::color::Color;
use crate::geometry::{Point, Rect, Size};
use crate::icon::{Icon, StyledShape};
use crate::layout::Padding;
use crate::style::Paint;
use crate::window::{DisplayTarget, OsdFlashBuilder, OsdWindow, WindowLevel};
use crate::Drawable;
use crate::FlashPosition;

use super::canvas::SkylightCanvas;
use super::window::{
    get_window_bounds, DisplayTarget as SkylightDisplayTarget, SkylightWindow,
    SkylightWindowBuilder, WindowLevel as SkylightWindowLevel,
};

/// Menu bar height in points (approximate).
const MENU_BAR_HEIGHT: f64 = 25.0;

/// SkyLight-based OSD window implementation.
pub struct SkylightOsdWindow {
    window: SkylightWindow,
    /// Content dimensions (frame minus padding)
    content_size: Size,
    /// Full window size (what the user specifies)
    frame_size: Size,
    /// Padding around content
    padding: Padding,
    /// Background color (if any)
    background: Option<Color>,
    /// Corner radius for background
    corner_radius: f64,
}

impl SkylightOsdWindow {
    /// Create a new SkyLight OSD window from an OsdFlashBuilder.
    pub fn from_builder(builder: OsdFlashBuilder) -> crate::Result<Self> {
        let dimensions = builder.get_dimensions();
        let position = builder.get_position();
        let margin = builder.get_margin();
        let padding = builder.get_padding();
        let background = builder.get_background();
        let corner_radius = builder.get_corner_radius();
        let level = builder.get_level();
        let display_target = builder.get_display_target();

        // Convert to backend-specific display target and get bounds
        let (bounds, top_inset, skylight_target) = match display_target {
            DisplayTarget::Main => {
                let display_bounds = get_main_display_bounds();
                (display_bounds, MENU_BAR_HEIGHT, SkylightDisplayTarget::Main)
            }
            DisplayTarget::Window(window_id) => {
                // Use window bounds for positioning (no menu bar offset)
                // Fall back to main display if window not found
                let window_bounds =
                    get_window_bounds(window_id).unwrap_or_else(get_main_display_bounds);
                (window_bounds, 0.0, SkylightDisplayTarget::Window(window_id))
            }
        };

        // dimensions is the full frame size (what the user specifies)
        // content_size is the area inside the padding
        let frame_size = dimensions;
        let content_size = Size::new(
            dimensions.width - padding.horizontal(),
            dimensions.height - padding.vertical(),
        );

        // Calculate frame based on position and margin (handles scaling internally)
        let frame = calculate_frame(&frame_size, &position, &margin, &bounds, top_inset);

        // Convert WindowLevel to SkyLight-specific level
        let skylight_level = convert_window_level(level);

        // Build the underlying SkyLight window
        let window = SkylightWindowBuilder::new()
            .frame(frame)
            .level(skylight_level)
            .display(skylight_target)
            .build()?;

        Ok(Self {
            window,
            content_size,
            frame_size,
            padding,
            background,
            corner_radius,
        })
    }
}

impl SkylightOsdWindow {
    /// Draw content onto the window (internal helper).
    fn draw_content(&self, content: &Icon) {
        // Clear and draw background
        let mut full_canvas =
            unsafe { SkylightCanvas::new(self.window.context_ptr(), self.frame_size) };
        full_canvas.clear();

        // Draw background if configured
        if let Some(bg_color) = self.background {
            let bg_rect =
                Rect::from_xywh(0.0, 0.0, self.frame_size.width, self.frame_size.height);
            let paint = Paint::fill(bg_color);
            full_canvas.draw_rounded_rect(&bg_rect, self.corner_radius, &paint);
        }

        // Create canvas with padding offset for content positioning
        let offset = Point::new(self.padding.left, self.padding.top);
        let mut canvas = unsafe {
            SkylightCanvas::with_frame_and_offset(
                self.window.context_ptr(),
                self.content_size,
                self.frame_size.height,
                offset,
            )
        };
        let bounds = Rect::new(Point::new(0.0, 0.0), self.content_size);

        content.draw(&mut canvas, &bounds);
        canvas.flush();
    }
}

impl OsdWindow for SkylightOsdWindow {
    fn draw(self, content: impl Into<Icon>) -> AnimatedWindow<Self> {
        AnimatedWindow::new(self, content.into())
    }

    fn show_window(&self) -> crate::Result<()> {
        self.window.show_visible()
    }

    fn hide_window(&self) -> crate::Result<()> {
        self.window.hide()
    }

    fn draw_and_show(&self, content: Icon, seconds: f64) -> crate::Result<()> {
        // Draw the content
        self.draw_content(&content);

        // Show window, run loop, hide window
        self.window.show_visible()?;

        // Run the event loop for the specified duration
        use std::ffi::c_void;
        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            fn CFRunLoopRunInMode(
                mode: *const c_void,
                seconds: f64,
                return_after_source_handled: bool,
            ) -> i32;
            static kCFRunLoopDefaultMode: *const c_void;
        }
        unsafe {
            CFRunLoopRunInMode(kCFRunLoopDefaultMode, seconds, false);
        }

        self.window.hide()
    }

    fn render_animation_frame(
        &self,
        content: &Icon,
        transform: &Transform,
        shapes: &[StyledShape],
    ) -> crate::Result<()> {
        // Create canvas for full frame
        let mut full_canvas =
            unsafe { SkylightCanvas::new(self.window.context_ptr(), self.frame_size) };

        // Always clear to transparent first (needed for rounded corners)
        full_canvas.clear();

        // Draw background if configured
        if let Some(bg_color) = self.background {
            let bg_rect =
                Rect::from_xywh(0.0, 0.0, self.frame_size.width, self.frame_size.height);
            let paint = Paint::fill(bg_color);
            full_canvas.draw_rounded_rect(&bg_rect, self.corner_radius, &paint);
        }

        // Create canvas with padding offset for content positioning
        let offset = Point::new(self.padding.left, self.padding.top);
        let mut canvas = unsafe {
            SkylightCanvas::with_frame_and_offset(
                self.window.context_ptr(),
                self.content_size,
                self.frame_size.height,
                offset,
            )
        };
        let bounds = Rect::new(Point::new(0.0, 0.0), self.content_size);
        let center = Point::new(self.content_size.width / 2.0, self.content_size.height / 2.0);

        // Apply transform if not identity
        if !transform.is_identity() {
            canvas.save_state();
            canvas.scale(transform.scale, &center);
        }

        // Draw base content
        content.draw(&mut canvas, &bounds);

        // Draw overlay shapes (from animation keyframes)
        for styled_shape in shapes {
            styled_shape.draw(&mut canvas, &bounds);
        }

        // Restore transform if applied
        if !transform.is_identity() {
            canvas.restore_state();
        }

        canvas.flush();
        Ok(())
    }
}

/// Get the main display bounds.
fn get_main_display_bounds() -> Rect {
    use core_graphics::display::{CGDisplayBounds, CGMainDisplayID};

    unsafe {
        let display_id = CGMainDisplayID();
        let bounds = CGDisplayBounds(display_id);
        Rect::from_xywh(
            bounds.origin.x,
            bounds.origin.y,
            bounds.size.width,
            bounds.size.height,
        )
    }
}

/// Retina scale factor: CGDisplayBounds returns physical pixels, SkyLight expects logical points.
const DISPLAY_SCALE: f64 = 2.0;

/// Calculate window frame based on position and margin.
///
/// All coordinates are in logical points.
fn calculate_frame(
    dimensions: &Size,
    position: &FlashPosition,
    margin: &crate::layout::Margin,
    bounds: &Rect,
    top_inset: f64,
) -> Rect {
    // Scale display bounds from physical pixels to logical points
    let bx = bounds.origin.x / DISPLAY_SCALE;
    let by = bounds.origin.y / DISPLAY_SCALE;
    let bw = bounds.size.width / DISPLAY_SCALE;
    let bh = bounds.size.height / DISPLAY_SCALE;

    // User dimensions, margins, and insets are already in logical points - don't scale
    let sw = dimensions.width;
    let sh = dimensions.height;
    let mt = margin.top;
    let mr = margin.right;
    let mb = margin.bottom;
    let ml = margin.left;
    let ti = top_inset;

    let origin = match position {
        FlashPosition::TopRight => Point::new(bx + bw - sw - mr, by + mt + ti),
        FlashPosition::TopLeft => Point::new(bx + ml, by + mt + ti),
        FlashPosition::BottomRight => Point::new(bx + bw - sw - mr, by + bh - sh - mb),
        FlashPosition::BottomLeft => Point::new(bx + ml, by + bh - sh - mb),
        FlashPosition::Center => Point::new(bx + bw / 2.0 - sw / 4.0, by + bh / 2.0 - sh / 4.0),
        FlashPosition::Custom { x, y } => Point::new(*x, *y),
    };

    Rect::new(origin, *dimensions).rounded()
}

/// Convert platform-agnostic WindowLevel to SkyLight-specific level.
fn convert_window_level(level: WindowLevel) -> SkylightWindowLevel {
    match level {
        WindowLevel::Normal => SkylightWindowLevel::Normal,
        WindowLevel::Floating => SkylightWindowLevel::Floating,
        WindowLevel::ModalPanel => SkylightWindowLevel::ModalPanel,
        WindowLevel::AboveAll => SkylightWindowLevel::AboveAll,
        WindowLevel::Custom(v) => SkylightWindowLevel::Custom(v),
    }
}
