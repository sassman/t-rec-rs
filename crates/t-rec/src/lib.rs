//! t-rec - Terminal Recorder Library
//!
//! This library provides programmatic access to t-rec's recording capabilities.
//! It is primarily designed for recording animations and GUI windows to GIF and MP4 format.
//!
//! # Feature Flags
//!
//! - `lib` - Enables the library API with [`HeadlessRecorder`]
//!
//! # Type-Safe Configuration
//!
//! The library uses strongly-typed enums for configuration options, providing compile-time
//! safety and clear documentation of valid values:
//!
//! - [`types::Decor`] - Decoration style (None, Shadow)
//! - [`types::BackgroundColor`] - Background color (Transparent, White, Black, or custom hex)
//! - [`types::Wallpaper`] - Wallpaper source (Ventura, or custom path)
//!
//! Validation happens at enum construction time via factory methods:
//! - `BackgroundColor::custom("#ff0000")` - Validates hex format
//! - `Wallpaper::custom("/path/to/image.png")` - Validates path exists
//!
//! The builder is always infallible - only `build()` can fail (for missing required fields).
//!
//! # Example
//!
//! ```ignore
//! use t_rec::HeadlessRecorder;
//! use t_rec::types::{Decor, BackgroundColor, Wallpaper};
//!
//! // Type-safe enum API (compile-time checked)
//! let mut recorder = HeadlessRecorder::builder()
//!     .window_id(12345)
//!     .fps(30)
//!     .decor(Decor::Shadow)
//!     .bg_color(BackgroundColor::White)
//!     .wallpaper(Wallpaper::Ventura, 60)
//!     .output_gif("demo.gif")
//!     .build()?;
//!
//! // Custom values use factory methods for validation
//! let mut recorder = HeadlessRecorder::builder()
//!     .window_id(12345)
//!     .bg_color(BackgroundColor::custom("#ff5500")?)  // Validates hex format
//!     .wallpaper(Wallpaper::custom("/path/to/bg.png")?, 80)  // Validates path exists
//!     .output_gif("demo.gif")
//!     .build()?;
//!
//! // Start recording
//! recorder.start()?;
//!
//! // ... run your animation or GUI ...
//!
//! // Stop and generate output files
//! let output = recorder.stop_and_generate()?;
//! println!("Captured {} frames", output.frame_count);
//! if let Some(gif_path) = output.gif_path {
//!     println!("GIF saved to: {}", gif_path.display());
//! }
//! if let Some(mp4_path) = output.mp4_path {
//!     println!("MP4 saved to: {}", mp4_path.display());
//! }
//! ```

// Platform-specific modules
#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

// Core modules needed for headless recording
mod assets;
mod capture;
mod common;
mod decors;
mod event_router;
mod generators;
mod post_processing;
mod screenshot;
pub mod types;
mod utils;
mod wallpapers;

// Headless recorder module (only with lib feature)
#[cfg(feature = "lib")]
pub mod headless;

// Re-export common types
use image::FlatSamples;

/// A captured image stored on the heap.
pub type Image = FlatSamples<Vec<u8>>;
/// Boxed image type for efficient passing.
pub type ImageOnHeap = Box<Image>;
/// Window identifier (platform-specific).
pub type WindowId = u64;
/// List of windows with optional names.
pub type WindowList = Vec<WindowListEntry>;
/// A window entry: optional name and ID.
pub type WindowListEntry = (Option<String>, WindowId);
/// Result type using anyhow for error handling.
pub type Result<T> = anyhow::Result<T>;

// Re-export Margin for other modules
pub use crate::common::Margin;

// Re-export platform API trait
pub use crate::common::PlatformApi;

// Re-export the main headless API
#[cfg(feature = "lib")]
pub use headless::{
    HeadlessRecorder, HeadlessRecorderBuilder, HeadlessRecorderConfig, RecordingOutput,
};
