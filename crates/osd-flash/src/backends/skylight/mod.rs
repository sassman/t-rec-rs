//! SkyLight backend for macOS.
//!
//! Uses Apple's private SkyLight framework to create overlay windows
//! that appear above all other content, including fullscreen apps.
//!
//! # Requirements
//! - macOS 10.14+
//! - Runs on main thread (requires NSRunLoop)

mod canvas;
pub(crate) mod cg_patches;
mod geometry_ext;
mod window;

pub use canvas::SkylightCanvas;
pub use window::{
    run_loop_for_seconds, DisplayTarget, SkylightWindow, SkylightWindowBuilder, WindowLevel,
};
