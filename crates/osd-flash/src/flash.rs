//! Flash indicator types and position definitions.
//!
//! This module provides the core types for positioning on-screen indicators.

pub use crate::color::Color;
pub use crate::geometry::{Margin, Point, Rect, Size};
pub use crate::icon::IconBuilder;

/// Position for the flash indicator on screen.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FlashPosition {
    /// Top-right corner (default, like macOS notifications)
    #[default]
    TopRight,
    /// Top-left corner
    TopLeft,
    /// Bottom-right corner
    BottomRight,
    /// Bottom-left corner
    BottomLeft,
    /// Center of screen
    Center,
    /// Custom position (x, y from top-left)
    Custom { x: f64, y: f64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flash_position_default() {
        assert_eq!(FlashPosition::default(), FlashPosition::TopRight);
    }
}
