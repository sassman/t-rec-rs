//! Common utilities for examples.
//!
//! This module provides helper functions used across multiple examples,
//! including the recording functionality.

#[cfg(feature = "record")]
mod recording;

#[cfg(feature = "record")]
pub use recording::show_with_recording;
