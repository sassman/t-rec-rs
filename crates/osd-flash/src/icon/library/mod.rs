//! Pre-built icon library.
//!
//! This module provides ready-to-use icon builders for common use cases.
//! Each icon builder allows customization while providing sensible defaults.
//!
//! # Available Icons
//!
//! - [`CameraIcon`] - Camera icon for screenshot feedback
//! - [`RecordingIcon`] - Recording indicator dot
//!
//! # Example
//!
//! ```ignore
//! use osd_flash::icon::{CameraIcon, RecordingIcon};
//!
//! // Screenshot indicator
//! let camera = CameraIcon::new(120.0).build();
//!
//! // Recording indicator with custom color
//! let recording = RecordingIcon::new(80.0)
//!     .dot_color(Color::GREEN)
//!     .build();
//! ```

mod camera;
mod recording;

pub use camera::CameraIcon;
pub use recording::RecordingIcon;
