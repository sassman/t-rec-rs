//! Builder for `CAShapeLayer` (vector shape rendering).

use crate::animation_builder::{CABasicAnimationBuilder, KeyPath};
use crate::color::Color;
use objc2::rc::Retained;
use objc2_core_foundation::{CFRetained, CGFloat, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{CGColor, CGPath};
use objc2_foundation::NSString;
use objc2_quartz_core::{CABasicAnimation, CAShapeLayer, CATransform3D};

/// A pending animation to be applied when the layer is built.
struct PendingAnimation {
    name: String,
    animation: Retained<CABasicAnimation>,
}

/// Builder for `CAShapeLayer`.
///
/// # Basic Usage
///
/// ```ignore
/// let shape = CAShapeLayerBuilder::new()
///     .path(circle_path)
///     .fill_color(Color::RED)
///     .stroke_color(Color::WHITE)
///     .line_width(2.0)
///     .build();
/// ```
///
/// # With Animations
///
/// Animations can be added inline using the `.animate()` method:
///
/// ```ignore
/// let shape = CAShapeLayerBuilder::new()
///     .path(circle_path)
///     .fill_color(Color::RED)
///     .animate("pulse", KeyPath::TransformScale, |a| {
///         a.values(0.85, 1.15)
///             .duration(800.millis())
///             .easing(Easing::InOut)
///             .autoreverses()
///             .repeat(Repeat::Forever)
///     })
///     .build();
/// ```
///
/// Multiple animations can be added:
///
/// ```ignore
/// CAShapeLayerBuilder::new()
///     .fill_color(Color::RED)
///     .animate("pulse", KeyPath::TransformScale, |a| {
///         a.values(0.9, 1.1).duration(500.millis()).repeat(Repeat::Forever)
///     })
///     .animate("fade", KeyPath::Opacity, |a| {
///         a.values(1.0, 0.7).duration(1.seconds()).repeat(Repeat::Forever)
///     })
///     .build()
/// ```
#[derive(Default)]
pub struct CAShapeLayerBuilder {
    bounds: Option<CGRect>,
    position: Option<CGPoint>,
    path: Option<CFRetained<CGPath>>,
    fill_color: Option<CFRetained<CGColor>>,
    stroke_color: Option<CFRetained<CGColor>>,
    line_width: Option<CGFloat>,
    transform: Option<CATransform3D>,
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
    animations: Vec<PendingAnimation>,
}

impl CAShapeLayerBuilder {
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the bounds rectangle.
    pub fn bounds(mut self, bounds: CGRect) -> Self {
        self.bounds = Some(bounds);
        self
    }

    /// Sets the position in superlayer coordinates.
    pub fn position(mut self, position: CGPoint) -> Self {
        self.position = Some(position);
        self
    }

    /// Sets the path to render.
    pub fn path(mut self, path: CFRetained<CGPath>) -> Self {
        self.path = Some(path);
        self
    }

    /// Sets the fill color.
    ///
    /// Accepts any type that implements `Into<CFRetained<CGColor>>`, including:
    /// - `Color::RED`, `Color::rgb(1.0, 0.0, 0.0)`
    /// - `CFRetained<CGColor>` directly
    pub fn fill_color(mut self, color: impl Into<CFRetained<CGColor>>) -> Self {
        self.fill_color = Some(color.into());
        self
    }

    /// Sets the fill color from RGBA values (0.0–1.0).
    pub fn fill_rgba(mut self, r: CGFloat, g: CGFloat, b: CGFloat, a: CGFloat) -> Self {
        self.fill_color = Some(Color::rgba(r, g, b, a).into());
        self
    }

    /// Sets the stroke color.
    ///
    /// Accepts any type that implements `Into<CFRetained<CGColor>>`, including:
    /// - `Color::WHITE`, `Color::rgb(1.0, 1.0, 1.0)`
    /// - `CFRetained<CGColor>` directly
    pub fn stroke_color(mut self, color: impl Into<CFRetained<CGColor>>) -> Self {
        self.stroke_color = Some(color.into());
        self
    }

    /// Sets the stroke color from RGBA values (0.0–1.0).
    pub fn stroke_rgba(mut self, r: CGFloat, g: CGFloat, b: CGFloat, a: CGFloat) -> Self {
        self.stroke_color = Some(Color::rgba(r, g, b, a).into());
        self
    }

    /// Sets the stroke line width.
    pub fn line_width(mut self, width: CGFloat) -> Self {
        self.line_width = Some(width);
        self
    }

    /// Sets the 3D transform.
    pub fn transform(mut self, transform: CATransform3D) -> Self {
        self.transform = Some(transform);
        self
    }

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
    /// CAShapeLayerBuilder::new()
    ///     .shadow_color(Color::BLACK)
    ///     .shadow_radius(10.0)
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
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CAShapeLayerBuilder::new()
    ///     .shadow_offset(0.0, 4.0)  // Shadow below
    ///     .shadow_radius(8.0)
    ///     .shadow_opacity(0.3)
    ///     .build();
    /// ```
    pub fn shadow_offset(mut self, dx: f64, dy: f64) -> Self {
        self.shadow_offset = Some((dx, dy));
        self
    }

    /// Sets the shadow blur radius.
    ///
    /// Larger values create a softer, more diffuse shadow.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CAShapeLayerBuilder::new()
    ///     .shadow_radius(15.0)  // Soft glow effect
    ///     .shadow_opacity(0.7)
    ///     .build();
    /// ```
    pub fn shadow_radius(mut self, radius: f64) -> Self {
        self.shadow_radius = Some(radius);
        self
    }

    /// Sets the shadow opacity (0.0 to 1.0).
    ///
    /// # Examples
    ///
    /// ```ignore
    /// CAShapeLayerBuilder::new()
    ///     .shadow_color(Color::CYAN)
    ///     .shadow_radius(10.0)
    ///     .shadow_opacity(0.8)  // Bright glow
    ///     .build();
    /// ```
    pub fn shadow_opacity(mut self, opacity: f32) -> Self {
        self.shadow_opacity = Some(opacity);
        self
    }

    // ========================================================================
    // Simple transform shortcuts
    // ========================================================================

    /// Sets a uniform scale transform.
    ///
    /// This is applied using `CATransform3D` internally.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Scale to 80% of original size
    /// CAShapeLayerBuilder::new()
    ///     .scale(0.8)
    ///     .build();
    /// ```
    ///
    /// # Notes
    ///
    /// When multiple transform shortcuts are set, they are composed in order:
    /// scale → rotation → translation.
    ///
    /// If you also call `.transform()`, the explicit transform takes
    /// precedence and `scale`/`rotation`/`translate` are ignored.
    pub fn scale(mut self, scale: f64) -> Self {
        self.scale = Some(scale);
        self
    }

    /// Sets a z-axis rotation transform (in radians).
    ///
    /// This is applied using `CATransform3D` internally.
    /// For degrees, use: `.rotation(45.0_f64.to_radians())`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::f64::consts::PI;
    ///
    /// // Rotate 45 degrees
    /// CAShapeLayerBuilder::new()
    ///     .rotation(PI / 4.0)
    ///     .build();
    ///
    /// // Or using to_radians()
    /// CAShapeLayerBuilder::new()
    ///     .rotation(45.0_f64.to_radians())
    ///     .build();
    /// ```
    ///
    /// # Notes
    ///
    /// When multiple transform shortcuts are set, they are composed in order:
    /// scale → rotation → translation.
    ///
    /// If you also call `.transform()`, the explicit transform takes
    /// precedence and `scale`/`rotation`/`translate` are ignored.
    pub fn rotation(mut self, radians: f64) -> Self {
        self.rotation = Some(radians);
        self
    }

    /// Sets a translation transform (dx, dy).
    ///
    /// This is applied using `CATransform3D` internally.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Move 10 points right and 20 points up
    /// CAShapeLayerBuilder::new()
    ///     .translate(10.0, 20.0)
    ///     .build();
    /// ```
    ///
    /// # Notes
    ///
    /// When multiple transform shortcuts are set, they are composed in order:
    /// scale → rotation → translation.
    ///
    /// If you also call `.transform()`, the explicit transform takes
    /// precedence and `scale`/`rotation`/`translate` are ignored.
    pub fn translate(mut self, dx: f64, dy: f64) -> Self {
        self.translation = Some((dx, dy));
        self
    }

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
    /// // Simple pulse animation
    /// CAShapeLayerBuilder::new()
    ///     .path(circle_path)
    ///     .fill_color(Color::RED)
    ///     .animate("pulse", KeyPath::TransformScale, |a| {
    ///         a.values(0.85, 1.15)
    ///             .duration(800.millis())
    ///             .easing(Easing::InOut)
    ///             .autoreverses()
    ///             .repeat(Repeat::Forever)
    ///     })
    ///     .build();
    ///
    /// // Multiple animations on the same layer
    /// CAShapeLayerBuilder::new()
    ///     .fill_color(Color::BLUE)
    ///     .animate("scale", KeyPath::TransformScale, |a| {
    ///         a.values(0.9, 1.1).duration(500.millis()).repeat(Repeat::Forever)
    ///     })
    ///     .animate("fade", KeyPath::Opacity, |a| {
    ///         a.values(1.0, 0.5).duration(1.seconds()).repeat(Repeat::Forever)
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

    /// Builds and returns the configured `CAShapeLayer`.
    ///
    /// All pending animations added via `.animate()` are applied to the layer.
    pub fn build(self) -> Retained<CAShapeLayer> {
        let layer = CAShapeLayer::new();

        if let Some(bounds) = self.bounds {
            layer.setBounds(bounds);
        }
        if let Some(position) = self.position {
            layer.setPosition(position);
        }
        if let Some(ref path) = self.path {
            layer.setPath(Some(&**path));
        }
        if let Some(ref color) = self.fill_color {
            layer.setFillColor(Some(&**color));
        }
        if let Some(ref color) = self.stroke_color {
            layer.setStrokeColor(Some(&**color));
        }
        if let Some(width) = self.line_width {
            layer.setLineWidth(width);
        }

        // Transform handling: explicit transform takes precedence over shortcuts
        if let Some(transform) = self.transform {
            layer.setTransform(transform);
        } else if self.scale.is_some() || self.rotation.is_some() || self.translation.is_some() {
            // Compose transforms in order: scale → rotation → translation
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
