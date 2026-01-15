//! Composition example - layered drawing.
//!
//! Demonstrates drawing multiple elements using layer composition.
//!
//! Run with: cargo run -p osd-flash --example composition

#[cfg(target_os = "macos")]
use osd_flash::prelude::*;

#[cfg(target_os = "macos")]
fn main() -> osd_flash::Result<()> {
    println!("Showing composition examples...\n");

    // Example 1: Recording indicator with text
    println!("1. Recording indicator");
    OsdBuilder::new()
        .dimensions(160.0, 80.0)
        .position(Position::TopRight)
        .margin(30.0)
        .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
        .corner_radius(16.0)
        // Recording dot
        .layer("dot", |l| {
            l.circle(32.0)
                .center_offset(-40.0, 0.0)
                .fill(Color::RED)
                .animate(Animation::pulse())
        })
        // REC text
        .layer("text", |l| {
            l.text("REC")
                .center_offset(20.0, 0.0)
                .font_size(24.0)
                .font_weight(FontWeight::Bold)
                .text_color(Color::WHITE)
        })
        .show_for(3.seconds())?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Example 2: Traffic light style indicator
    println!("2. Traffic light indicator");
    OsdBuilder::new()
        .dimensions(60.0, 140.0)
        .position(Position::Center)
        .background(Color::rgba(0.2, 0.2, 0.2, 0.95))
        .corner_radius(12.0)
        // Red light (dim)
        .layer("red", |l| {
            l.circle(30.0)
                .center_offset(0.0, -40.0)
                .fill(Color::rgba(0.5, 0.0, 0.0, 1.0))
        })
        // Yellow light (dim)
        .layer("yellow", |l| {
            l.circle(30.0)
                .center()
                .fill(Color::rgba(0.5, 0.5, 0.0, 1.0))
        })
        // Green light (bright - active, with pulse)
        .layer("green", |l| {
            l.circle(30.0)
                .center_offset(0.0, 40.0)
                .fill(Color::GREEN)
                .animate(Animation::pulse_range(0.95, 1.05))
        })
        .show_for(3.seconds())?;

    println!("\nDone!");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This example only runs on macOS");
}
