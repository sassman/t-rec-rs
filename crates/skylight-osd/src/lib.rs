mod color;
mod core_foundation_sys_patches;
mod drawing;
mod flash;
mod geometry;
mod skylight;

/// Icon building API for creating custom on-screen indicators.
pub mod icon;

// TODO: once stable migrate to `thiserror` and own error types
pub use anyhow::Result;
pub use flash::*;

pub mod prelude {
    pub use crate::color::Color;
    pub use crate::drawing::{Canvas, Shape};
    pub use crate::geometry::{Point, Rect, Size};
    pub use crate::icon::{Icon, IconBuilder};
    pub use crate::skylight::run_loop_for_seconds;
    pub use crate::skylight::{DisplayTarget, SkylightWindow, SkylightWindowBuilder, WindowLevel};
    pub use crate::{FlashConfig, FlashPosition};
}
