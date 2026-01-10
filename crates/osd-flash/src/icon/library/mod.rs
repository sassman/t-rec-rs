//! Pre-built icon library.
//!
//! This module provides ready-to-use icon builders for common use cases.
//! Each icon builder allows customization while providing sensible defaults.
//!
//! # Available Icons
//!
//! - [`CameraIcon`] - Camera icon for screenshot feedback
//! - [`RecordingIcon`] - Recording indicator dot (static)
//! - [`PulsingRecordingIcon`] - Recording indicator with pulsing animation
//!
//! # Example
//!
//! ```ignore
//! use osd_flash::icon::{CameraIcon, RecordingIcon, PulsingRecordingIcon};
//!
//! // Screenshot indicator
//! let camera = CameraIcon::new(120.0).build();
//!
//! // Static recording indicator with custom color
//! let recording = RecordingIcon::new(80.0)
//!     .dot_color(Color::GREEN)
//!     .build();
//!
//! // Animated recording indicator
//! let pulsing = PulsingRecordingIcon::new(80.0);
//! // Use pulsing.icon() for the base icon, then pulsing.animate(window)
//! ```

mod camera;
mod pulsing_recording;
mod recording;

pub use camera::CameraIcon;
pub use pulsing_recording::PulsingRecordingIcon;
pub use recording::RecordingIcon;
