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
mod geometry;
mod shape;

/// Icon building API for creating custom on-screen indicators.
pub mod icon;

/// Platform-specific backend implementations.
#[cfg(target_os = "macos")]
pub mod backends;

// TODO: once stable migrate to `thiserror` and own error types
pub use anyhow::Result;
pub use canvas::Canvas;
pub use flash::*;
pub use shape::Shape;

/// Prelude for convenient imports.
pub mod prelude {
    // Common types
    pub use crate::canvas::Canvas;
    pub use crate::color::Color;
    pub use crate::geometry::{Point, Rect, Size};
    pub use crate::icon::{CameraIcon, Icon, IconBuilder, RecordingIcon};
    pub use crate::shape::Shape;
    pub use crate::{FlashConfig, FlashPosition};

    // Backend-specific types
    #[cfg(target_os = "macos")]
    pub use crate::backends::{
        run_loop_for_seconds, DisplayTarget, SkylightCanvas, SkylightWindow,
        SkylightWindowBuilder, WindowLevel,
    };
}
