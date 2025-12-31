//! Composition example - layered drawing.
//!
//! Demonstrates drawing multiple elements (shapes, icons, text) onto a single window.
//!
//! Run with: cargo run -p osd-flash --example composition

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    println!("Showing composition with layered drawing...\n");

    // Example 1: Recording indicator with text
    println!("1. Recording indicator with 'REC' text");
    OsdFlashBuilder::new()
        .dimensions(120.0)
        .position(FlashPosition::TopRight)
        .margin(20.0)
        .build()?
        // First layer: dark background
        .draw(Shape::rounded_rect(
            Rect::from_xywh(0.0, 0.0, 120.0, 120.0),
            16.0,
            Color::rgba(0.1, 0.1, 0.1, 0.9),
        ))
        // Second layer: red recording dot
        .draw(Shape::circle_at(40.0, 60.0, 20.0, Color::RED))
        // Third layer: text
        .draw(Shape::text_at("REC", 68.0, 52.0, 24.0, Color::WHITE))
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
        .draw(Shape::rounded_rect(
            Rect::from_xywh(0.0, 0.0, 140.0, 140.0),
            20.0,
            Color::rgba(0.0, 0.0, 0.0, 0.85),
        ))
        // Camera icon
        .draw(CameraIcon::new(80.0).padding(30.0).build())
        // Status text
        .draw(Shape::text_at("Ready", 45.0, 115.0, 18.0, Color::WHITE))
        .show_for_seconds(2.0)?;

    std::thread::sleep(std::time::Duration::from_millis(300));

    // Example 3: Multiple shapes composition
    println!("3. Traffic light style indicator");
    OsdFlashBuilder::new()
        .dimensions(Size::new(60.0, 140.0))
        .position(FlashPosition::Center)
        .build()?
        // Background
        .draw(Shape::rounded_rect(
            Rect::from_xywh(0.0, 0.0, 60.0, 140.0),
            12.0,
            Color::rgba(0.2, 0.2, 0.2, 0.95),
        ))
        // Red light (dim)
        .draw(Shape::circle_at(30.0, 30.0, 18.0, Color::rgba(0.5, 0.0, 0.0, 1.0)))
        // Yellow light (dim)
        .draw(Shape::circle_at(30.0, 70.0, 18.0, Color::rgba(0.5, 0.5, 0.0, 1.0)))
        // Green light (bright - active)
        .draw(Shape::circle_at(30.0, 110.0, 18.0, Color::GREEN))
        .show_for_seconds(2.0)?;

    println!("\nDone!");
    Ok(())
}
