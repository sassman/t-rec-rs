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
//! - [`wallpapers::Wallpaper`] - Wallpaper source (Ventura, or custom path)
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
//! use t_rec::types::{Decor, BackgroundColor};
//! use t_rec::wallpapers::Wallpaper;
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

// Core shared modules - made public so binary can use it directly
// This avoids code duplication between lib and bin targets
pub mod core;

// Library API (only compiled when not building CLI binary)
#[cfg(not(feature = "cli"))]
mod api;

// Re-export core types
pub use core::{Image, ImageOnHeap, Margin, PlatformApi, Result, WindowId, WindowList, WindowListEntry};

// Re-export public modules
pub use core::error;
pub use core::types;
pub use core::types::{BackgroundColor, Decor};
pub use core::wallpapers;
pub use core::wallpapers::{resolve_wallpaper, Wallpaper};
#[cfg(not(feature = "cli"))]
pub use core::wallpapers::load_and_validate_wallpaper;

// Re-export headless recorder API (only when not building CLI)
#[cfg(not(feature = "cli"))]
pub use api::{HeadlessRecorder, HeadlessRecorderBuilder, HeadlessRecorderConfig, RecordingOutput};

// Re-export CLI-only items when cli feature is enabled
// These are used by the binary but need to be exported to avoid dead code warnings
#[cfg(feature = "cli")]
pub use core::common::{Platform, PlatformApiFactory};
#[cfg(feature = "cli")]
pub use core::event_router::{CaptureEvent, Event, EventRouter, FlashEvent, LifecycleEvent};
#[cfg(feature = "cli")]
pub use core::post_processing::post_process_screenshots;
#[cfg(feature = "cli")]
pub use core::screenshot::{screenshot_file_name, screenshot_output_name, ScreenshotInfo};
#[cfg(feature = "cli")]
#[cfg(target_os = "macos")]
pub use core::macos::DEFAULT_SHELL;
#[cfg(feature = "cli")]
#[cfg(target_os = "linux")]
pub use core::linux::DEFAULT_SHELL;
