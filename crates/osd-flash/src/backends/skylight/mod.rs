//! SkyLight backend for macOS.
//!
//! Uses Apple's private SkyLight framework to create overlay windows
//! that appear above all other content, including fullscreen apps.
//!
//! # Requirements
//! - macOS 10.14+
//! - Runs on main thread (requires CFRunLoop)
//!
//! # API Levels
//!
//! This module provides two levels of API:
//!
//! - **High-level** (`SkylightOsdWindow`): Used internally by `OsdFlashBuilder::build()`.
//!   Most users should use `OsdFlashBuilder` from the prelude instead.
//!
//! - **Low-level** (`SkylightWindow`, `SkylightWindowBuilder`, `SkylightCanvas`):
//!   Direct access to SkyLight primitives for advanced use cases like custom
//!   window management or direct Core Graphics rendering.

mod canvas;
pub(crate) mod cg_patches;
mod geometry_ext;
mod osd_window;
mod window;

// High-level API (used by OsdFlashBuilder)
pub(crate) use osd_window::SkylightOsdWindow;

// Low-level API (for advanced users)
pub use canvas::SkylightCanvas;
pub use window::{SkylightWindow, SkylightWindowBuilder};
pub use window::{DisplayTarget as SkylightDisplayTarget, WindowLevel as SkylightWindowLevel};
