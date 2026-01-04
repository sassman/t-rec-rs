//! Box model for layout calculations.

use crate::geometry::Rect;

use super::{Border, Margin, Padding};

/// A layout box combining bounds with margin, border, and padding.
///
/// Follows the CSS box model:
/// ```text
/// ┌─────────────────────────── Margin Box ───────────────────────────┐
/// │                              margin                               │
/// │   ┌───────────────────── Border Box ─────────────────────────┐   │
/// │   │                        border                             │   │
/// │   │   ┌───────────────── Padding Box ───────────────────┐    │   │
/// │   │   │                    padding                       │    │   │
/// │   │   │   ┌──────────── Content Box ────────────────┐   │    │   │
/// │   │   │   │                                          │   │    │   │
/// │   │   │   │              content                     │   │    │   │
/// │   │   │   │                                          │   │    │   │
/// │   │   │   └──────────────────────────────────────────┘   │    │   │
/// │   │   │                    padding                       │    │   │
/// │   │   └──────────────────────────────────────────────────┘    │   │
/// │   │                        border                             │   │
/// │   └───────────────────────────────────────────────────────────┘   │
/// │                              margin                               │
/// └───────────────────────────────────────────────────────────────────┘
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutBox {
    /// The outer bounds (margin box).
    pub bounds: Rect,
    /// External spacing around the border box.
    pub margin: Margin,
    /// Border specification (width affects content bounds).
    pub border: Border,
    /// Internal spacing between border and content.
    pub padding: Padding,
}

impl LayoutBox {
    /// Create a new layout box with the given bounds.
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            margin: Margin::zero(),
            border: Border::none(),
            padding: Padding::zero(),
        }
    }

    /// Create a layout box from width and height, positioned at origin.
    pub fn from_size(width: f64, height: f64) -> Self {
        Self::new(Rect::from_xywh(0.0, 0.0, width, height))
    }

    /// Set the margin.
    pub fn with_margin(mut self, margin: impl Into<Margin>) -> Self {
        self.margin = margin.into();
        self
    }

    /// Set the border.
    pub fn with_border(mut self, border: Border) -> Self {
        self.border = border;
        self
    }

    /// Set the padding.
    pub fn with_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Get the border box bounds (inside margin, outside border).
    ///
    /// This is where the border is drawn.
    pub fn border_bounds(&self) -> Rect {
        Rect::from_xywh(
            self.bounds.origin.x + self.margin.left,
            self.bounds.origin.y + self.margin.top,
            self.bounds.size.width - self.margin.horizontal(),
            self.bounds.size.height - self.margin.vertical(),
        )
    }

    /// Get the padding box bounds (inside border, outside padding).
    ///
    /// This is the area inside the border.
    pub fn padding_bounds(&self) -> Rect {
        let border = self.border_bounds();
        Rect::from_xywh(
            border.origin.x + self.border.width,
            border.origin.y + self.border.width,
            border.size.width - self.border.horizontal(),
            border.size.height - self.border.vertical(),
        )
    }

    /// Get the content box bounds (inside padding).
    ///
    /// This is where content should be placed.
    pub fn content_bounds(&self) -> Rect {
        let padding_box = self.padding_bounds();
        Rect::from_xywh(
            padding_box.origin.x + self.padding.left,
            padding_box.origin.y + self.padding.top,
            padding_box.size.width - self.padding.horizontal(),
            padding_box.size.height - self.padding.vertical(),
        )
    }

    /// Get the center point of the content box.
    pub fn content_center(&self) -> crate::geometry::Point {
        self.content_bounds().center()
    }

    /// Get the total inset from the outer bounds to content.
    ///
    /// Returns (horizontal_inset, vertical_inset).
    pub fn total_inset(&self) -> (f64, f64) {
        let horizontal = self.margin.horizontal()
            + self.border.horizontal()
            + self.padding.horizontal();
        let vertical = self.margin.vertical()
            + self.border.vertical()
            + self.padding.vertical();
        (horizontal, vertical)
    }
}

impl Default for LayoutBox {
    fn default() -> Self {
        Self::new(Rect::default())
    }
}

impl From<Rect> for LayoutBox {
    fn from(bounds: Rect) -> Self {
        Self::new(bounds)
    }
}

#[cfg(test)]
mod tests {
    use crate::Color;

    use super::*;

    #[test]
    fn test_new() {
        let lb = LayoutBox::from_size(100.0, 80.0);
        assert_eq!(lb.bounds.size.width, 100.0);
        assert_eq!(lb.bounds.size.height, 80.0);
    }

    #[test]
    fn test_border_bounds() {
        let lb = LayoutBox::from_size(100.0, 100.0)
            .with_margin(10.0);

        let bb = lb.border_bounds();
        assert_eq!(bb.origin.x, 10.0);
        assert_eq!(bb.origin.y, 10.0);
        assert_eq!(bb.size.width, 80.0);
        assert_eq!(bb.size.height, 80.0);
    }

    #[test]
    fn test_padding_bounds() {
        let lb = LayoutBox::from_size(100.0, 100.0)
            .with_margin(10.0)
            .with_border(Border::solid(2.0, Color::BLACK));

        let pb = lb.padding_bounds();
        assert_eq!(pb.origin.x, 12.0);  // margin + border
        assert_eq!(pb.origin.y, 12.0);
        assert_eq!(pb.size.width, 76.0);  // 80 - 2*2
        assert_eq!(pb.size.height, 76.0);
    }

    #[test]
    fn test_content_bounds() {
        let lb = LayoutBox::from_size(100.0, 100.0)
            .with_margin(10.0)
            .with_border(Border::solid(2.0, Color::BLACK))
            .with_padding(5.0);

        let cb = lb.content_bounds();
        assert_eq!(cb.origin.x, 17.0);  // margin + border + padding
        assert_eq!(cb.origin.y, 17.0);
        assert_eq!(cb.size.width, 66.0);  // 76 - 2*5
        assert_eq!(cb.size.height, 66.0);
    }

    #[test]
    fn test_total_inset() {
        let lb = LayoutBox::from_size(100.0, 100.0)
            .with_margin(10.0)
            .with_border(Border::solid(2.0, Color::BLACK))
            .with_padding(5.0);

        let (h, v) = lb.total_inset();
        // margin: 20 + border: 4 + padding: 10 = 34
        assert_eq!(h, 34.0);
        assert_eq!(v, 34.0);
    }

    #[test]
    fn test_asymmetric_spacing() {
        let lb = LayoutBox::from_size(100.0, 100.0)
            .with_margin(Margin::new(5.0, 10.0, 15.0, 20.0))
            .with_padding(Padding::symmetric(3.0, 6.0));

        let cb = lb.content_bounds();
        // x: 20 (left margin) + 6 (left padding) = 26
        assert_eq!(cb.origin.x, 26.0);
        // y: 5 (top margin) + 3 (top padding) = 8
        assert_eq!(cb.origin.y, 8.0);
        // width: 100 - 20 - 10 - 6 - 6 = 58
        assert_eq!(cb.size.width, 58.0);
        // height: 100 - 5 - 15 - 3 - 3 = 74
        assert_eq!(cb.size.height, 74.0);
    }
}
