//! Error types for the t-rec library API.
//!
//! This module provides strongly-typed errors for library operations.

use std::fmt;
use std::path::PathBuf;

/// Errors that can occur when using the t-rec library API.
///
/// Note: This module is used by both the library and binary targets.
/// The binary doesn't use these types directly, hence the allow attribute.
#[derive(Debug, Clone)]
pub enum Error {
    /// Wallpaper file path does not exist
    WallpaperNotFound(PathBuf),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::WallpaperNotFound(path) => {
                write!(f, "Wallpaper file not found: {}", path.display())
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result type for library operations.
pub(crate) type Result<T> = std::result::Result<T, Error>;
