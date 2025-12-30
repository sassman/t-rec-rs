//! Backend implementations for on-screen display.
//!
//! Currently supported backends:
//! - `skylight` - macOS SkyLight private API (default on macOS)

#[cfg(target_os = "macos")]
pub mod skylight;

// Re-export the default backend for the current platform
#[cfg(target_os = "macos")]
pub use skylight::*;
