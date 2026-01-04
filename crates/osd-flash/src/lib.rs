//! On-screen display (OSD) flash indicators.
//!
//! This crate provides platform-specific backends for displaying brief
//! on-screen indicators, such as camera flash effects or status icons.
//!
//! # Architecture
//!
//! The crate is organized into:
//! - **Common types** (`Color`, `Shape`, `Icon`, `Canvas` trait) - platform-agnostic
//! - **Backends** - platform-specific implementations (`skylight` for macOS)
//!
//! # Example
//!
//! ```ignore
//! use osd_flash::prelude::*;
//!
//! // Use a pre-built icon from the library
//! let icon = CameraIcon::new(120.0).build();
//!
//! // Or build a custom icon
//! let custom = IconBuilder::new(120.0)
//!     .background(Color::VIBRANT_BLUE, 16.0)
//!     .circle(60.0, 60.0, 30.0, Color::WHITE)
//!     .build();
//! ```

// Common modules (platform-agnostic)
mod canvas;
mod color;
mod flash;
mod shape;
mod window;

/// Geometry types for positioning and sizing.
pub mod geometry;

/// Layout types for spacing (padding, border, box model).
pub mod layout;

/// Styling types for rendering (paint, text style).
pub mod style;

/// Icon building API for creating custom on-screen indicators.
pub mod icon;

/// Platform-specific backend implementations.
pub mod backends;

// TODO: once stable migrate to `thiserror` and own error types
pub use anyhow::Result;
pub use canvas::Canvas;
pub use color::Color;
pub use flash::*;
pub use shape::Shape;
pub use window::{DisplayTarget, Drawable, OsdFlashBuilder, OsdWindow, WindowLevel};

/// Prelude for convenient imports.
///
/// This module exports the platform-agnostic public API. For advanced usage
/// requiring direct backend access, import from `osd_flash::backends` directly.
pub mod prelude {
    pub use crate::canvas::Canvas;
    pub use crate::color::Color;
    pub use crate::geometry::{Point, Rect, Size};
    pub use crate::icon::{CameraIcon, Icon, IconBuilder, RecordingIcon, StyledShape, StyledText};
    pub use crate::layout::{Border, LayoutBox, Margin, Padding};
    pub use crate::shape::Shape;
    pub use crate::style::{FontWeight, Paint, TextAlignment, TextStyle};
    pub use crate::window::{DisplayTarget, Drawable, OsdFlashBuilder, OsdWindow, WindowLevel};
    pub use crate::FlashPosition;
}
