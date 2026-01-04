//! Composition example - layered drawing.
//!
//! Demonstrates drawing multiple elements (shapes, icons, text) onto a single window.
//!
//! Run with: cargo run -p osd-flash --example composition

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    println!("Showing composition with layered drawing...\n");

    // Example 1: Recording indicator with shape-level padding
    // The background shape is 120x120, but expanded by 40px horizontal padding
    // Result: 200x120 background with content centered
    println!("1. Recording indicator with padded background");
    OsdFlashBuilder::new()
        .dimensions(Size::new(200.0, 120.0)) // Full size including padding
        .position(FlashPosition::TopRight)
        .margin(30.0)
        // All drawn elements will be expanded by this padding, so they have space around them, but are visible in this space, like the background color fo the first drawn shape.
        .padding(Padding::symmetric(0.0, 40.0))
        .build()?
        // Dark background with padding - expands from 120x120 to 200x120
        .draw(StyledShape::new(
            Shape::rounded_rect(Rect::from_xywh(40.0, 0.0, 120.0, 120.0), 16.0),
            Color::rgba(0.1, 0.1, 0.1, 0.9),
        ))
        // Red recording dot (positioned in center of content area)
        .draw(StyledShape::new(
            Shape::circle_at(80.0, 60.0, 20.0),
            Color::RED,
        ))
        // REC text
        .draw(StyledText::at("REC", 108.0, 52.0, 24.0, Color::WHITE))
        .show_for_seconds(2.0)?;

    std::thread::sleep(std::time::Duration::from_millis(300));

    // Example 2: Status badge with icon and text
    println!("2. Camera icon with status text");
    OsdFlashBuilder::new()
        .dimensions(140.0)
        .position(FlashPosition::TopLeft)
        .margin(20.0)
        .build()?
        // Background
        .draw(StyledShape::new(
            Shape::rounded_rect(Rect::from_xywh(0.0, 0.0, 140.0, 140.0), 20.0),
            Color::rgba(0.0, 0.0, 0.0, 0.85),
        ))
        // Camera icon
        .draw(CameraIcon::new(80.0).padding(30.0).build())
        // Status text
        .draw(StyledText::at("Ready", 45.0, 115.0, 18.0, Color::WHITE))
        .show_for_seconds(2.0)?;

    std::thread::sleep(std::time::Duration::from_millis(300));

    // Example 3: Multiple shapes composition
    println!("3. Traffic light style indicator");
    OsdFlashBuilder::new()
        .dimensions(Size::new(60.0, 140.0))
        .position(FlashPosition::Center)
        .build()?
        // Background
        .draw(StyledShape::new(
            Shape::rounded_rect(Rect::from_xywh(0.0, 0.0, 60.0, 140.0), 12.0),
            Color::rgba(0.2, 0.2, 0.2, 0.95),
        ))
        // Red light (dim)
        .draw(StyledShape::new(
            Shape::circle_at(30.0, 30.0, 18.0),
            Color::rgba(0.5, 0.0, 0.0, 1.0),
        ))
        // Yellow light (dim)
        .draw(StyledShape::new(
            Shape::circle_at(30.0, 70.0, 18.0),
            Color::rgba(0.5, 0.5, 0.0, 1.0),
        ))
        // Green light (bright - active)
        .draw(StyledShape::new(
            Shape::circle_at(30.0, 110.0, 18.0),
            Color::GREEN,
        ))
        .show_for_seconds(2.0)?;

    println!("\nDone!");
    Ok(())
}
