//! Platform-agnostic window abstractions for OSD flash indicators.
//!
//! This module provides the high-level API for creating and displaying
//! on-screen indicators. The actual rendering is delegated to platform-specific
//! backends.

use std::time::Duration;

use crate::animation::animated_window::AnimatedWindow;
use crate::animation::effects::AnimationSet;
use crate::canvas::Canvas;
use crate::color::Color;
use crate::geometry::Size;
use crate::icon::Icon;
use crate::layout::{Margin, Padding};
use crate::{FlashPosition, Rect};

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
    fn draw(&self, canvas: &mut dyn Canvas, bounds: &Rect);
}

/// Configuration for GPU-accelerated animations.
///
/// This describes animations that run on the GPU compositor thread,
/// providing smoother animation than CPU-based keyframe interpolation.
#[derive(Clone, Debug)]
pub struct GpuAnimationConfig {
    /// Animation cycle duration.
    pub duration: Duration,
    /// Scale animation: (from_scale, to_scale). None to disable.
    pub scale: Option<(f64, f64)>,
    /// Glow animation (shadow-based): (color, radius, min_opacity, max_opacity). None to disable.
    /// Note: Shadow-based glow may not be visible against semi-transparent backgrounds.
    pub glow: Option<(Color, f64, f32, f32)>,
    /// Glow ring animation (shape-based): (color, radius, min_opacity, max_opacity). None to disable.
    /// This creates an actual circle layer behind the content, which is more visible than shadows.
    pub glow_ring: Option<(Color, f64, f32, f32)>,
}

impl GpuAnimationConfig {
    /// Create a new GPU animation config with the given duration.
    pub fn new(duration: Duration) -> Self {
        Self {
            duration,
            scale: None,
            glow: None,
            glow_ring: None,
        }
    }

    /// Add a scale animation.
    ///
    /// # Arguments
    ///
    /// * `from` - Starting scale (1.0 = normal size)
    /// * `to` - Ending scale
    pub fn with_scale(mut self, from: f64, to: f64) -> Self {
        self.scale = Some((from, to));
        self
    }

    /// Add a glow animation (shadow-based).
    ///
    /// Note: Shadow-based glow may not be visible against semi-transparent backgrounds.
    /// Consider using [`with_glow_ring`](Self::with_glow_ring) for better visibility.
    ///
    /// # Arguments
    ///
    /// * `color` - Glow color
    /// * `radius` - Blur radius
    /// * `min_opacity` - Minimum glow opacity (0.0 to 1.0)
    /// * `max_opacity` - Maximum glow opacity (0.0 to 1.0)
    pub fn with_glow(
        mut self,
        color: Color,
        radius: f64,
        min_opacity: f32,
        max_opacity: f32,
    ) -> Self {
        self.glow = Some((color, radius, min_opacity, max_opacity));
        self
    }

    /// Add a glow ring animation (shape-based).
    ///
    /// Unlike [`with_glow`](Self::with_glow) which uses shadows, this creates an actual
    /// circle layer that renders behind the content and pulses in opacity.
    /// This is more visible, especially against semi-transparent backgrounds.
    ///
    /// # Arguments
    ///
    /// * `color` - Glow ring color
    /// * `radius` - Radius of the glow ring circle
    /// * `min_opacity` - Minimum opacity (0.0 to 1.0)
    /// * `max_opacity` - Maximum opacity (0.0 to 1.0)
    pub fn with_glow_ring(
        mut self,
        color: Color,
        radius: f64,
        min_opacity: f32,
        max_opacity: f32,
    ) -> Self {
        self.glow_ring = Some((color, radius, min_opacity, max_opacity));
        self
    }
}

/// Trait for platform-specific OSD windows.
///
/// Backends implement this trait to provide the actual window rendering.
/// The high-level API chain is:
///
/// ```ignore
/// OsdFlashBuilder::new()
///     .build()?           // -> impl OsdWindow
///     .draw(icon)         // -> AnimatedWindow
///     .show_for_seconds() // -> Result<()>  (static display)
///
/// // Or with animation:
///     .show_animated(effects, duration) // -> Result<()>  (GPU animation)
/// ```
pub trait OsdWindow: Sized {
    /// Draw an icon and return an AnimatedWindow for display or animation.
    ///
    /// This transfers ownership of the window to the AnimatedWindow wrapper,
    /// which provides both static display and animation capabilities.
    ///
    /// Accepts any type that can be converted to an Icon, including:
    /// - `Icon` - a complete icon
    /// - `StyledShape` - a single styled shape
    /// - `Vec<StyledShape>` - multiple styled shapes
    fn draw(self, content: impl Into<Icon>) -> AnimatedWindow<Self>;

    /// Show the window (make it visible).
    fn show_window(&self) -> crate::Result<()>;

    /// Hide the window (make it invisible).
    fn hide_window(&self) -> crate::Result<()>;

    /// Draw content and show for a duration (static display).
    ///
    /// This is used for non-animated display.
    fn draw_and_show(&self, content: Icon, seconds: f64) -> crate::Result<()>;
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
    padding: Padding,
    background: Option<Color>,
    corner_radius: f64,
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
            padding: Padding::zero(),
            background: None,
            corner_radius: 0.0,
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

    /// Set the padding inside the window.
    ///
    /// Padding defines the space between the window frame and the content area.
    /// Content is drawn within the padded area, while the background fills
    /// the entire window including padding.
    ///
    /// Accepts various padding formats:
    /// - `f64`: Same padding on all sides
    /// - `(f64, f64)`: (vertical, horizontal) padding
    /// - `(f64, f64, f64, f64)`: (top, right, bottom, left) padding
    /// - `Padding`: Direct padding value
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Set the background color of the window.
    ///
    /// When set, the window will automatically draw a background rectangle
    /// covering the entire window area (including padding) with the specified
    /// color and corner radius.
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }

    /// Set the corner radius for the window background.
    ///
    /// Only applies when a background color is set via `.background()`.
    pub fn corner_radius(mut self, radius: f64) -> Self {
        self.corner_radius = radius;
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

    /// Get the configured padding.
    pub fn get_padding(&self) -> Padding {
        self.padding
    }

    /// Get the configured background color.
    pub fn get_background(&self) -> Option<Color> {
        self.background
    }

    /// Get the configured corner radius.
    pub fn get_corner_radius(&self) -> f64 {
        self.corner_radius
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
