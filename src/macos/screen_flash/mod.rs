//! Screenshot visual feedback for macOS.
//!
//! Shows a brief visual indicator when a screenshot is taken.
//! Uses private SkyLight APIs for overlay windows.
//!
//! # Architecture
//!
//! - `geometry` - Core geometry types (Rect, Point, Size)
//! - `color` - RGBA color representation
//! - `drawing` - Drawing primitives and canvas abstraction
//! - `icon` - Icon builder for composing visual elements
//! - `skylight` - macOS SkyLight private API bindings
//!
//! # Example
//!
//! ```ignore
//! use screen_flash::{IconBuilder, Color, Position};
//!
//! let icon = IconBuilder::new(120.0)
//!     .background(Color::rgba(0.15, 0.45, 0.9, 0.92))
//!     .corner_radius(16.0)
//!     .add_shape(Shape::rounded_rect(...))
//!     .add_shape(Shape::circle(...))
//!     .build();
//! ```

mod color;
mod drawing;
mod geometry;
mod icon;
mod skylight;

pub use color::Color;
pub use drawing::{Canvas, Shape};
pub use geometry::{Point, Rect, Size};
pub use icon::IconBuilder;
pub use skylight::{DisplayTarget, SkylightWindow, SkylightWindowBuilder, WindowLevel};

pub use skylight::run_loop_for_seconds;

/// Position for the flash indicator on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlashPosition {
    /// Top-right corner (default, like macOS notifications)
    TopRight,
    /// Top-left corner
    TopLeft,
    /// Bottom-right corner
    BottomRight,
    /// Bottom-left corner
    BottomLeft,
    /// Center of screen
    Center,
    /// Custom position (x, y from top-left)
    Custom { x: f64, y: f64 },
}

impl Default for FlashPosition {
    fn default() -> Self {
        Self::TopRight
    }
}

/// Configuration for the screen flash.
#[derive(Debug, Clone)]
pub struct FlashConfig {
    /// Size of the icon in points
    pub icon_size: f64,
    /// Position on screen
    pub position: FlashPosition,
    /// Duration to show the flash in seconds
    pub duration_secs: f64,
    /// Margin from screen edge in points
    pub margin: f64,
}

impl Default for FlashConfig {
    fn default() -> Self {
        Self {
            icon_size: 120.0,
            position: FlashPosition::TopRight,
            duration_secs: 0.5,
            margin: 20.0,
        }
    }
}

impl FlashConfig {
    /// Create a new flash config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the icon size.
    pub fn icon_size(mut self, size: f64) -> Self {
        self.icon_size = size;
        self
    }

    /// Set the position.
    pub fn position(mut self, pos: FlashPosition) -> Self {
        self.position = pos;
        self
    }

    /// Set the duration in seconds.
    pub fn duration(mut self, secs: f64) -> Self {
        self.duration_secs = secs;
        self
    }

    /// Set the margin from screen edge.
    pub fn margin(mut self, margin: f64) -> Self {
        self.margin = margin;
        self
    }
}

/// Show visual feedback for screenshot capture.
///
/// Displays a brief camera icon at the configured position.
///
/// # Note
/// Must run on main thread for SkyLight windows to appear (run loop requirement).
pub fn flash_screenshot(config: &FlashConfig, win_id: u64) {
    if let Err(e) = show_indicator_screenshot_indicator(config, win_id) {
        log::warn!("Failed to show screenshot indicator: {}", e);
    }
}

/// Show visual feedback with default configuration.
pub fn flash_screenshot_default(win_id: u64) {
    flash_screenshot(&FlashConfig::default(), win_id);
}

pub fn show_indicator_screenshot_indicator(config: &FlashConfig, win_id: u64) -> crate::Result<()> {
    // Build the camera icon
    let icon = icon::camera_icon(config.icon_size);

    // Create and show the SkyLight window
    let mut window =
        SkylightWindowBuilder::from_config_with_target(config, DisplayTarget::Window(win_id))
            .level(WindowLevel::AboveAll)
            .build()?;
    window.draw(&icon)?;
    window.show(config.duration_secs)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flash_config_default() {
        let config = FlashConfig::default();
        assert_eq!(config.icon_size, 120.0);
        assert_eq!(config.position, FlashPosition::TopRight);
        assert_eq!(config.duration_secs, 3.0);
    }

    #[test]
    fn test_flash_config_builder() {
        let config = FlashConfig::new()
            .icon_size(80.0)
            .position(FlashPosition::Center)
            .duration(1.0)
            .margin(30.0);

        assert_eq!(config.icon_size, 80.0);
        assert_eq!(config.position, FlashPosition::Center);
        assert_eq!(config.duration_secs, 1.0);
        assert_eq!(config.margin, 30.0);
    }

    #[test]
    fn test_flash_position_default() {
        assert_eq!(FlashPosition::default(), FlashPosition::TopRight);
    }
}
