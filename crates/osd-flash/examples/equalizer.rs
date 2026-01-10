//! Audio Equalizer visualization.
//!
//! Displays a stylized audio equalizer with colored bars.
//!
//! Run with: cargo run -p osd-flash --example equalizer

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    println!("Showing audio equalizer...\n");

    // Bar heights (simulating audio levels)
    let bar_heights = [0.3, 0.6, 0.9, 0.7, 1.0, 0.8, 0.5, 0.7, 0.4, 0.6, 0.8, 0.3];
    let num_bars = bar_heights.len();
    let bar_width = 12.0;
    let bar_gap = 6.0;
    let max_height = 180.0;
    let total_width = (bar_width + bar_gap) * num_bars as f64 - bar_gap;
    let padding = 30.0;

    // Collect all shapes
    let mut shapes: Vec<StyledShape> = Vec::new();

    // Draw each bar with gradient-like coloring (bottom to top: green -> yellow -> red)
    for (i, &height_ratio) in bar_heights.iter().enumerate() {
        let x = i as f64 * (bar_width + bar_gap);
        let bar_height = height_ratio * max_height;
        let y = max_height - bar_height;

        // Color based on height (green at bottom, red at top)
        let color = if height_ratio > 0.8 {
            Color::rgba(1.0, 0.2, 0.2, 1.0) // Red for high
        } else if height_ratio > 0.5 {
            Color::rgba(1.0, 0.8, 0.0, 1.0) // Yellow/orange for mid
        } else {
            Color::rgba(0.2, 1.0, 0.4, 1.0) // Green for low
        };

        shapes.push(StyledShape::new(
            Shape::rounded_rect(Rect::from_xywh(x, y, bar_width, bar_height), 4.0),
            color,
        ));
    }

    OsdFlashBuilder::new()
        .dimensions(Size::new(
            total_width + padding * 2.0,
            max_height + padding * 2.0,
        ))
        .position(FlashPosition::Center)
        .background(Color::rgba(0.05, 0.05, 0.1, 0.95))
        .corner_radius(20.0)
        .padding(Padding::all(padding))
        .build()?
        .draw(shapes)
        // Add title
        .draw(StyledText::at(
            "AUDIO",
            total_width / 2.0 - 30.0,
            max_height + 5.0,
            14.0,
            Color::WHITE,
        ))
        .show_for_seconds(4.0)?;

    println!("Done!");
    Ok(())
}
