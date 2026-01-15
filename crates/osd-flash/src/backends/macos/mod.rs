//! macOS backend using Core Animation for GPU-accelerated rendering.
//!
//! This backend is a thin glue layer that translates osd-flash's platform-agnostic
//! configuration to Core Animation calls via the `core-animation` crate.
//!
//! # Architecture
//!
//! - `OsdConfig` -> `WindowBuilder` configuration
//! - `LayerConfig` -> `CAShapeLayerBuilder` configuration
//! - `Animation` -> `CABasicAnimationBuilder` configuration
//!
//! All animations run on the GPU compositor thread, not the main thread.

use std::time::Duration;

use core_animation::prelude::*;
use objc2::MainThreadMarker;
use objc2_app_kit::NSScreen;

use crate::builder::OsdConfig;
use crate::color::Color;
use crate::composition::animation::{Animation, Easing as OsdEasing, Repeat as OsdRepeat};
use crate::composition::layer_builder::{LayerConfig, LayerPosition, ShapeKind, TextAlign};
use crate::error::{Error, Result};
use crate::level::WindowLevel as OsdWindowLevel;
use crate::position::Position;

/// macOS OSD window backed by Core Animation.
///
/// This is a thin wrapper around `core_animation::Window` that handles
/// the translation from osd-flash's platform-agnostic types.
pub struct MacOsWindow {
    window: Window,
}

impl MacOsWindow {
    /// Create a new window from the given configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not called from the main thread ([`Error::NotOnMainThread`])
    /// - No screen is available for positioning ([`Error::NoScreenAvailable`])
    pub fn from_config(config: OsdConfig) -> Result<Self> {
        let (width, height) = (config.size.width, config.size.height);

        let mut builder = WindowBuilder::new()
            .size(width, height)
            .borderless()
            .transparent()
            .non_activating() // OSD windows should never steal focus
            .level(config.level.into());

        // Apply position
        builder = apply_position(builder, &config)?;

        // Apply background styling
        if let Some(bg) = config.background {
            builder = builder.background_color(bg);
        }

        if config.corner_radius > 0.0 {
            builder = builder.corner_radius(config.corner_radius);
        }

        // Convert each layer to CAShapeLayer
        for layer_config in &config.layers {
            builder = add_layer(builder, layer_config, width, height);
        }

        let window = builder.build();

        Ok(Self { window })
    }

    /// Show the window for the specified duration.
    ///
    /// Animations defined on layers will run automatically on the GPU.
    pub fn show_for(&self, duration: Duration) {
        self.window.show_for(duration);
    }

    /// Show the window.
    pub fn show(&self) {
        self.window.show();
    }

    /// Run a single iteration of the event loop.
    pub fn run_loop_tick(&self) {
        self.window.run_loop_tick();
    }

    /// Get the underlying window for advanced usage.
    pub fn inner(&self) -> &Window {
        &self.window
    }

    /// Get the window ID (useful for screen recording).
    pub fn window_id(&self) -> u64 {
        self.window.window_id()
    }
}

/// Convert osd-flash WindowLevel to core-animation WindowLevel.
impl From<OsdWindowLevel> for WindowLevel {
    fn from(level: OsdWindowLevel) -> Self {
        match level {
            OsdWindowLevel::Normal => WindowLevel::Normal,
            OsdWindowLevel::Floating => WindowLevel::Floating,
            OsdWindowLevel::ModalPanel => WindowLevel::ModalPanel,
            OsdWindowLevel::ScreenSaver => WindowLevel::ScreenSaver,
            OsdWindowLevel::AboveAll => WindowLevel::AboveAll,
            OsdWindowLevel::Custom(n) => WindowLevel::Custom(n),
        }
    }
}

/// Convert osd-flash Color to core-animation Color.
impl From<Color> for core_animation::Color {
    fn from(color: Color) -> Self {
        core_animation::Color::rgba(color.r, color.g, color.b, color.a)
    }
}

/// Convert osd-flash Easing to core-animation Easing.
impl From<OsdEasing> for Easing {
    fn from(easing: OsdEasing) -> Self {
        match easing {
            OsdEasing::Linear => Easing::Linear,
            OsdEasing::In => Easing::In,
            OsdEasing::Out => Easing::Out,
            OsdEasing::InOut => Easing::InOut,
        }
    }
}

/// Convert osd-flash Repeat to core-animation Repeat.
impl From<OsdRepeat> for Repeat {
    fn from(repeat: OsdRepeat) -> Self {
        match repeat {
            OsdRepeat::Once => Repeat::Once,
            OsdRepeat::Times(n) => Repeat::Times(n),
            OsdRepeat::Forever => Repeat::Forever,
        }
    }
}

/// Convert osd-flash TextAlign to core-animation TextAlign.
impl From<TextAlign> for core_animation::TextAlign {
    fn from(align: TextAlign) -> Self {
        match align {
            TextAlign::Left => core_animation::TextAlign::Left,
            TextAlign::Center => core_animation::TextAlign::Center,
            TextAlign::Right => core_animation::TextAlign::Right,
        }
    }
}

/// Apply position configuration to the window builder.
fn apply_position(builder: WindowBuilder, config: &OsdConfig) -> Result<WindowBuilder> {
    let mtm = MainThreadMarker::new().ok_or(Error::NotOnMainThread)?;
    let screen = NSScreen::mainScreen(mtm).ok_or(Error::NoScreenAvailable)?;
    let screen_frame = screen.frame();

    let (width, height) = (config.size.width, config.size.height);
    let margin = &config.margin;

    let positioned = match config.position {
        Position::TopRight => {
            let x = screen_frame.origin.x + screen_frame.size.width - width - margin.right;
            let y = screen_frame.origin.y + screen_frame.size.height - height - margin.top;
            builder.position(x, y)
        }
        Position::TopLeft => {
            let x = screen_frame.origin.x + margin.left;
            let y = screen_frame.origin.y + screen_frame.size.height - height - margin.top;
            builder.position(x, y)
        }
        Position::BottomRight => {
            let x = screen_frame.origin.x + screen_frame.size.width - width - margin.right;
            let y = screen_frame.origin.y + margin.bottom;
            builder.position(x, y)
        }
        Position::BottomLeft => {
            let x = screen_frame.origin.x + margin.left;
            let y = screen_frame.origin.y + margin.bottom;
            builder.position(x, y)
        }
        Position::Center => builder.centered(),
        Position::Custom { x, y } => builder.position(x, y),
    };

    Ok(positioned)
}

/// Add a layer to the window builder.
///
/// If the layer has text content, creates a CATextLayer.
/// Otherwise, creates a CAShapeLayer.
fn add_layer(
    builder: WindowBuilder,
    config: &LayerConfig,
    parent_width: f64,
    parent_height: f64,
) -> WindowBuilder {
    // Check if this is a text layer
    if config.text.is_some() {
        builder.text_layer(&config.name, |text_builder| {
            convert_text_layer(text_builder, config, parent_width, parent_height)
        })
    } else {
        builder.layer(&config.name, |shape_builder| {
            convert_shape_layer(shape_builder, config, parent_width, parent_height)
        })
    }
}

/// Convert a LayerConfig to CAShapeLayerBuilder configuration.
fn convert_shape_layer(
    mut builder: CAShapeLayerBuilder,
    config: &LayerConfig,
    parent_width: f64,
    parent_height: f64,
) -> CAShapeLayerBuilder {
    // Apply shape
    if let Some(ref shape) = config.shape {
        builder = apply_shape(builder, shape);
    }

    // Calculate and apply position
    let position = calculate_position(config, parent_width, parent_height);
    builder = builder.position(position);

    // Apply fill color
    if let Some(fill) = config.fill {
        builder = builder.fill_color(core_animation::Color::from(fill));
    }

    // Apply stroke
    if let Some((color, width)) = config.stroke {
        builder = builder
            .stroke_color(core_animation::Color::from(color))
            .line_width(width);
    }

    // Apply opacity
    if config.opacity < 1.0 {
        builder = builder.opacity(config.opacity);
    }

    // Apply shadow/glow
    if let Some(ref shadow) = config.shadow {
        builder = builder
            .shadow_color(core_animation::Color::from(shadow.color))
            .shadow_radius(shadow.radius)
            .shadow_offset(shadow.offset.0, shadow.offset.1)
            .shadow_opacity(shadow.opacity);
    }

    // Apply animations
    for (idx, animation) in config.animations.iter().enumerate() {
        builder = add_animation(builder, animation, idx);
    }

    builder
}

/// Convert a LayerConfig to CATextLayerBuilder configuration.
fn convert_text_layer(
    mut builder: CATextLayerBuilder,
    config: &LayerConfig,
    parent_width: f64,
    parent_height: f64,
) -> CATextLayerBuilder {
    // Apply text content
    if let Some(ref text) = config.text {
        builder = builder.text(text);
    }

    // Apply font properties
    builder = builder.font_size(config.font_size);

    if let Some(ref family) = config.font_family {
        builder = builder.font_name(family);
    }

    // Apply text color (use text_color, then fall back to fill)
    if let Some(color) = config.text_color {
        builder = builder.foreground_color(core_animation::Color::from(color));
    } else if let Some(fill) = config.fill {
        builder = builder.foreground_color(core_animation::Color::from(fill));
    }

    // Apply text alignment
    builder = builder.alignment(config.text_align.into());

    // Calculate and apply position
    let position = calculate_text_position(config, parent_width, parent_height);
    builder = builder.position(position);

    // Apply bounds for text sizing
    // Use font size to estimate height, width based on parent or shape
    let text_height = config.font_size * 1.5; // Approximate line height
    let text_width = config
        .shape
        .as_ref()
        .map(|s| match s {
            ShapeKind::Circle { diameter } => *diameter,
            ShapeKind::Ellipse { width, .. } => *width,
            ShapeKind::RoundedRect { width, .. } => *width,
        })
        .unwrap_or(parent_width);
    builder = builder.size(text_width, text_height);

    // Apply opacity
    if config.opacity < 1.0 {
        builder = builder.opacity(config.opacity);
    }

    // Apply text-specific opacity if set
    if let Some(text_opacity) = config.text_opacity {
        builder = builder.opacity(text_opacity);
    }

    // Apply shadow/glow
    if let Some(ref shadow) = config.shadow {
        builder = builder
            .shadow_color(core_animation::Color::from(shadow.color))
            .shadow_radius(shadow.radius)
            .shadow_offset(shadow.offset.0, shadow.offset.1)
            .shadow_opacity(shadow.opacity);
    }

    // Apply animations (text layers support the same animations)
    for (idx, animation) in config.animations.iter().enumerate() {
        builder = add_text_animation(builder, animation, idx);
    }

    builder
}

/// Calculate the position for a text layer.
fn calculate_text_position(config: &LayerConfig, parent_width: f64, parent_height: f64) -> CGPoint {
    match config.position {
        LayerPosition::Center => CGPoint::new(parent_width / 2.0, parent_height / 2.0),
        LayerPosition::CenterOffset { dx, dy } => {
            CGPoint::new(parent_width / 2.0 + dx, parent_height / 2.0 + dy)
        }
        LayerPosition::Absolute { x, y } => {
            // For text, position is the anchor point
            CGPoint::new(x, y)
        }
    }
}

/// Add an animation to a text layer builder.
fn add_text_animation(
    builder: CATextLayerBuilder,
    animation: &Animation,
    idx: usize,
) -> CATextLayerBuilder {
    match animation {
        Animation::Pulse {
            min_scale,
            max_scale,
            duration,
            easing,
        } => {
            let name = format!("pulse_{}", idx);
            builder.animate(&name, KeyPath::TransformScale, |a| {
                a.values(*min_scale, *max_scale)
                    .duration(*duration)
                    .easing((*easing).into())
                    .autoreverses()
                    .repeat(Repeat::Forever)
            })
        }
        Animation::Fade {
            from,
            to,
            duration,
            easing,
            autoreverses,
            repeat,
        } => {
            let name = format!("fade_{}", idx);
            builder.animate(&name, KeyPath::Opacity, |a| {
                let mut anim = a
                    .values(*from as f64, *to as f64)
                    .duration(*duration)
                    .easing((*easing).into())
                    .repeat((*repeat).into());
                if *autoreverses {
                    anim = anim.autoreverses();
                }
                anim
            })
        }
        Animation::Glow {
            min_radius,
            max_radius,
            duration,
            easing,
        } => {
            let name = format!("glow_{}", idx);
            builder.animate(&name, KeyPath::ShadowRadius, |a| {
                a.values(*min_radius, *max_radius)
                    .duration(*duration)
                    .easing((*easing).into())
                    .autoreverses()
                    .repeat(Repeat::Forever)
            })
        }
        Animation::Rotate {
            from,
            to,
            duration,
            easing,
            repeat,
        } => {
            let name = format!("rotate_{}", idx);
            builder.animate(&name, KeyPath::TransformRotation, |a| {
                a.values(*from, *to)
                    .duration(*duration)
                    .easing((*easing).into())
                    .repeat((*repeat).into())
            })
        }
        Animation::Group(animations) => {
            // Apply all animations in the group
            let mut b = builder;
            for (sub_idx, anim) in animations.iter().enumerate() {
                b = add_text_animation(b, anim, idx * 100 + sub_idx);
            }
            b
        }
    }
}

/// Apply shape configuration to the builder.
fn apply_shape(builder: CAShapeLayerBuilder, shape: &ShapeKind) -> CAShapeLayerBuilder {
    match shape {
        ShapeKind::Circle { diameter } => builder.circle(*diameter),
        ShapeKind::Ellipse { width, height } => builder.ellipse(*width, *height),
        ShapeKind::RoundedRect {
            width,
            height,
            corner_radius,
        } => {
            // For rounded rectangles, we need to create a custom path
            // For now, fall back to an ellipse-ish shape
            // TODO: Add rounded_rect support to core-animation
            let rect = CGRect::new(CGPoint::ZERO, CGSize::new(*width, *height));
            let path = unsafe {
                CGPath::with_rounded_rect(rect, *corner_radius, *corner_radius, std::ptr::null())
            };
            builder.path(path).bounds(rect)
        }
    }
}

/// Calculate the position for a layer based on its configuration.
fn calculate_position(config: &LayerConfig, parent_width: f64, parent_height: f64) -> CGPoint {
    // Get shape dimensions for centering calculations
    let (shape_width, shape_height) = config
        .shape
        .as_ref()
        .map(|s| match s {
            ShapeKind::Circle { diameter } => (*diameter, *diameter),
            ShapeKind::Ellipse { width, height } => (*width, *height),
            ShapeKind::RoundedRect { width, height, .. } => (*width, *height),
        })
        .unwrap_or((0.0, 0.0));

    match config.position {
        LayerPosition::Center => CGPoint::new(parent_width / 2.0, parent_height / 2.0),
        LayerPosition::CenterOffset { dx, dy } => {
            CGPoint::new(parent_width / 2.0 + dx, parent_height / 2.0 + dy)
        }
        LayerPosition::Absolute { x, y } => {
            // Convert from top-left origin to center-based position
            CGPoint::new(x + shape_width / 2.0, y + shape_height / 2.0)
        }
    }
}

/// Add an animation to the layer builder.
fn add_animation(
    builder: CAShapeLayerBuilder,
    animation: &Animation,
    idx: usize,
) -> CAShapeLayerBuilder {
    match animation {
        Animation::Pulse {
            min_scale,
            max_scale,
            duration,
            easing,
        } => {
            let name = format!("pulse_{}", idx);
            builder.animate(&name, KeyPath::TransformScale, |a| {
                a.values(*min_scale, *max_scale)
                    .duration(*duration)
                    .easing((*easing).into())
                    .autoreverses()
                    .repeat(Repeat::Forever)
            })
        }
        Animation::Fade {
            from,
            to,
            duration,
            easing,
            autoreverses,
            repeat,
        } => {
            let name = format!("fade_{}", idx);
            builder.animate(&name, KeyPath::Opacity, |a| {
                let mut anim = a
                    .values(*from as f64, *to as f64)
                    .duration(*duration)
                    .easing((*easing).into())
                    .repeat((*repeat).into());
                if *autoreverses {
                    anim = anim.autoreverses();
                }
                anim
            })
        }
        Animation::Glow {
            min_radius,
            max_radius,
            duration,
            easing,
        } => {
            let name = format!("glow_{}", idx);
            builder.animate(&name, KeyPath::ShadowRadius, |a| {
                a.values(*min_radius, *max_radius)
                    .duration(*duration)
                    .easing((*easing).into())
                    .autoreverses()
                    .repeat(Repeat::Forever)
            })
        }
        Animation::Rotate {
            from,
            to,
            duration,
            easing,
            repeat,
        } => {
            let name = format!("rotate_{}", idx);
            builder.animate(&name, KeyPath::TransformRotation, |a| {
                a.values(*from, *to)
                    .duration(*duration)
                    .easing((*easing).into())
                    .repeat((*repeat).into())
            })
        }
        Animation::Group(animations) => {
            // Apply all animations in the group
            let mut b = builder;
            for (sub_idx, anim) in animations.iter().enumerate() {
                b = add_animation(b, anim, idx * 100 + sub_idx);
            }
            b
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Size;
    use crate::layout::Margin;

    #[test]
    fn test_convert_window_level() {
        let level: WindowLevel = OsdWindowLevel::AboveAll.into();
        assert!(matches!(level, WindowLevel::AboveAll));

        let level: WindowLevel = OsdWindowLevel::Normal.into();
        assert!(matches!(level, WindowLevel::Normal));

        let level: WindowLevel = OsdWindowLevel::Custom(500).into();
        assert!(matches!(level, WindowLevel::Custom(500)));
    }

    #[test]
    fn test_convert_color() {
        let osd_color = Color::rgba(1.0, 0.5, 0.25, 0.8);
        let ca_color: core_animation::Color = osd_color.into();
        // Color conversion should preserve values
        assert_eq!(ca_color.r, 1.0);
        assert_eq!(ca_color.g, 0.5);
        assert_eq!(ca_color.b, 0.25);
        assert_eq!(ca_color.a, 0.8);
    }

    #[test]
    fn test_convert_easing() {
        let easing: Easing = OsdEasing::Linear.into();
        assert!(matches!(easing, Easing::Linear));

        let easing: Easing = OsdEasing::InOut.into();
        assert!(matches!(easing, Easing::InOut));

        let easing: Easing = OsdEasing::In.into();
        assert!(matches!(easing, Easing::In));

        let easing: Easing = OsdEasing::Out.into();
        assert!(matches!(easing, Easing::Out));
    }

    #[test]
    fn test_convert_repeat() {
        let repeat: Repeat = OsdRepeat::Once.into();
        assert!(matches!(repeat, Repeat::Once));

        let repeat: Repeat = OsdRepeat::Forever.into();
        assert!(matches!(repeat, Repeat::Forever));

        let repeat: Repeat = OsdRepeat::Times(5).into();
        assert!(matches!(repeat, Repeat::Times(5)));
    }

    #[test]
    fn test_convert_text_align() {
        let align: core_animation::TextAlign = TextAlign::Left.into();
        assert!(matches!(align, core_animation::TextAlign::Left));

        let align: core_animation::TextAlign = TextAlign::Center.into();
        assert!(matches!(align, core_animation::TextAlign::Center));

        let align: core_animation::TextAlign = TextAlign::Right.into();
        assert!(matches!(align, core_animation::TextAlign::Right));
    }

    #[test]
    fn test_config_creation() {
        let config = OsdConfig {
            size: Size::square(80.0),
            position: Position::TopRight,
            margin: Margin::all(20.0),
            level: OsdWindowLevel::AboveAll,
            background: Some(Color::rgba(0.1, 0.1, 0.1, 0.9)),
            corner_radius: 12.0,
            layers: vec![],
        };

        assert_eq!(config.size.width, 80.0);
        assert_eq!(config.size.height, 80.0);
    }
}
