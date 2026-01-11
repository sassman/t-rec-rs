//! Pre-built layer compositions for common OSD patterns.
//!
//! This module provides ready-to-use compositions that implement
//! `Into<LayerComposition>`, allowing them to be passed directly to
//! `OsdBuilder::composition()`.
//!
//! # Examples
//!
//! ```ignore
//! use osd_flash::prelude::*;
//!
//! // Recording indicator with default settings
//! OsdBuilder::new()
//!     .composition(RecordingIndicator::new())
//!     .show_for(10.seconds())?;
//!
//! // Camera flash with custom size
//! OsdBuilder::new()
//!     .composition(CameraFlash::new().size(150.0))
//!     .show_for(2.seconds())?;
//! ```

mod camera;
mod recording;

pub use camera::CameraFlash;
pub use recording::RecordingIndicator;
