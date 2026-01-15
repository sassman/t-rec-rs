//! Layer composition types for declarative OSD content.
//!
//! This module provides types for defining OSD content as a composition of layers.
//! Each layer can have shapes, text, styling, and animations.
//!
//! # Examples
//!
//! ```ignore
//! use osd_flash::prelude::*;
//!
//! // Create a composition with multiple layers
//! let composition = CompositionBuilder::new(80.0)
//!     .layer("glow", |l| {
//!         l.circle(44.0)
//!             .center()
//!             .fill(Color::rgba(1.0, 0.2, 0.2, 0.35))
//!             .animate(Animation::pulse_range(0.9, 1.1))
//!     })
//!     .layer("dot", |l| {
//!         l.circle(32.0)
//!             .center()
//!             .fill(Color::RED)
//!             .animate(Animation::pulse())
//!     })
//!     .build();
//! ```

pub mod animation;
pub mod layer_builder;

pub use animation::{Animation, Easing, Repeat};
pub use layer_builder::{
    FontWeight, LayerBuilder, LayerConfig, LayerPosition, ShadowConfig, ShapeKind, TextAlign,
};

use crate::geometry::Size;

/// A complete layer composition with optional animations.
///
/// Contains all the layer configurations needed to render an OSD.
/// Library types like `RecordingIndicator` implement `Into<LayerComposition>`
/// to allow them to be passed to `OsdBuilder::composition()`.
///
/// # Examples
///
/// ```ignore
/// // Create directly
/// let comp = CompositionBuilder::new(100.0)
///     .layer("circle", |l| l.circle(50.0).fill(Color::RED))
///     .build();
///
/// // From library type
/// let comp: LayerComposition = RecordingIndicator::new().into();
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct LayerComposition {
    /// Size of the composition (used for positioning).
    pub size: Size,
    /// The layers in this composition, ordered back-to-front.
    pub layers: Vec<LayerConfig>,
}

impl LayerComposition {
    /// Create a new empty composition with the given size.
    pub fn new(size: impl Into<Size>) -> Self {
        Self {
            size: size.into(),
            layers: Vec::new(),
        }
    }

    /// Create a new composition builder.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the composition (can be a single value for square)
    pub fn builder(size: impl Into<Size>) -> CompositionBuilder {
        CompositionBuilder::new(size)
    }

    /// Add a layer to this composition.
    pub fn add_layer(&mut self, layer: LayerConfig) {
        self.layers.push(layer);
    }

    /// Check if this composition has any layers.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Get the number of layers.
    pub fn len(&self) -> usize {
        self.layers.len()
    }
}

impl Default for LayerComposition {
    fn default() -> Self {
        Self::new(Size::square(100.0))
    }
}

/// Builder for creating layer compositions.
///
/// Provides a fluent API for defining compositions with multiple layers.
///
/// # Examples
///
/// ```ignore
/// let composition = CompositionBuilder::new(80.0)
///     .layer("background", |l| {
///         l.rounded_rect(80.0, 80.0, 16.0)
///             .fill(Color::rgba(0.1, 0.1, 0.1, 0.9))
///     })
///     .layer("dot", |l| {
///         l.circle(32.0)
///             .center()
///             .fill(Color::RED)
///     })
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct CompositionBuilder {
    size: Size,
    layers: Vec<LayerConfig>,
}

impl CompositionBuilder {
    /// Create a new composition builder with the given size.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the composition
    pub fn new(size: impl Into<Size>) -> Self {
        Self {
            size: size.into(),
            layers: Vec::new(),
        }
    }

    /// Add a layer to this composition.
    ///
    /// The closure receives a `LayerBuilder` for configuration.
    /// Layers are rendered in order (first layer at back, last at front).
    ///
    /// # Arguments
    ///
    /// * `name` - A unique identifier for this layer
    /// * `configure` - A closure that configures the layer
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CompositionBuilder::new(100.0)
    ///     .layer("glow", |l| {
    ///         l.circle(60.0).fill(Color::RED.with_alpha(0.3))
    ///     })
    ///     .layer("dot", |l| {
    ///         l.circle(40.0).fill(Color::RED)
    ///     })
    ///     .build()
    /// ```
    pub fn layer<F>(mut self, name: &str, configure: F) -> Self
    where
        F: FnOnce(LayerBuilder) -> LayerBuilder,
    {
        let builder = LayerBuilder::new();
        let configured = configure(builder);
        let config = configured.build(name);
        self.layers.push(config);
        self
    }

    /// Build the layer composition.
    pub fn build(self) -> LayerComposition {
        LayerComposition {
            size: self.size,
            layers: self.layers,
        }
    }
}

// Implement Into<Size> for f64 to allow square compositions
impl From<f64> for Size {
    fn from(side: f64) -> Self {
        Size::square(side)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::Color;

    #[test]
    fn test_composition_builder() {
        let comp = CompositionBuilder::new(100.0)
            .layer("test", |l| l.circle(50.0).fill(Color::RED))
            .build();

        assert_eq!(comp.size, Size::square(100.0));
        assert_eq!(comp.layers.len(), 1);
        assert_eq!(comp.layers[0].name, "test");
    }

    #[test]
    fn test_composition_builder_multiple_layers() {
        let comp = CompositionBuilder::new(Size::new(200.0, 100.0))
            .layer("back", |l| l.circle(80.0))
            .layer("front", |l| l.circle(40.0))
            .build();

        assert_eq!(comp.size, Size::new(200.0, 100.0));
        assert_eq!(comp.layers.len(), 2);
        assert_eq!(comp.layers[0].name, "back");
        assert_eq!(comp.layers[1].name, "front");
    }

    #[test]
    fn test_layer_composition_new() {
        let comp = LayerComposition::new(80.0);
        assert_eq!(comp.size, Size::square(80.0));
        assert!(comp.is_empty());
    }

    #[test]
    fn test_layer_composition_add_layer() {
        let mut comp = LayerComposition::new(100.0);
        comp.add_layer(LayerBuilder::new().circle(50.0).build("test"));

        assert_eq!(comp.len(), 1);
        assert!(!comp.is_empty());
    }

    #[test]
    fn test_size_from_f64() {
        let size: Size = 80.0.into();
        assert_eq!(size, Size::square(80.0));
    }
}
