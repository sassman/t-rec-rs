//! On-screen display (OSD) flash indicators.
//!
//! This crate provides a platform-agnostic API for displaying brief
//! on-screen indicators with GPU-accelerated animations.
//!
//! # Quick Start
//!
//! ```ignore
//! use osd_flash::prelude::*;
//!
//! // Simple recording indicator with pulsing animation
//! OsdBuilder::new()
//!     .size(80.0)
//!     .position(Position::TopRight)
//!     .margin(20.0)
//!     .composition(RecordingIndicator::new())
//!     .show_for(10.seconds())?;
//! ```
//!
//! # Architecture
//!
//! The crate provides:
//! - **`OsdBuilder`** - Main entry point for creating OSD windows
//! - **`LayerComposition`** - Declarative layer configuration with animations
//! - **`Animation`** - GPU-accelerated animation presets (pulse, fade, glow, etc.)
//! - **Geometry types** - Platform-agnostic `Point`, `Size`, `Rect`
//!
//! # Inline Layer Definition
//!
//! ```ignore
//! use osd_flash::prelude::*;
//!
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
//!
//! # Pre-built Compositions
//!
//! The library provides ready-to-use compositions:
//!
//! ```ignore
//! use osd_flash::prelude::*;
//!
//! // Recording indicator with pulsing dot
//! OsdBuilder::new()
//!     .composition(RecordingIndicator::new())
//!     .show_for(10.seconds())?;
//!
//! // Camera flash icon
//! OsdBuilder::new()
//!     .composition(CameraFlash::new())
//!     .show_for(3.seconds())?;
//! ```

// New modules (platform-agnostic)
mod builder;
mod color;
mod duration_ext;
mod error;
mod level;
mod position;

/// Composition types for declarative OSD content.
pub mod composition;

/// Geometry types for positioning and sizing.
pub mod geometry;

/// Layout types for spacing (margin, padding).
pub mod layout;

/// Pre-built layer compositions for common OSD patterns.
pub mod library;

/// Platform-specific backend implementations.
pub mod backends;

// Public re-exports (new API)
pub use builder::{OsdBuilder, OsdConfig};
pub use color::Color;
pub use composition::{
    Animation, CompositionBuilder, Easing, LayerBuilder, LayerComposition, Repeat,
};
pub use duration_ext::DurationExt;
pub use error::{Error, Result};
pub use geometry::{Point, Rect, Size};
pub use layout::Margin;
pub use level::WindowLevel;
pub use library::{CameraFlash, RecordingIndicator};
pub use position::Position;

/// Prelude for convenient imports.
///
/// This module exports the platform-agnostic public API. For advanced usage
/// requiring direct backend access, import from `osd_flash::backends` directly.
pub mod prelude {
    pub use crate::builder::{OsdBuilder, OsdConfig};
    pub use crate::color::Color;
    pub use crate::composition::{
        Animation, CompositionBuilder, Easing, FontWeight, LayerBuilder, LayerComposition,
        LayerConfig, LayerPosition, Repeat, ShadowConfig, ShapeKind, TextAlign,
    };
    pub use crate::duration_ext::DurationExt;
    pub use crate::error::{Error, Result};
    pub use crate::geometry::{Point, Rect, Size};
    pub use crate::layout::Margin;
    pub use crate::level::WindowLevel;
    pub use crate::library::{CameraFlash, RecordingIndicator};
    pub use crate::position::Position;
}
