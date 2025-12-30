//! Platform-agnostic window abstractions for OSD flash indicators.
//!
//! This module provides the high-level API for creating and displaying
//! on-screen indicators. The actual rendering is delegated to platform-specific
//! backends.

use crate::canvas::Canvas;
use crate::geometry::{Margin, Size};
use crate::icon::Icon;
use crate::FlashPosition;

/// Target for the OSD window display.
///
/// Controls which display or window the OSD appears relative to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayTarget {
    /// The main/primary display (default).
    #[default]
    Main,
    /// The display containing a specific window.
    /// The OSD will appear relative to this window's position.
    Window(u64),
}

impl From<u64> for DisplayTarget {
    fn from(window_id: u64) -> Self {
        DisplayTarget::Window(window_id)
    }
}

/// Window level determining the z-order of the overlay window.
///
/// Controls where the window appears in the window stack relative to other windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowLevel {
    /// Normal window level, appears with regular application windows.
    Normal,
    /// Floating window level, appears above normal windows but below screen savers.
    Floating,
    /// Modal panel level, appears above floating windows.
    ModalPanel,
    /// Above all other windows, including fullscreen apps and the Dock.
    #[default]
    AboveAll,
    /// Custom window level value (platform-specific interpretation).
    Custom(i32),
}

/// Trait for types that can be drawn onto a canvas.
///
/// This allows the window API to accept any drawable type, not just `Icon`.
pub trait Drawable {
    /// Draw this object onto the given canvas.
    fn draw(&self, canvas: &mut dyn Canvas);
}

impl Drawable for Icon {
    fn draw(&self, canvas: &mut dyn Canvas) {
        canvas.clear();
        canvas.draw_shapes(&self.shapes);
        canvas.flush();
    }
}

/// Trait for platform-specific OSD windows.
///
/// Backends implement this trait to provide the actual window rendering.
pub trait OsdWindow: Sized {
    /// Draw a drawable object onto the window.
    fn draw(self, drawable: impl Drawable) -> Self;

    /// Show the window for the specified duration in seconds.
    fn show_for_seconds(self, seconds: f64) -> crate::Result<()>;
}

/// Builder for creating OSD flash windows.
///
/// This provides a platform-agnostic API for creating overlay windows.
/// The actual window creation is delegated to the appropriate backend
/// based on the current platform.
///
/// # Example
///
/// ```ignore
/// use osd_flash::prelude::*;
///
/// OsdFlashBuilder::new()
///     .dimensions(120.0)
///     .position(FlashPosition::TopRight)
///     .margin(20.0)
///     .level(WindowLevel::AboveAll)
///     .build()?
///     .draw(CameraIcon::new(120.0).build())
///     .show_for_seconds(1.5)?;
/// ```
#[derive(Debug, Clone)]
pub struct OsdFlashBuilder {
    dimensions: Size,
    position: FlashPosition,
    margin: Margin,
    level: WindowLevel,
    display_target: DisplayTarget,
}

impl Default for OsdFlashBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OsdFlashBuilder {
    /// Create a new OSD flash builder with default settings.
    pub fn new() -> Self {
        Self {
            dimensions: Size::square(120.0),
            position: FlashPosition::TopRight,
            margin: Margin::all(20.0),
            level: WindowLevel::AboveAll,
            display_target: DisplayTarget::Main,
        }
    }

    /// Set the window dimensions.
    ///
    /// Accepts any type that can be converted to `Size`:
    /// - `f64`: Creates a square window
    /// - `Size`: Uses the exact dimensions
    pub fn dimensions(mut self, size: impl Into<Size>) -> Self {
        self.dimensions = size.into();
        self
    }

    /// Set the window position on screen.
    pub fn position(mut self, position: FlashPosition) -> Self {
        self.position = position;
        self
    }

    /// Set the margin from screen edges.
    ///
    /// Accepts various margin formats:
    /// - `f64`: Same margin on all sides
    /// - `(f64, f64)`: (vertical, horizontal) margins
    /// - `(f64, f64, f64, f64)`: (top, right, bottom, left) margins
    /// - `Margin`: Direct margin value
    pub fn margin(mut self, margin: impl Into<Margin>) -> Self {
        self.margin = margin.into();
        self
    }

    /// Set the window level (z-order).
    pub fn level(mut self, level: WindowLevel) -> Self {
        self.level = level;
        self
    }

    /// Attach the OSD to a specific window or display target.
    ///
    /// When attached to a window, the OSD will appear on the same display
    /// as the target window, positioned relative to that window's bounds.
    ///
    /// Accepts:
    /// - `u64`: Window ID to attach to
    /// - `DisplayTarget`: Explicit display target
    pub fn attach_to_window(mut self, target: impl Into<DisplayTarget>) -> Self {
        self.display_target = target.into();
        self
    }

    /// Get the configured dimensions.
    pub fn get_dimensions(&self) -> Size {
        self.dimensions
    }

    /// Get the configured position.
    pub fn get_position(&self) -> FlashPosition {
        self.position
    }

    /// Get the configured margin.
    pub fn get_margin(&self) -> Margin {
        self.margin
    }

    /// Get the configured window level.
    pub fn get_level(&self) -> WindowLevel {
        self.level
    }

    /// Get the configured display target.
    pub fn get_display_target(&self) -> DisplayTarget {
        self.display_target
    }

    /// Build the OSD window using the platform-specific backend.
    #[cfg(target_os = "macos")]
    pub fn build(self) -> crate::Result<impl OsdWindow> {
        use crate::backends::skylight::SkylightOsdWindow;
        SkylightOsdWindow::from_builder(self)
    }

    /// Build the OSD window (stub for non-macOS platforms).
    #[cfg(not(target_os = "macos"))]
    pub fn build(self) -> crate::Result<impl OsdWindow> {
        Err(anyhow::anyhow!(
            "OSD flash is not yet supported on this platform"
        ))
    }
}

// Implement From<f64> for Size to allow `.dimensions(120.0)`
impl From<f64> for Size {
    fn from(value: f64) -> Self {
        Size::square(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_defaults() {
        let builder = OsdFlashBuilder::new();
        assert_eq!(builder.dimensions, Size::square(120.0));
        assert_eq!(builder.position, FlashPosition::TopRight);
        assert_eq!(builder.margin, Margin::all(20.0));
        assert_eq!(builder.level, WindowLevel::AboveAll);
    }

    #[test]
    fn test_builder_chain() {
        let builder = OsdFlashBuilder::new()
            .dimensions(80.0)
            .position(FlashPosition::Center)
            .margin(15.0)
            .level(WindowLevel::Floating);

        assert_eq!(builder.dimensions, Size::square(80.0));
        assert_eq!(builder.position, FlashPosition::Center);
        assert_eq!(builder.margin, Margin::all(15.0));
        assert_eq!(builder.level, WindowLevel::Floating);
    }

    #[test]
    fn test_dimensions_from_f64() {
        let size: Size = 100.0.into();
        assert_eq!(size, Size::square(100.0));
    }

    #[test]
    fn test_window_level_default() {
        assert_eq!(WindowLevel::default(), WindowLevel::AboveAll);
    }
}
