//! Core modules shared between library and CLI.

pub mod assets;
pub mod capture;
pub mod common;
pub mod decors;
pub mod error;
pub mod event_router;
pub mod generators;
pub mod post_processing;
pub mod screenshot;
pub mod types;
pub mod utils;
pub mod wallpapers;

// Platform-specific modules
#[cfg(any(target_os = "linux", target_os = "netbsd"))]
pub mod linux;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod win;
#[cfg(target_os = "windows")]
pub mod windows;

// Re-export common types used throughout
pub use common::{Margin, PlatformApi};

// Re-export image types
use image::FlatSamples;
pub type Image = FlatSamples<Vec<u8>>;
pub type ImageOnHeap = Box<Image>;
pub type WindowId = u64;
pub type WindowList = Vec<WindowListEntry>;
pub type WindowListEntry = (Option<String>, WindowId);
pub type Result<T> = anyhow::Result<T>;
