//! t-rec - Terminal Recorder Library
//!
//! This library provides programmatic access to t-rec's recording capabilities.
//! It is primarily designed for recording animations and GUI windows to GIF and MP4 format.
//!
//! # Feature Flags
//!
//! - `headless` — Enables the [`HeadlessRecorder`] public API (opt-in; no extra deps).
//! - `bin` — Bundled by default; enables the `t-rec` binary and its CLI deps.
//!   Library consumers should use `--no-default-features` to exclude it.
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
//! - `"/path/to/image.png".parse::<Wallpaper>()` - Parses into a `Wallpaper::Custom` variant
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
//! // Custom values use factory methods / FromStr for validation
//! let mut recorder = HeadlessRecorder::builder()
//!     .window_id(12345)
//!     .bg_color(BackgroundColor::custom("#ff5500")?)  // Validates hex format
//!     .wallpaper("/path/to/bg.png".parse::<Wallpaper>()?, 80)  // Parses into Wallpaper::Custom
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

// Core shared modules — used by both the binary and library consumers.
pub mod core;

// Public library API — opt-in via the `headless` feature.
#[cfg(feature = "headless")]
mod api;

#[cfg(feature = "headless")]
pub use api::{HeadlessRecorder, HeadlessRecorderBuilder, HeadlessRecorderConfig, RecordingOutput};

// Re-export core types.
pub use core::{
    Image, ImageOnHeap, Margin, PlatformApi, Result, WindowId, WindowList, WindowListEntry,
};

// Re-export public modules.
pub use core::error;
pub use core::types;
pub use core::types::{BackgroundColor, Decor};
pub use core::wallpapers;
pub use core::wallpapers::{load_and_validate_wallpaper, resolve_wallpaper, Wallpaper};

// Re-exports used by the binary and available to any consumer.
pub use core::common::{Platform, PlatformApiFactory};
pub use core::event_router::{CaptureEvent, Event, EventRouter, FlashEvent, LifecycleEvent};
#[cfg(target_os = "linux")]
pub use core::linux::DEFAULT_SHELL;
#[cfg(target_os = "macos")]
pub use core::macos::DEFAULT_SHELL;
pub use core::post_processing::post_process_screenshots;
pub use core::screenshot::{screenshot_file_name, screenshot_output_name, ScreenshotInfo};
#[cfg(target_os = "windows")]
pub use core::windows::DEFAULT_SHELL;
