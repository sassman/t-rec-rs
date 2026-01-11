//! Builder for `CATextLayer` (text rendering layer).

use crate::animation_builder::{CABasicAnimationBuilder, KeyPath};
use crate::color::Color;
use objc2::rc::Retained;
use objc2_core_foundation::{CFRetained, CFString, CGFloat, CGPoint, CGRect, CGSize};
use objc2_core_graphics::CGColor;
use objc2_core_text::CTFont;
use objc2_foundation::NSString;
use objc2_quartz_core::{
    kCAAlignmentCenter, kCAAlignmentJustified, kCAAlignmentLeft, kCAAlignmentNatural,
    kCAAlignmentRight, kCATruncationEnd, kCATruncationMiddle, kCATruncationNone, kCATruncationStart,
    CABasicAnimation, CATextLayer, CATransform3D,
};

/// A pending animation to be applied when the layer is built.
struct PendingAnimation {
    name: String,
    animation: Retained<CABasicAnimation>,
}

/// Text alignment modes for `CATextLayer`.
///
/// These map to Core Animation's text alignment constants.
///
/// # Examples
///
/// ```ignore
/// CATextLayerBuilder::new()
///     .text("Hello, World!")
///     .alignment(TextAlign::Center)
///     .build();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextAlign {
    /// Natural alignment based on the localization setting of the system.
    /// This is the default alignment.
    #[default]
    Natural,
    /// Left-aligned text.
    Left,
    /// Right-aligned text.
    Right,
    /// Center-aligned text.
    Center,
    /// Justified text (both left and right edges aligned).
    Justified,
}

impl TextAlign {
    /// Returns the Core Animation alignment mode string for this alignment.
    fn to_ca_alignment(&self) -> &'static NSString {
        // SAFETY: These extern statics are always valid on macOS.
        unsafe {
            match self {
                TextAlign::Natural => kCAAlignmentNatural,
                TextAlign::Left => kCAAlignmentLeft,
                TextAlign::Right => kCAAlignmentRight,
                TextAlign::Center => kCAAlignmentCenter,
                TextAlign::Justified => kCAAlignmentJustified,
            }
        }
    }
}

/// Truncation modes for `CATextLayer`.
///
/// These control how text is truncated when it doesn't fit in the layer bounds.
///
/// # Examples
///
/// ```ignore
/// CATextLayerBuilder::new()
///     .text("This is a very long text that might be truncated...")
///     .truncation(Truncation::End)
///     .build();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Truncation {
    /// No truncation. Text may overflow the layer bounds.
    #[default]
    None,
    /// Truncate at the start of the text (e.g., "...long text").
    Start,
    /// Truncate at the end of the text (e.g., "Long text...").
    End,
    /// Truncate in the middle of the text (e.g., "Long...text").
    Middle,
}

impl Truncation {
    /// Returns the Core Animation truncation mode string for this truncation mode.
    fn to_ca_truncation(&self) -> &'static NSString {
        // SAFETY: These extern statics are always valid on macOS.
        unsafe {
            match self {
                Truncation::None => kCATruncationNone,
                Truncation::Start => kCATruncationStart,
                Truncation::End => kCATruncationEnd,
                Truncation::Middle => kCATruncationMiddle,
            }
        }
    }
}

/// Builder for `CATextLayer`.
///
/// `CATextLayer` renders text using Core Text. This builder provides an ergonomic
/// API for configuring text layers with fonts, colors, alignment, and animations.
///
/// # Basic Usage
///
/// ```ignore
/// let text = CATextLayerBuilder::new()
///     .text("Hello, World!")
///     .font_size(24.0)
///     .foreground_color(Color::WHITE)
///     .alignment(TextAlign::Center)
///     .build();
/// ```
///
/// # With Font Name
///
/// ```ignore
/// let text = CATextLayerBuilder::new()
///     .text("Monospaced")
///     .font_name("Menlo")
///     .font_size(16.0)
///     .foreground_color(Color::CYAN)
///     .build();
/// ```
///
/// # With CTFont
///
/// For more control over font attributes, you can provide a `CTFont` directly:
///
/// ```ignore
/// let font = unsafe {
///     CTFont::with_name(&CFString::from_static_str("Helvetica-Bold"), 18.0, std::ptr::null())
/// };
///
/// let text = CATextLayerBuilder::new()
///     .text("Bold Text")
///     .font(font)
///     .foreground_color(Color::ORANGE)
///     .build();
/// ```
///
/// # With Animations
///
/// Animations can be added inline using the `.animate()` method:
///
/// ```ignore
/// let text = CATextLayerBuilder::new()
///     .text("Pulsing Text")
///     .font_size(32.0)
///     .foreground_color(Color::RED)
///     .animate("pulse", KeyPath::TransformScale, |a| {
///         a.values(0.9, 1.1)
///             .duration(500.millis())
///             .autoreverses()
///             .repeat(Repeat::Forever)
///     })
///     .build();
/// ```
///
/// # Text Wrapping and Truncation
///
/// ```ignore
/// let text = CATextLayerBuilder::new()
///     .text("This is a long text that will wrap to multiple lines")
///     .bounds(CGRect::new(CGPoint::ZERO, CGSize::new(200.0, 100.0)))
///     .wrapped(true)
///     .truncation(Truncation::End)
///     .build();
/// ```
#[derive(Default)]
pub struct CATextLayerBuilder {
    // Text content
    text: Option<String>,

    // Font properties
    font: Option<CFRetained<CTFont>>,
    font_name: Option<String>,
    font_size: Option<CGFloat>,

    // Appearance
    foreground_color: Option<CFRetained<CGColor>>,
    alignment: Option<TextAlign>,
    truncation: Option<Truncation>,
    wrapped: Option<bool>,

    // Layer geometry
    bounds: Option<CGRect>,
    position: Option<CGPoint>,
    transform: Option<CATransform3D>,

    // Layer properties
    hidden: Option<bool>,
    opacity: Option<f32>,

    // Shadow properties
    shadow_color: Option<CFRetained<CGColor>>,
    shadow_offset: Option<(f64, f64)>,
    shadow_radius: Option<f64>,
    shadow_opacity: Option<f32>,

    // Simple transform shortcuts
    scale: Option<f64>,
    rotation: Option<f64>,
    translation: Option<(f64, f64)>,

    // Animations
    animations: Vec<PendingAnimation>,
}

impl CATextLayerBuilder {
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    // ========================================================================
    // Text content
    // ========================================================================

    /// Sets the text content to display.
    ///
    /// # Arguments
    ///
    /// * `text` - The string to render
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CATextLayerBuilder::new()
    ///     .text("Hello, World!")
    ///     .build();
    /// ```
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    // ========================================================================
    // Font properties
    // ========================================================================

    /// Sets the font using a `CTFont` object.
    ///
    /// This gives you full control over the font, including traits like bold,
    /// italic, etc.
    ///
    /// # Arguments
    ///
    /// * `font` - A Core Text font object
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let font = unsafe {
    ///     CTFont::with_name(
    ///         &CFString::from_static_str("Helvetica-Bold"),
    ///         18.0,
    ///         std::ptr::null()
    ///     )
    /// };
    ///
    /// CATextLayerBuilder::new()
    ///     .text("Bold Text")
    ///     .font(font)
    ///     .build();
    /// ```
    ///
    /// # Notes
    ///
    /// When `.font()` is set, it takes precedence over `.font_name()`.
    /// The font size from the `CTFont` will be used unless `.font_size()` is
    /// also called.
    pub fn font(mut self, font: CFRetained<CTFont>) -> Self {
        self.font = Some(font);
        self
    }

    /// Sets the font by name (PostScript name preferred).
    ///
    /// Common font names include:
    /// - "Helvetica", "Helvetica-Bold", "Helvetica-Oblique"
    /// - "Menlo", "Menlo-Bold" (monospaced)
    /// - "SF Pro", "SF Pro Display" (system fonts on modern macOS)
    /// - "Times New Roman"
    ///
    /// # Arguments
    ///
    /// * `name` - The font name (PostScript name preferred)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CATextLayerBuilder::new()
    ///     .text("Monospaced")
    ///     .font_name("Menlo")
    ///     .font_size(14.0)
    ///     .build();
    /// ```
    ///
    /// # Notes
    ///
    /// If `.font()` is also set, it takes precedence over `.font_name()`.
    /// Use `.font_size()` to set the size when using `.font_name()`.
    pub fn font_name(mut self, name: impl Into<String>) -> Self {
        self.font_name = Some(name.into());
        self
    }

    /// Sets the font size in points.
    ///
    /// # Arguments
    ///
    /// * `size` - Font size in points (e.g., 12.0, 16.0, 24.0)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CATextLayerBuilder::new()
    ///     .text("Large Text")
    ///     .font_size(48.0)
    ///     .build();
    /// ```
    ///
    /// # Notes
    ///
    /// If `.font()` is set, the font size from the `CTFont` is used unless
    /// `.font_size()` is explicitly called to override it.
    pub fn font_size(mut self, size: CGFloat) -> Self {
        self.font_size = Some(size);
        self
    }

    // ========================================================================
    // Appearance
    // ========================================================================

    /// Sets the text foreground color.
    ///
    /// Accepts any type that implements `Into<CFRetained<CGColor>>`, including:
    /// - `Color::RED`, `Color::rgb(1.0, 0.0, 0.0)`
    /// - `CFRetained<CGColor>` directly
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CATextLayerBuilder::new()
    ///     .text("Red Text")
    ///     .foreground_color(Color::RED)
    ///     .build();
    /// ```
    pub fn foreground_color(mut self, color: impl Into<CFRetained<CGColor>>) -> Self {
        self.foreground_color = Some(color.into());
        self
    }

    /// Sets the text foreground color from RGBA values (0.0-1.0).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CATextLayerBuilder::new()
    ///     .text("Orange Text")
    ///     .foreground_rgba(1.0, 0.5, 0.0, 1.0)
    ///     .build();
    /// ```
    pub fn foreground_rgba(mut self, r: CGFloat, g: CGFloat, b: CGFloat, a: CGFloat) -> Self {
        self.foreground_color = Some(Color::rgba(r, g, b, a).into());
        self
    }

    /// Sets the text alignment.
    ///
    /// # Arguments
    ///
    /// * `alignment` - The text alignment mode
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CATextLayerBuilder::new()
    ///     .text("Centered")
    ///     .alignment(TextAlign::Center)
    ///     .build();
    /// ```
    pub fn alignment(mut self, alignment: TextAlign) -> Self {
        self.alignment = Some(alignment);
        self
    }

    /// Sets the truncation mode for text that doesn't fit.
    ///
    /// # Arguments
    ///
    /// * `truncation` - The truncation mode
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CATextLayerBuilder::new()
    ///     .text("This text is too long to fit...")
    ///     .truncation(Truncation::End)
    ///     .build();
    /// ```
    pub fn truncation(mut self, truncation: Truncation) -> Self {
        self.truncation = Some(truncation);
        self
    }

    /// Sets whether text should wrap to multiple lines.
    ///
    /// When `true`, text wraps at word boundaries when it exceeds the layer width.
    /// When `false` (default), text remains on a single line.
    ///
    /// # Arguments
    ///
    /// * `wrapped` - Whether to enable text wrapping
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CATextLayerBuilder::new()
    ///     .text("This is a long text that will wrap")
    ///     .bounds(CGRect::new(CGPoint::ZERO, CGSize::new(100.0, 200.0)))
    ///     .wrapped(true)
    ///     .build();
    /// ```
    pub fn wrapped(mut self, wrapped: bool) -> Self {
        self.wrapped = Some(wrapped);
        self
    }

    // ========================================================================
    // Layer geometry
    // ========================================================================

    /// Sets the bounds rectangle.
    ///
    /// The bounds define the layer's size and the coordinate space for sublayers.
    /// For text layers, bounds control the area where text is rendered.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CATextLayerBuilder::new()
    ///     .text("Text in a box")
    ///     .bounds(CGRect::new(CGPoint::ZERO, CGSize::new(200.0, 50.0)))
    ///     .build();
    /// ```
    pub fn bounds(mut self, bounds: CGRect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the bounds from width and height (origin at ZERO).
    ///
    /// Convenience method equivalent to:
    /// ```ignore
    /// .bounds(CGRect::new(CGPoint::ZERO, CGSize::new(width, height)))
    /// ```
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CATextLayerBuilder::new()
    ///     .text("Sized text area")
    ///     .size(200.0, 50.0)
    ///     .build();
    /// ```
    pub fn size(mut self, width: CGFloat, height: CGFloat) -> Self {
        self.bounds = Some(CGRect::new(CGPoint::ZERO, CGSize::new(width, height)));
        self
    }

    /// Sets the position in superlayer coordinates.
    ///
    /// The position is where the layer's anchor point is placed in the superlayer.
    /// By default, the anchor point is at the center of the layer (0.5, 0.5).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CATextLayerBuilder::new()
    ///     .text("Positioned text")
    ///     .position(CGPoint::new(100.0, 100.0))
    ///     .build();
    /// ```
    pub fn position(mut self, position: CGPoint) -> Self {
        self.position = Some(position);
        self
    }

    /// Sets the 3D transform.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let rotate = CATransform3D::new_rotation(0.1, 0.0, 0.0, 1.0);
    /// CATextLayerBuilder::new()
    ///     .text("Rotated")
    ///     .transform(rotate)
    ///     .build();
    /// ```
    pub fn transform(mut self, transform: CATransform3D) -> Self {
        self.transform = Some(transform);
        self
    }

    // ========================================================================
    // Layer properties
    // ========================================================================

    /// Sets whether the layer is hidden.
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = Some(hidden);
        self
    }

    /// Sets the opacity (0.0-1.0).
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }

    // ========================================================================
    // Shadow properties
    // ========================================================================

    /// Sets the shadow color.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CATextLayerBuilder::new()
    ///     .text("Shadowed")
    ///     .shadow_color(Color::BLACK)
    ///     .shadow_radius(5.0)
    ///     .shadow_opacity(0.5)
    ///     .build();
    /// ```
    pub fn shadow_color(mut self, color: impl Into<CFRetained<CGColor>>) -> Self {
        self.shadow_color = Some(color.into());
        self
    }

    /// Sets the shadow offset (dx, dy).
    ///
    /// Positive `dx` moves the shadow right, positive `dy` moves it down.
    pub fn shadow_offset(mut self, dx: f64, dy: f64) -> Self {
        self.shadow_offset = Some((dx, dy));
        self
    }

    /// Sets the shadow blur radius.
    ///
    /// Larger values create a softer, more diffuse shadow.
    pub fn shadow_radius(mut self, radius: f64) -> Self {
        self.shadow_radius = Some(radius);
        self
    }

    /// Sets the shadow opacity (0.0 to 1.0).
    pub fn shadow_opacity(mut self, opacity: f32) -> Self {
        self.shadow_opacity = Some(opacity);
        self
    }

    // ========================================================================
    // Simple transform shortcuts
    // ========================================================================

    /// Sets a uniform scale transform.
    ///
    /// # Notes
    ///
    /// When multiple transform shortcuts are set, they are composed in order:
    /// scale -> rotation -> translation.
    ///
    /// If you also call `.transform()`, the explicit transform takes
    /// precedence and `scale`/`rotation`/`translate` are ignored.
    pub fn scale(mut self, scale: f64) -> Self {
        self.scale = Some(scale);
        self
    }

    /// Sets a z-axis rotation transform (in radians).
    ///
    /// For degrees, use: `.rotation(45.0_f64.to_radians())`
    ///
    /// # Notes
    ///
    /// When multiple transform shortcuts are set, they are composed in order:
    /// scale -> rotation -> translation.
    pub fn rotation(mut self, radians: f64) -> Self {
        self.rotation = Some(radians);
        self
    }

    /// Sets a translation transform (dx, dy).
    ///
    /// # Notes
    ///
    /// When multiple transform shortcuts are set, they are composed in order:
    /// scale -> rotation -> translation.
    pub fn translate(mut self, dx: f64, dy: f64) -> Self {
        self.translation = Some((dx, dy));
        self
    }

    // ========================================================================
    // Animations
    // ========================================================================

    /// Adds an animation to be applied when the layer is built.
    ///
    /// The animation is configured using a closure that receives a
    /// [`CABasicAnimationBuilder`] and returns the configured builder.
    ///
    /// # Arguments
    ///
    /// * `name` - A unique identifier for this animation (used as the animation key)
    /// * `key_path` - The property to animate (e.g., [`KeyPath::TransformScale`])
    /// * `configure` - A closure that configures the animation builder
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Pulsing text
    /// CATextLayerBuilder::new()
    ///     .text("Pulsing")
    ///     .font_size(24.0)
    ///     .foreground_color(Color::CYAN)
    ///     .animate("pulse", KeyPath::TransformScale, |a| {
    ///         a.values(0.9, 1.1)
    ///             .duration(500.millis())
    ///             .autoreverses()
    ///             .repeat(Repeat::Forever)
    ///     })
    ///     .build();
    ///
    /// // Fading text
    /// CATextLayerBuilder::new()
    ///     .text("Fading")
    ///     .foreground_color(Color::WHITE)
    ///     .animate("fade", KeyPath::Opacity, |a| {
    ///         a.values(1.0, 0.3)
    ///             .duration(1.seconds())
    ///             .autoreverses()
    ///             .repeat(Repeat::Forever)
    ///     })
    ///     .build();
    /// ```
    pub fn animate<F>(mut self, name: impl Into<String>, key_path: KeyPath, configure: F) -> Self
    where
        F: FnOnce(CABasicAnimationBuilder) -> CABasicAnimationBuilder,
    {
        let builder = CABasicAnimationBuilder::new(key_path);
        let animation = configure(builder).build();
        self.animations.push(PendingAnimation {
            name: name.into(),
            animation,
        });
        self
    }

    // ========================================================================
    // Build
    // ========================================================================

    /// Builds and returns the configured `CATextLayer`.
    ///
    /// All pending animations added via `.animate()` are applied to the layer.
    pub fn build(self) -> Retained<CATextLayer> {
        let layer = CATextLayer::new();

        // Set text content
        if let Some(ref text) = self.text {
            let ns_string = NSString::from_str(text);
            // SAFETY: NSString is a valid object type for the string property
            unsafe {
                layer.setString(Some(&ns_string));
            }
        }

        // Set font - CTFont takes precedence over font_name
        if let Some(ref font) = self.font {
            // SAFETY: CTFont is toll-free bridged with NSFont and is valid for setFont
            unsafe {
                layer.setFont(Some(&**font));
            }
        } else if let Some(ref font_name) = self.font_name {
            // Create CTFont from name and set it
            let cf_name = CFString::from_str(font_name);
            let size = self.font_size.unwrap_or(12.0);
            // SAFETY: null matrix is valid and means identity transform
            let font = unsafe { CTFont::with_name(&cf_name, size, std::ptr::null()) };
            // SAFETY: CTFont is valid for setFont
            unsafe {
                layer.setFont(Some(&*font));
            }
        }

        // Set font size (overrides font's size if both are set)
        if let Some(size) = self.font_size {
            layer.setFontSize(size);
        }

        // Set foreground color
        if let Some(ref color) = self.foreground_color {
            layer.setForegroundColor(Some(&**color));
        }

        // Set alignment
        if let Some(alignment) = self.alignment {
            layer.setAlignmentMode(alignment.to_ca_alignment());
        }

        // Set truncation mode
        if let Some(truncation) = self.truncation {
            layer.setTruncationMode(truncation.to_ca_truncation());
        }

        // Set wrapping
        if let Some(wrapped) = self.wrapped {
            layer.setWrapped(wrapped);
        }

        // Set geometry
        if let Some(bounds) = self.bounds {
            layer.setBounds(bounds);
        }
        if let Some(position) = self.position {
            layer.setPosition(position);
        }

        // Transform handling: explicit transform takes precedence over shortcuts
        if let Some(transform) = self.transform {
            layer.setTransform(transform);
        } else if self.scale.is_some() || self.rotation.is_some() || self.translation.is_some() {
            // Compose transforms in order: scale -> rotation -> translation
            let mut transform = CATransform3D::new_scale(1.0, 1.0, 1.0); // identity

            if let Some(s) = self.scale {
                transform = CATransform3D::new_scale(s, s, 1.0);
            }

            if let Some(r) = self.rotation {
                let rotation_transform = CATransform3D::new_rotation(r, 0.0, 0.0, 1.0);
                transform = transform.concat(rotation_transform);
            }

            if let Some((dx, dy)) = self.translation {
                let translation_transform = CATransform3D::new_translation(dx, dy, 0.0);
                transform = transform.concat(translation_transform);
            }

            layer.setTransform(transform);
        }

        // Set layer properties
        if let Some(hidden) = self.hidden {
            layer.setHidden(hidden);
        }
        if let Some(opacity) = self.opacity {
            layer.setOpacity(opacity);
        }

        // Apply shadow properties
        if let Some(ref color) = self.shadow_color {
            layer.setShadowColor(Some(&**color));
        }
        if let Some((dx, dy)) = self.shadow_offset {
            layer.setShadowOffset(CGSize::new(dx, dy));
        }
        if let Some(radius) = self.shadow_radius {
            layer.setShadowRadius(radius);
        }
        if let Some(opacity) = self.shadow_opacity {
            layer.setShadowOpacity(opacity);
        }

        // Apply all pending animations
        for pending in self.animations {
            let key = NSString::from_str(&pending.name);
            layer.addAnimation_forKey(&pending.animation, Some(&key));
        }

        layer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_align_default() {
        assert_eq!(TextAlign::default(), TextAlign::Natural);
    }

    #[test]
    fn test_truncation_default() {
        assert_eq!(Truncation::default(), Truncation::None);
    }

    #[test]
    fn test_builder_default() {
        let builder = CATextLayerBuilder::new();
        assert!(builder.text.is_none());
        assert!(builder.font.is_none());
        assert!(builder.font_name.is_none());
        assert!(builder.font_size.is_none());
        assert!(builder.foreground_color.is_none());
        assert!(builder.alignment.is_none());
        assert!(builder.truncation.is_none());
        assert!(builder.wrapped.is_none());
        assert!(builder.bounds.is_none());
        assert!(builder.position.is_none());
        assert!(builder.opacity.is_none());
        assert!(builder.animations.is_empty());
    }

    #[test]
    fn test_builder_chaining() {
        let builder = CATextLayerBuilder::new()
            .text("Hello")
            .font_name("Helvetica")
            .font_size(24.0)
            .alignment(TextAlign::Center)
            .truncation(Truncation::End)
            .wrapped(true)
            .opacity(0.8);

        assert_eq!(builder.text.as_deref(), Some("Hello"));
        assert_eq!(builder.font_name.as_deref(), Some("Helvetica"));
        assert_eq!(builder.font_size, Some(24.0));
        assert_eq!(builder.alignment, Some(TextAlign::Center));
        assert_eq!(builder.truncation, Some(Truncation::End));
        assert_eq!(builder.wrapped, Some(true));
        assert_eq!(builder.opacity, Some(0.8));
    }

    #[test]
    fn test_size_convenience() {
        let builder = CATextLayerBuilder::new().size(200.0, 50.0);

        let bounds = builder.bounds.unwrap();
        assert!((bounds.size.width - 200.0).abs() < f64::EPSILON);
        assert!((bounds.size.height - 50.0).abs() < f64::EPSILON);
    }
}
