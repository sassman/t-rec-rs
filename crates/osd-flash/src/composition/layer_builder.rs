//! Layer builder for declarative layer configuration.

use crate::color::Color;
use crate::composition::animation::Animation;

/// Position of a layer within its parent composition.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum LayerPosition {
    /// Centered within the parent (default).
    #[default]
    Center,
    /// Centered with an offset from the center.
    CenterOffset { dx: f64, dy: f64 },
    /// Absolute position relative to parent's origin.
    Absolute { x: f64, y: f64 },
}

/// The kind of shape for a layer.
#[derive(Debug, Clone, PartialEq)]
pub enum ShapeKind {
    /// A perfect circle.
    Circle { diameter: f64 },
    /// An ellipse (oval).
    Ellipse { width: f64, height: f64 },
    /// A rectangle with rounded corners.
    RoundedRect {
        width: f64,
        height: f64,
        corner_radius: f64,
    },
}

/// Configuration for a layer's shadow effect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadowConfig {
    /// Shadow color.
    pub color: Color,
    /// Shadow blur radius.
    pub radius: f64,
    /// Shadow offset (dx, dy).
    pub offset: (f64, f64),
    /// Shadow opacity (0.0 to 1.0).
    pub opacity: f32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            color: Color::BLACK,
            radius: 10.0,
            offset: (0.0, 0.0),
            opacity: 0.5,
        }
    }
}

/// Text alignment within layer bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlign {
    Left,
    #[default]
    Center,
    Right,
}

/// Font weight for text layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontWeight {
    Thin,
    Light,
    #[default]
    Regular,
    Medium,
    Semibold,
    Bold,
    Heavy,
    Black,
}

/// Configuration for a single layer.
///
/// Contains all the properties needed to create a layer in the backend.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerConfig {
    /// Layer name (used as animation key).
    pub name: String,
    /// The shape to render (None for text-only layers).
    pub shape: Option<ShapeKind>,
    /// Position within parent.
    pub position: LayerPosition,
    /// Fill color.
    pub fill: Option<Color>,
    /// Stroke color and width.
    pub stroke: Option<(Color, f64)>,
    /// Layer opacity (0.0 to 1.0).
    pub opacity: f32,
    /// Shadow/glow configuration.
    pub shadow: Option<ShadowConfig>,
    /// Animations to apply.
    pub animations: Vec<Animation>,
    /// Text content (for text layers).
    pub text: Option<String>,
    /// Font size in points.
    pub font_size: f64,
    /// Font weight.
    pub font_weight: FontWeight,
    /// Font family name.
    pub font_family: Option<String>,
    /// Text color (uses fill color if not set).
    pub text_color: Option<Color>,
    /// Text alignment.
    pub text_align: TextAlign,
    /// Text opacity.
    pub text_opacity: Option<f32>,
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            shape: None,
            position: LayerPosition::Center,
            fill: None,
            stroke: None,
            opacity: 1.0,
            shadow: None,
            animations: Vec::new(),
            text: None,
            font_size: 16.0,
            font_weight: FontWeight::Regular,
            font_family: None,
            text_color: None,
            text_align: TextAlign::Center,
            text_opacity: None,
        }
    }
}

/// Builder for configuring individual layers.
///
/// Provides a fluent API for defining layer properties including shape,
/// position, styling, and animations.
///
/// # Examples
///
/// ```ignore
/// LayerBuilder::new()
///     .circle(32.0)
///     .center()
///     .fill(Color::RED)
///     .animate(Animation::pulse())
///     .build("dot")
/// ```
#[derive(Debug, Clone, Default)]
pub struct LayerBuilder {
    config: LayerConfig,
}

impl LayerBuilder {
    /// Create a new layer builder.
    pub fn new() -> Self {
        Self::default()
    }

    // =========================================================================
    // Shape methods
    // =========================================================================

    /// Set the shape to a circle.
    ///
    /// # Arguments
    ///
    /// * `diameter` - The diameter of the circle
    pub fn circle(mut self, diameter: f64) -> Self {
        self.config.shape = Some(ShapeKind::Circle { diameter });
        self
    }

    /// Set the shape to an ellipse.
    ///
    /// # Arguments
    ///
    /// * `width` - The width of the ellipse
    /// * `height` - The height of the ellipse
    pub fn ellipse(mut self, width: f64, height: f64) -> Self {
        self.config.shape = Some(ShapeKind::Ellipse { width, height });
        self
    }

    /// Set the shape to a rounded rectangle.
    ///
    /// # Arguments
    ///
    /// * `width` - The width of the rectangle
    /// * `height` - The height of the rectangle
    /// * `corner_radius` - The corner radius
    pub fn rounded_rect(mut self, width: f64, height: f64, corner_radius: f64) -> Self {
        self.config.shape = Some(ShapeKind::RoundedRect {
            width,
            height,
            corner_radius,
        });
        self
    }

    // =========================================================================
    // Position methods
    // =========================================================================

    /// Set the position to absolute coordinates.
    ///
    /// # Arguments
    ///
    /// * `x` - Horizontal position
    /// * `y` - Vertical position
    pub fn position(mut self, x: f64, y: f64) -> Self {
        self.config.position = LayerPosition::Absolute { x, y };
        self
    }

    /// Center the layer within its parent.
    pub fn center(mut self) -> Self {
        self.config.position = LayerPosition::Center;
        self
    }

    /// Center the layer with an offset.
    ///
    /// # Arguments
    ///
    /// * `dx` - Horizontal offset from center
    /// * `dy` - Vertical offset from center
    pub fn center_offset(mut self, dx: f64, dy: f64) -> Self {
        self.config.position = LayerPosition::CenterOffset { dx, dy };
        self
    }

    // =========================================================================
    // Style methods
    // =========================================================================

    /// Set the fill color.
    ///
    /// # Arguments
    ///
    /// * `color` - The fill color
    pub fn fill(mut self, color: Color) -> Self {
        self.config.fill = Some(color);
        self
    }

    /// Set the stroke color and width.
    ///
    /// # Arguments
    ///
    /// * `color` - The stroke color
    /// * `width` - The stroke width
    pub fn stroke(mut self, color: Color, width: f64) -> Self {
        self.config.stroke = Some((color, width));
        self
    }

    /// Set the layer opacity.
    ///
    /// # Arguments
    ///
    /// * `opacity` - Opacity value (0.0 = transparent, 1.0 = opaque)
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.config.opacity = opacity;
        self
    }

    // =========================================================================
    // Shadow methods
    // =========================================================================

    /// Add a shadow/glow effect.
    ///
    /// # Arguments
    ///
    /// * `color` - Shadow color
    /// * `radius` - Blur radius
    pub fn shadow(mut self, color: Color, radius: f64) -> Self {
        self.config.shadow = Some(ShadowConfig {
            color,
            radius,
            offset: (0.0, 0.0),
            opacity: 1.0,
        });
        self
    }

    /// Set the shadow offset.
    ///
    /// Must be called after `shadow()`.
    ///
    /// # Arguments
    ///
    /// * `dx` - Horizontal offset
    /// * `dy` - Vertical offset
    pub fn shadow_offset(mut self, dx: f64, dy: f64) -> Self {
        if let Some(ref mut shadow) = self.config.shadow {
            shadow.offset = (dx, dy);
        }
        self
    }

    /// Set the shadow opacity.
    ///
    /// Must be called after `shadow()`.
    ///
    /// # Arguments
    ///
    /// * `opacity` - Shadow opacity (0.0 to 1.0)
    pub fn shadow_opacity(mut self, opacity: f32) -> Self {
        if let Some(ref mut shadow) = self.config.shadow {
            shadow.opacity = opacity;
        }
        self
    }

    // =========================================================================
    // Animation methods
    // =========================================================================

    /// Add an animation to this layer.
    ///
    /// Multiple animations can be added and will run in parallel.
    ///
    /// # Arguments
    ///
    /// * `animation` - The animation to apply
    pub fn animate(mut self, animation: Animation) -> Self {
        self.config.animations.push(animation);
        self
    }

    // =========================================================================
    // Text methods
    // =========================================================================

    /// Set text content for this layer.
    ///
    /// When text is set, the layer becomes a text layer.
    ///
    /// # Arguments
    ///
    /// * `content` - The text to display
    pub fn text(mut self, content: impl Into<String>) -> Self {
        self.config.text = Some(content.into());
        self
    }

    /// Set the font size.
    ///
    /// # Arguments
    ///
    /// * `size` - Font size in points
    pub fn font_size(mut self, size: f64) -> Self {
        self.config.font_size = size;
        self
    }

    /// Set the font weight.
    ///
    /// # Arguments
    ///
    /// * `weight` - The font weight
    pub fn font_weight(mut self, weight: FontWeight) -> Self {
        self.config.font_weight = weight;
        self
    }

    /// Set the font family.
    ///
    /// # Arguments
    ///
    /// * `family` - Font family name (e.g., "Helvetica Neue", "SF Mono")
    pub fn font_family(mut self, family: impl Into<String>) -> Self {
        self.config.font_family = Some(family.into());
        self
    }

    /// Set the text color.
    ///
    /// If not set, uses the fill color.
    ///
    /// # Arguments
    ///
    /// * `color` - The text color
    pub fn text_color(mut self, color: Color) -> Self {
        self.config.text_color = Some(color);
        self
    }

    /// Set the text alignment.
    ///
    /// # Arguments
    ///
    /// * `align` - The text alignment
    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.config.text_align = align;
        self
    }

    /// Set the text opacity.
    ///
    /// # Arguments
    ///
    /// * `opacity` - Text opacity (0.0 to 1.0)
    pub fn text_opacity(mut self, opacity: f32) -> Self {
        self.config.text_opacity = Some(opacity);
        self
    }

    // =========================================================================
    // Build methods
    // =========================================================================

    /// Build the layer configuration with the given name.
    ///
    /// # Arguments
    ///
    /// * `name` - A unique identifier for this layer
    pub fn build(mut self, name: impl Into<String>) -> LayerConfig {
        self.config.name = name.into();
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle() {
        let layer = LayerBuilder::new().circle(50.0).build("test");
        assert_eq!(layer.shape, Some(ShapeKind::Circle { diameter: 50.0 }));
    }

    #[test]
    fn test_ellipse() {
        let layer = LayerBuilder::new().ellipse(100.0, 50.0).build("test");
        assert_eq!(
            layer.shape,
            Some(ShapeKind::Ellipse {
                width: 100.0,
                height: 50.0
            })
        );
    }

    #[test]
    fn test_rounded_rect() {
        let layer = LayerBuilder::new()
            .rounded_rect(100.0, 50.0, 10.0)
            .build("test");
        assert_eq!(
            layer.shape,
            Some(ShapeKind::RoundedRect {
                width: 100.0,
                height: 50.0,
                corner_radius: 10.0
            })
        );
    }

    #[test]
    fn test_position() {
        let layer = LayerBuilder::new().position(10.0, 20.0).build("test");
        assert_eq!(layer.position, LayerPosition::Absolute { x: 10.0, y: 20.0 });
    }

    #[test]
    fn test_center() {
        let layer = LayerBuilder::new().center().build("test");
        assert_eq!(layer.position, LayerPosition::Center);
    }

    #[test]
    fn test_center_offset() {
        let layer = LayerBuilder::new().center_offset(5.0, -5.0).build("test");
        assert_eq!(
            layer.position,
            LayerPosition::CenterOffset { dx: 5.0, dy: -5.0 }
        );
    }

    #[test]
    fn test_fill() {
        let layer = LayerBuilder::new().fill(Color::RED).build("test");
        assert_eq!(layer.fill, Some(Color::RED));
    }

    #[test]
    fn test_stroke() {
        let layer = LayerBuilder::new().stroke(Color::WHITE, 2.0).build("test");
        assert_eq!(layer.stroke, Some((Color::WHITE, 2.0)));
    }

    #[test]
    fn test_opacity() {
        let layer = LayerBuilder::new().opacity(0.5).build("test");
        assert_eq!(layer.opacity, 0.5);
    }

    #[test]
    fn test_shadow() {
        let layer = LayerBuilder::new()
            .shadow(Color::BLACK, 10.0)
            .shadow_offset(2.0, 4.0)
            .shadow_opacity(0.8)
            .build("test");

        let shadow = layer.shadow.unwrap();
        assert_eq!(shadow.color, Color::BLACK);
        assert_eq!(shadow.radius, 10.0);
        assert_eq!(shadow.offset, (2.0, 4.0));
        assert_eq!(shadow.opacity, 0.8);
    }

    #[test]
    fn test_animate() {
        let layer = LayerBuilder::new()
            .animate(Animation::pulse())
            .animate(Animation::fade_in())
            .build("test");
        assert_eq!(layer.animations.len(), 2);
    }

    #[test]
    fn test_text() {
        let layer = LayerBuilder::new()
            .text("Hello")
            .font_size(24.0)
            .font_weight(FontWeight::Bold)
            .text_color(Color::WHITE)
            .build("test");

        assert_eq!(layer.text, Some("Hello".to_string()));
        assert_eq!(layer.font_size, 24.0);
        assert_eq!(layer.font_weight, FontWeight::Bold);
        assert_eq!(layer.text_color, Some(Color::WHITE));
    }
}
