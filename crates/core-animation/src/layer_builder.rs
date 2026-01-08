//! Builder for `CALayer` (the basic compositing layer).

use crate::color::Color;
use objc2::rc::Retained;
use objc2_core_foundation::{CFRetained, CGFloat, CGPoint, CGRect};
use objc2_core_graphics::CGColor;
use objc2_quartz_core::{CALayer, CATransform3D};

/// Builder for `CALayer`.
///
/// ```ignore
/// let layer = CALayerBuilder::new()
///     .bounds(CGRect::new(CGPoint::ZERO, CGSize::new(100.0, 100.0)))
///     .position(CGPoint::new(50.0, 50.0))
///     .background_color(Color::DARK_GRAY)
///     .corner_radius(8.0)
///     .build();
/// ```
#[derive(Default)]
pub struct CALayerBuilder {
    bounds: Option<CGRect>,
    position: Option<CGPoint>,
    background_color: Option<CFRetained<CGColor>>,
    corner_radius: Option<CGFloat>,
    hidden: Option<bool>,
    transform: Option<CATransform3D>,
    opacity: Option<f32>,
}

impl CALayerBuilder {
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

    /// Sets the background color.
    ///
    /// Accepts any type that implements `Into<CFRetained<CGColor>>`, including:
    /// - `Color::RED`, `Color::rgb(1.0, 0.0, 0.0)`
    /// - `CFRetained<CGColor>` directly
    ///
    /// # Example
    ///
    /// ```ignore
    /// .background_color(Color::BLUE)
    /// .background_color(Color::rgb(0.2, 0.2, 0.2))
    /// .background_color(Color::WHITE.with_alpha(0.5))
    /// ```
    pub fn background_color(mut self, color: impl Into<CFRetained<CGColor>>) -> Self {
        self.background_color = Some(color.into());
        self
    }

    /// Sets the background color from RGBA values (0.0–1.0).
    pub fn background_rgba(mut self, r: CGFloat, g: CGFloat, b: CGFloat, a: CGFloat) -> Self {
        self.background_color = Some(Color::rgba(r, g, b, a).into());
        self
    }

    /// Sets the corner radius.
    pub fn corner_radius(mut self, radius: CGFloat) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Sets whether the layer is hidden.
    pub fn hidden(mut self, hidden: bool) -> Self {
        self.hidden = Some(hidden);
        self
    }

    /// Sets the 3D transform.
    pub fn transform(mut self, transform: CATransform3D) -> Self {
        self.transform = Some(transform);
        self
    }

    /// Sets the opacity (0.0–1.0).
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.opacity = Some(opacity);
        self
    }

    /// Builds and returns the configured `CALayer`.
    pub fn build(self) -> Retained<CALayer> {
        let layer = CALayer::new();

        if let Some(bounds) = self.bounds {
            layer.setBounds(bounds);
        }
        if let Some(position) = self.position {
            layer.setPosition(position);
        }
        if let Some(ref color) = self.background_color {
            layer.setBackgroundColor(Some(&**color));
        }
        if let Some(radius) = self.corner_radius {
            layer.setCornerRadius(radius);
        }
        if let Some(hidden) = self.hidden {
            layer.setHidden(hidden);
        }
        if let Some(transform) = self.transform {
            layer.setTransform(transform);
        }
        if let Some(opacity) = self.opacity {
            layer.setOpacity(opacity);
        }

        layer
    }
}
