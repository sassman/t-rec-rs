//! Styling types for rendering shapes and text.
//!
//! - `Paint` - fill specification (color, opacity)
//! - `TextStyle` - text rendering (font, size, weight, alignment)

mod paint;
mod text_style;

pub use paint::Paint;
pub use text_style::{FontWeight, TextAlignment, TextStyle};
