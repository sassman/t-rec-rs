//! Rust bindings for macOS Core Animation with builder APIs.
//!
//! Core Animation is Apple's GPU-accelerated rendering system. This crate
//! wraps it with ergonomic builders, focusing on **particle effects** and
//! **layer composition**.
//!
//! # Quick Start
//!
//! Particles burst outward from a point:
//!
//! ```ignore
//! use std::f64::consts::PI;
//! use core_animation::prelude::*;
//!
//! let emitter = CAEmitterLayerBuilder::new()
//!     .position(320.0, 240.0)
//!     .shape(EmitterShape::Point)
//!     .particle(|p| {
//!         p.birth_rate(100.0)           // spawn rate
//!             .lifetime(5.0)            // seconds until particle disappears
//!             .velocity(80.0)           // movement speed
//!             .emission_range(PI * 2.0) // spread angle (full circle)
//!             .color(Color::CYAN)
//!             .image(ParticleImage::soft_glow(64))
//!     })
//!     .build();
//!
//! window.container().add_sublayer(&emitter);
//! window.show_for(10.seconds());
//! ```
//!
//! Simpler API for the same effect:
//!
//! ```ignore
//! let burst = PointBurstBuilder::new(320.0, 240.0)
//!     .velocity(100.0)
//!     .color(Color::PINK)
//!     .build();
//! ```
//!
//! # Examples
//!
//! See [`examples/README.md`](https://github.com/sassman/t-rec-rs/tree/main/crates/core-animation/examples)
//! for runnable demos with screenshots.
//!
//! ```bash
//! cargo run -p core-animation --example emitter
//! ```
//!
//! # Modules
//!
//! - [`animation_builder`] - GPU-accelerated animations ([`CABasicAnimationBuilder`](animation_builder::CABasicAnimationBuilder),
//!   [`KeyPath`](animation_builder::KeyPath), [`Easing`](animation_builder::Easing))
//! - [`particles`] - Particle emitter builders ([`CAEmitterLayerBuilder`](particles::CAEmitterLayerBuilder),
//!   [`PointBurstBuilder`](particles::PointBurstBuilder), [`ParticleImage`](particles::ParticleImage))
//! - [`window`] - Test window for examples ([`WindowBuilder`])
//!
//! # Types
//!
//! **This crate:**
//! - [`Color`] - RGBA color with presets (`Color::CYAN`, `Color::rgb(...)`)
//! - [`CALayerBuilder`] - Generic layer builder
//! - [`CAShapeLayerBuilder`] - Vector shape builder
//! - [`DurationExt`] - `5.seconds()`, `500.millis()` syntax
//!
//! **Re-exported from Apple frameworks:**
//! - [`CALayer`], [`CAShapeLayer`], [`CATransform3D`] - Core Animation
//! - [`CGPoint`](objc2_core_foundation::CGPoint), [`CGSize`](objc2_core_foundation::CGSize),
//!   [`CGRect`](objc2_core_foundation::CGRect) - Geometry
//! - [`CGPath`](objc2_core_graphics::CGPath), [`CGColor`](objc2_core_graphics::CGColor) - Graphics
//!
//! Use [`prelude`] to import common types.

#![cfg(target_os = "macos")]

pub mod animation_builder;
mod color;
mod duration_ext;
mod layer_builder;
mod layer_ext;
pub mod particles;
mod shape_layer_builder;
pub mod window;

// Re-export Color type
pub use color::Color;

// Re-export the main types from objc2-quartz-core
pub use objc2_quartz_core::{CALayer, CAShapeLayer, CATransform3D};

// Re-export our builders
pub use layer_builder::CALayerBuilder;
pub use shape_layer_builder::CAShapeLayerBuilder;

// Re-export window types
pub use window::{Screen, Window, WindowBuilder, WindowLevel, WindowStyle};

// Re-export duration extension
pub use duration_ext::DurationExt;

// Re-export layer extension
pub use layer_ext::CALayerExt;

// Re-export dependencies for convenience
pub use objc2_core_foundation;
pub use objc2_core_graphics;
pub use objc2_core_text;
pub use objc2_quartz_core;

/// Prelude module for convenient imports.
pub mod prelude {
    // Color type
    pub use crate::color::Color;

    // Animation builder types
    pub use crate::animation_builder::{CABasicAnimationBuilder, Easing, KeyPath, Repeat};

    // Builders
    pub use crate::layer_builder::CALayerBuilder;
    pub use crate::particles::{
        CAEmitterCellBuilder, CAEmitterLayerBuilder, EmitterMode, EmitterShape, ParticleImage,
        PointBurstBuilder, RenderMode,
    };
    pub use crate::shape_layer_builder::CAShapeLayerBuilder;
    pub use crate::window::{Screen, Window, WindowBuilder, WindowLevel, WindowStyle};

    // Duration extension for ergonomic timing
    pub use crate::duration_ext::DurationExt;

    // Layer extension for snake_case methods
    pub use crate::layer_ext::CALayerExt;

    // Core Animation types
    pub use crate::{CALayer, CAShapeLayer, CATransform3D};
    pub use objc2_quartz_core::CABasicAnimation;

    // Core Foundation types (geometry, strings, collections, run loop)
    pub use objc2_core_foundation::{
        kCFRunLoopDefaultMode, kCFTypeDictionaryKeyCallBacks, kCFTypeDictionaryValueCallBacks,
        CFAttributedString, CFDictionary, CFDictionaryKeyCallBacks, CFDictionaryValueCallBacks,
        CFIndex, CFRetained, CFRunLoop, CFString, CFStringBuiltInEncodings, CFTimeInterval,
        CGAffineTransform, CGFloat, CGPoint, CGRect, CGSize,
    };

    // Core Graphics types (context, colors, paths, transforms, display)
    pub use objc2_core_graphics::{
        CGAffineTransformIdentity, CGColor, CGContext, CGDirectDisplayID, CGDisplayBounds,
        CGMainDisplayID, CGPath,
    };

    // Core Text types (fonts, lines, string attributes)
    pub use objc2_core_text::{
        kCTFontAttributeName, kCTForegroundColorAttributeName, CTFont, CTLine,
    };

    // AppKit types (NSApplication)
    pub use objc2_app_kit::NSApplication;

    // Smart pointer for Objective-C objects
    pub use objc2::rc::Retained;
}
