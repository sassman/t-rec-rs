//! SkyLight implementation of the OsdWindow trait.

use crate::geometry::{Point, Rect, Size};
use crate::window::{DisplayTarget, Drawable, OsdFlashBuilder, OsdWindow, WindowLevel};
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
    size: Size,
}

impl SkylightOsdWindow {
    /// Create a new SkyLight OSD window from an OsdFlashBuilder.
    pub fn from_builder(builder: OsdFlashBuilder) -> crate::Result<Self> {
        let dimensions = builder.get_dimensions();
        let position = builder.get_position();
        let margin = builder.get_margin();
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
                (
                    window_bounds,
                    0.0,
                    SkylightDisplayTarget::Window(window_id),
                )
            }
        };

        // Calculate frame based on position and margin
        let frame = calculate_frame(&dimensions, &position, &margin, &bounds, top_inset);

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
            size: dimensions,
        })
    }
}

impl OsdWindow for SkylightOsdWindow {
    fn draw(self, drawable: impl Drawable) -> Self {
        let mut canvas = unsafe { SkylightCanvas::new(self.window.context_ptr(), self.size) };
        drawable.draw(&mut canvas);
        self
    }

    fn show_for_seconds(mut self, seconds: f64) -> crate::Result<()> {
        self.window.show(seconds)
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

/// Calculate window frame based on position and margin.
fn calculate_frame(
    dimensions: &Size,
    position: &FlashPosition,
    margin: &crate::geometry::Margin,
    bounds: &Rect,
    top_inset: f64,
) -> Rect {
    let size = dimensions.width; // Assuming square for now

    let origin = match position {
        FlashPosition::TopRight => Point::new(
            bounds.origin.x / 2.0 + bounds.size.width / 2.0 - size / 2.0 - margin.right / 2.0,
            bounds.origin.y / 2.0 + margin.top + top_inset,
        ),
        FlashPosition::TopLeft => Point::new(
            bounds.origin.x / 2.0 + margin.left / 2.0,
            bounds.origin.y / 2.0 + margin.top + top_inset,
        ),
        FlashPosition::BottomRight => Point::new(
            bounds.origin.x / 2.0 + bounds.size.width / 2.0 - size / 2.0 - margin.right / 2.0,
            bounds.origin.y / 2.0 + bounds.size.height / 2.0 - size / 2.0 - margin.bottom / 2.0,
        ),
        FlashPosition::BottomLeft => Point::new(
            bounds.origin.x / 2.0 + margin.left / 2.0,
            bounds.origin.y / 2.0 + bounds.size.height / 2.0 - size / 2.0 - margin.bottom / 2.0,
        ),
        FlashPosition::Center => Point::new(
            bounds.origin.x / 2.0 + bounds.size.width / 4.0 - size / 4.0,
            bounds.origin.y / 2.0 + bounds.size.height / 4.0 - size / 4.0,
        ),
        FlashPosition::Custom { x, y } => Point::new(*x, *y),
    };

    Rect::new(origin, Size::square(size)).rounded()
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
