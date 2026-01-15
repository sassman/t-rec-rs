//! Error types for the osd-flash crate.
//!
//! This module provides typed errors for OSD window creation and display.

use std::fmt;

/// Error type for OSD operations.
#[derive(Debug)]
pub enum Error {
    /// Operation requires the main thread but was called from a different thread.
    ///
    /// On macOS, window creation and management must occur on the main thread.
    NotOnMainThread,

    /// No screen is available for positioning the OSD window.
    ///
    /// This can occur if the system has no displays connected.
    NoScreenAvailable,

    /// Platform is not supported.
    ///
    /// The OSD backend is not available on this platform.
    #[allow(dead_code)]
    UnsupportedPlatform,
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::NotOnMainThread => {
                write!(
                    f,
                    "OSD window operations must be performed on the main thread"
                )
            }
            Error::NoScreenAvailable => {
                write!(f, "no screen available for OSD positioning")
            }
            Error::UnsupportedPlatform => {
                write!(f, "OSD is not supported on this platform")
            }
        }
    }
}

/// Result type for OSD operations.
pub type Result<T> = std::result::Result<T, Error>;
