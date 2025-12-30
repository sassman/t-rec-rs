//! Geometry types for the screen flash module.
//!
//! Provides simple, ergonomic types for working with positions, sizes, rectangles, and margins.

mod margin;
mod point;
mod rect;
mod size;

pub use margin::Margin;
pub use point::Point;
pub use rect::Rect;
pub use size::Size;
