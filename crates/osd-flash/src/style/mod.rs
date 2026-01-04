//! Styling types for rendering shapes and text.
//!
//! This module provides abstractions for visual styling:
//! - [`Paint`] - How to fill a shape (color, opacity)
//! - [`TextStyle`] - Text rendering specification (font, size, weight, alignment)

mod paint;
mod text_style;

pub use paint::Paint;
pub use text_style::{FontWeight, TextAlignment, TextStyle};
