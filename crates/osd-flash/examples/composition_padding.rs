//! Composition example - layered drawing.
//!
//! Demonstrates drawing multiple elements (shapes, icons, text) onto a single window.
//!
//! Run with: cargo run -p osd-flash --example composition_padding

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    println!("Showing composition with layered drawing...\n");

    // Example 1: Recording indicator with styled window
    // Window: 200x120, padding: 40px horizontal, content area: 120x120
    // Background and corner radius are set on the window itself.
    // Content is drawn in content coordinates (offset by padding automatically).
    println!("1. Recording indicator with styled window");
    OsdFlashBuilder::new()
        .dimensions(Size::new(200.0, 120.0)) // Full window size
        .position(FlashPosition::TopRight)
        .margin(Margin::all(20.0))
        .padding(Padding::symmetric(0.0, 20.0)) // 40px left/right padding
        .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
        .corner_radius(16.0)
        .build()?
        // Red recording dot (center at x=20 so left edge is at content boundary)
        .draw(StyledShape::new(
            Shape::circle_at(20.0, 60.0, 20.0),
            Color::RED,
        ))
        // REC text (positioned to the right of the dot)
        .draw(StyledText::at("REC", 50.0, 50.0, 24.0, Color::WHITE))
        .show_for_seconds(2.0)?;

    std::thread::sleep(std::time::Duration::from_millis(1500));

    println!("\nDone!");
    Ok(())
}
