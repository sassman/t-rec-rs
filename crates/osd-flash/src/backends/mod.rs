//! Backend implementations for on-screen display.
//!
//! This module provides access to platform-specific backends for advanced usage.
//! Most users should use the high-level API via `OsdBuilder` instead.
//!
//! Currently supported backends:
//! - `macos` - macOS Core Animation (GPU-accelerated)
//!
//! # Example (advanced usage)
//!
//! ```ignore
//! use osd_flash::backends::macos::MacOsWindow;
//! ```

#[cfg(target_os = "macos")]
pub mod macos;
