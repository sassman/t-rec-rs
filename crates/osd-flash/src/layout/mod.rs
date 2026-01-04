//! Layout types for positioning and spacing.
//!
//! This module provides abstractions for the CSS-like box model:
//! - [`Margin`] - External spacing around the border box
//! - [`Padding`] - Internal spacing between border and content
//! - [`Border`] - Border specification (width affects content bounds)
//! - [`LayoutBox`] - Combines bounds, margin, border, and padding

mod border;
mod box_model;
mod margin;
mod padding;

pub use border::Border;
pub use box_model::LayoutBox;
pub use margin::Margin;
pub use padding::Padding;
