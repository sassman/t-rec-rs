//! Layout types for positioning and spacing.
//!
//! Provides CSS-like box model abstractions:
//! - `Margin` - external spacing around the border box
//! - `Padding` - internal spacing between border and content
//! - `Border` - border specification (width affects content bounds)
//! - `LayoutBox` - combines bounds, margin, border, and padding

mod border;
mod box_model;
mod margin;
mod padding;

pub use border::Border;
pub use box_model::LayoutBox;
pub use margin::Margin;
pub use padding::Padding;
