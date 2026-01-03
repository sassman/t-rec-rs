//! Backend implementations for on-screen display.
//!
//! This module provides access to platform-specific backends for advanced usage.
//! Most users should use the high-level API via `OsdFlashBuilder` instead.
//!
//! Currently supported backends:
//! - `skylight` - macOS SkyLight private API (default on macOS)
//!
//! # Example (advanced usage)
//!
//! ```ignore
//! use osd_flash::backends::skylight::{SkylightWindowBuilder, WindowLevel};
//! ```

#[cfg(target_os = "macos")]
pub mod skylight;
