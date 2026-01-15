//! OSD window builder - main entry point for creating OSD windows.
//!
//! The `OsdBuilder` provides a fluent API for creating on-screen display windows
//! with optional animations. It abstracts over platform-specific backends.
//!
//! # Examples
//!
//! ```ignore
//! use osd_flash::prelude::*;
//!
//! // Simple recording indicator
//! OsdBuilder::new()
//!     .size(80.0)
//!     .position(Position::TopRight)
//!     .margin(20.0)
//!     .composition(RecordingIndicator::new())
//!     .show_for(10.seconds())?;
//!
//! // Custom layer composition
//! OsdBuilder::new()
//!     .size(100.0)
//!     .position(Position::Center)
//!     .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
//!     .corner_radius(16.0)
//!     .layer("dot", |l| {
//!         l.circle(32.0)
//!             .center()
//!             .fill(Color::RED)
//!             .animate(Animation::pulse())
//!     })
//!     .show_for(5.seconds())?;
//! ```

use std::time::Duration;

use crate::color::Color;
use crate::composition::{LayerBuilder, LayerComposition, LayerConfig};
use crate::geometry::Size;
use crate::layout::Margin;
use crate::level::WindowLevel;
use crate::position::Position;
use crate::Result;

/// Configuration for an OSD window.
///
/// This struct holds all the configuration needed to create an OSD window.
/// It is passed to the backend for platform-specific window creation.
#[derive(Debug, Clone)]
pub struct OsdConfig {
    /// Window size.
    pub size: Size,
    /// Window position on screen.
    pub position: Position,
    /// Margin from screen edge.
    pub margin: Margin,
    /// Window level (z-order).
    pub level: WindowLevel,
    /// Background color.
    pub background: Option<Color>,
    /// Corner radius.
    pub corner_radius: f64,
    /// Layer configurations.
    pub layers: Vec<LayerConfig>,
}

impl Default for OsdConfig {
    fn default() -> Self {
        Self {
            size: Size::square(100.0),
            position: Position::TopRight,
            margin: Margin::all(20.0),
            level: WindowLevel::AboveAll,
            background: None,
            corner_radius: 0.0,
            layers: Vec::new(),
        }
    }
}

/// Builder for creating OSD windows.
///
/// Provides a fluent API for configuring and displaying OSD windows.
/// Content can be added via `.composition()` for pre-built compositions
/// or `.layer()` for inline layer definitions.
///
/// # Examples
///
/// ```ignore
/// use osd_flash::prelude::*;
///
/// // Using a pre-built composition
/// OsdBuilder::new()
///     .size(80.0)
///     .position(Position::TopLeft)
///     .margin(30.0)
///     .background(Color::rgba(0.1, 0.1, 0.1, 0.88))
///     .corner_radius(14.0)
///     .composition(RecordingIndicator::new())
///     .show_for(10.seconds())?;
///
/// // Using inline layers
/// OsdBuilder::new()
///     .size(100.0)
///     .layer("circle", |l| {
///         l.circle(50.0).center().fill(Color::RED)
///     })
///     .show_for(5.seconds())?;
/// ```
#[derive(Debug, Clone)]
pub struct OsdBuilder {
    config: OsdConfig,
}

impl OsdBuilder {
    /// Create a new OSD builder with default settings.
    ///
    /// Default configuration:
    /// - Size: 100x100
    /// - Position: TopRight
    /// - Margin: 20
    /// - Level: AboveAll
    /// - No background
    /// - Corner radius: 0
    pub fn new() -> Self {
        Self {
            config: OsdConfig::default(),
        }
    }

    // =========================================================================
    // Dimension methods
    // =========================================================================

    /// Set the window size.
    ///
    /// Accepts `f64` for square windows or `Size` for rectangular.
    ///
    /// # Arguments
    ///
    /// * `size` - The window size
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Square window
    /// OsdBuilder::new().size(100.0)
    ///
    /// // Rectangular window
    /// OsdBuilder::new().size(Size::new(200.0, 100.0))
    /// ```
    pub fn size(mut self, size: impl Into<Size>) -> Self {
        self.config.size = size.into();
        self
    }

    /// Set the window size with explicit width and height.
    ///
    /// # Arguments
    ///
    /// * `width` - Window width in points
    /// * `height` - Window height in points
    pub fn dimensions(mut self, width: f64, height: f64) -> Self {
        self.config.size = Size::new(width, height);
        self
    }

    // =========================================================================
    // Positioning methods
    // =========================================================================

    /// Set the window position on screen.
    ///
    /// # Arguments
    ///
    /// * `pos` - The position (TopRight, TopLeft, Center, etc.)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// OsdBuilder::new()
    ///     .position(Position::TopRight)
    ///     .margin(20.0)
    /// ```
    pub fn position(mut self, pos: Position) -> Self {
        self.config.position = pos;
        self
    }

    /// Set the margin from the screen edge.
    ///
    /// For corner positions (TopRight, TopLeft, etc.), this is the distance
    /// from the screen edge. For Center, margin is ignored.
    ///
    /// # Arguments
    ///
    /// * `margin` - The margin (accepts f64, (f64, f64), or Margin)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Uniform margin
    /// OsdBuilder::new().margin(20.0)
    ///
    /// // Asymmetric margin (vertical, horizontal)
    /// OsdBuilder::new().margin((30.0, 20.0))
    /// ```
    pub fn margin(mut self, margin: impl Into<Margin>) -> Self {
        self.config.margin = margin.into();
        self
    }

    /// Set the window level (z-order).
    ///
    /// Controls where the window appears relative to other windows.
    ///
    /// # Arguments
    ///
    /// * `level` - The window level
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Above all windows including fullscreen apps
    /// OsdBuilder::new().level(WindowLevel::AboveAll)
    ///
    /// // Normal window level
    /// OsdBuilder::new().level(WindowLevel::Normal)
    /// ```
    pub fn level(mut self, level: WindowLevel) -> Self {
        self.config.level = level;
        self
    }

    // =========================================================================
    // Styling methods
    // =========================================================================

    /// Set the background color.
    ///
    /// If not set, the window will have a transparent background.
    ///
    /// # Arguments
    ///
    /// * `color` - The background color
    ///
    /// # Examples
    ///
    /// ```ignore
    /// OsdBuilder::new()
    ///     .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
    /// ```
    pub fn background(mut self, color: Color) -> Self {
        self.config.background = Some(color);
        self
    }

    /// Set the corner radius.
    ///
    /// Only has visual effect when a background color is set.
    ///
    /// # Arguments
    ///
    /// * `radius` - The corner radius in points
    pub fn corner_radius(mut self, radius: f64) -> Self {
        self.config.corner_radius = radius;
        self
    }

    // =========================================================================
    // Content methods
    // =========================================================================

    /// Set the content from a composition.
    ///
    /// Accepts any type that implements `Into<LayerComposition>`, including:
    /// - `LayerComposition` directly
    /// - Library types like `RecordingIndicator`
    ///
    /// This replaces any previously added layers.
    ///
    /// # Arguments
    ///
    /// * `comp` - The composition
    ///
    /// # Examples
    ///
    /// ```ignore
    /// OsdBuilder::new()
    ///     .composition(RecordingIndicator::new())
    ///     .show_for(10.seconds())?;
    /// ```
    pub fn composition(mut self, comp: impl Into<LayerComposition>) -> Self {
        let composition = comp.into();
        // Use the composition's size if not explicitly set
        if self.config.size == Size::square(100.0) {
            self.config.size = composition.size;
        }
        self.config.layers = composition.layers;
        self
    }

    /// Add a layer to the window.
    ///
    /// Layers are rendered in order (first layer at back, last at front).
    /// Can be called multiple times to add multiple layers.
    ///
    /// # Arguments
    ///
    /// * `name` - A unique identifier for this layer
    /// * `configure` - A closure that configures the layer
    ///
    /// # Examples
    ///
    /// ```ignore
    /// OsdBuilder::new()
    ///     .size(100.0)
    ///     .layer("glow", |l| {
    ///         l.circle(60.0)
    ///             .center()
    ///             .fill(Color::RED.with_alpha(0.3))
    ///             .animate(Animation::pulse_range(0.9, 1.2))
    ///     })
    ///     .layer("dot", |l| {
    ///         l.circle(40.0)
    ///             .center()
    ///             .fill(Color::RED)
    ///             .animate(Animation::pulse())
    ///     })
    ///     .show_for(5.seconds())?;
    /// ```
    pub fn layer<F>(mut self, name: &str, configure: F) -> Self
    where
        F: FnOnce(LayerBuilder) -> LayerBuilder,
    {
        let builder = LayerBuilder::new();
        let configured = configure(builder);
        let layer_config = configured.build(name);
        self.config.layers.push(layer_config);
        self
    }

    // =========================================================================
    // Display methods
    // =========================================================================

    /// Display the OSD for the specified duration.
    ///
    /// This is the primary way to show an OSD. It creates the window,
    /// displays it for the duration, and then closes it.
    ///
    /// Animations defined on layers will run automatically.
    ///
    /// # Arguments
    ///
    /// * `duration` - How long to display the OSD
    ///
    /// # Errors
    ///
    /// Returns an error if the window cannot be created.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use osd_flash::prelude::*;
    ///
    /// OsdBuilder::new()
    ///     .size(80.0)
    ///     .composition(RecordingIndicator::new())
    ///     .show_for(10.seconds())?;
    /// ```
    #[cfg(target_os = "macos")]
    pub fn show_for(self, duration: Duration) -> Result<()> {
        use crate::backends::macos::MacOsWindow;

        let window = MacOsWindow::from_config(self.config)?;
        window.show_for(duration);
        Ok(())
    }

    /// Display the OSD for the specified duration.
    ///
    /// On unsupported platforms, this is a no-op.
    #[cfg(not(target_os = "macos"))]
    pub fn show_for(self, _duration: Duration) -> Result<()> {
        // No-op on unsupported platforms
        Ok(())
    }

    /// Build the OSD window without displaying it.
    ///
    /// Returns a handle that can be used to show/hide the window manually.
    ///
    /// # Errors
    ///
    /// Returns an error if the window cannot be created.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let window = OsdBuilder::new()
    ///     .size(80.0)
    ///     .composition(RecordingIndicator::new())
    ///     .build()?;
    ///
    /// window.show();
    /// // ... do other work ...
    /// window.hide();
    /// ```
    #[cfg(target_os = "macos")]
    pub fn build(self) -> Result<crate::backends::macos::MacOsWindow> {
        crate::backends::macos::MacOsWindow::from_config(self.config)
    }

    /// Build the OSD window without displaying it.
    ///
    /// On unsupported platforms, returns a stub window.
    #[cfg(not(target_os = "macos"))]
    pub fn build(self) -> Result<StubWindow> {
        Ok(StubWindow)
    }

    /// Get the current configuration.
    ///
    /// Useful for testing or passing to backends directly.
    pub fn into_config(self) -> OsdConfig {
        self.config
    }
}

impl Default for OsdBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Stub window for unsupported platforms.
#[cfg(not(target_os = "macos"))]
pub struct StubWindow;

#[cfg(not(target_os = "macos"))]
impl StubWindow {
    /// Show the window (no-op).
    pub fn show(&self) {}
    /// Hide the window (no-op).
    pub fn hide(&self) {}
    /// Show the window for a duration (no-op).
    pub fn show_for(&self, _duration: Duration) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::composition::Animation;

    #[test]
    fn test_builder_defaults() {
        let config = OsdBuilder::new().into_config();
        assert_eq!(config.size, Size::square(100.0));
        assert_eq!(config.position, Position::TopRight);
        assert_eq!(config.level, WindowLevel::AboveAll);
        assert!(config.background.is_none());
        assert_eq!(config.corner_radius, 0.0);
        assert!(config.layers.is_empty());
    }

    #[test]
    fn test_builder_size() {
        let config = OsdBuilder::new().size(80.0).into_config();
        assert_eq!(config.size, Size::square(80.0));
    }

    #[test]
    fn test_builder_dimensions() {
        let config = OsdBuilder::new().dimensions(200.0, 100.0).into_config();
        assert_eq!(config.size, Size::new(200.0, 100.0));
    }

    #[test]
    fn test_builder_position() {
        let config = OsdBuilder::new().position(Position::Center).into_config();
        assert_eq!(config.position, Position::Center);
    }

    #[test]
    fn test_builder_margin() {
        let config = OsdBuilder::new().margin(30.0).into_config();
        assert_eq!(config.margin, Margin::all(30.0));
    }

    #[test]
    fn test_builder_background() {
        let config = OsdBuilder::new()
            .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
            .into_config();
        assert!(config.background.is_some());
    }

    #[test]
    fn test_builder_corner_radius() {
        let config = OsdBuilder::new().corner_radius(16.0).into_config();
        assert_eq!(config.corner_radius, 16.0);
    }

    #[test]
    fn test_builder_layer() {
        let config = OsdBuilder::new()
            .size(100.0)
            .layer("dot", |l| l.circle(40.0).center().fill(Color::RED))
            .into_config();
        assert_eq!(config.layers.len(), 1);
        assert_eq!(config.layers[0].name, "dot");
    }

    #[test]
    fn test_builder_multiple_layers() {
        let config = OsdBuilder::new()
            .size(100.0)
            .layer("glow", |l| {
                l.circle(60.0)
                    .center()
                    .fill(Color::RED.with_alpha(0.3))
                    .animate(Animation::pulse_range(0.9, 1.2))
            })
            .layer("dot", |l| {
                l.circle(40.0)
                    .center()
                    .fill(Color::RED)
                    .animate(Animation::pulse())
            })
            .into_config();
        assert_eq!(config.layers.len(), 2);
        assert_eq!(config.layers[0].name, "glow");
        assert_eq!(config.layers[1].name, "dot");
    }

    #[test]
    fn test_builder_composition() {
        use crate::composition::CompositionBuilder;

        let comp = CompositionBuilder::new(80.0)
            .layer("test", |l| l.circle(40.0).fill(Color::RED))
            .build();

        let config = OsdBuilder::new().composition(comp).into_config();
        assert_eq!(config.size, Size::square(80.0)); // Size from composition
        assert_eq!(config.layers.len(), 1);
    }
}
