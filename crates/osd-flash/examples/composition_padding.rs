//! Composition with padding example.
//!
//! Demonstrates a recording indicator with text laid out horizontally.
//!
//! Run with: cargo run -p osd-flash --example composition_padding

#[cfg(target_os = "macos")]
use osd_flash::prelude::*;

#[cfg(target_os = "macos")]
fn main() -> osd_flash::Result<()> {
    println!("Showing composition with layered drawing...\n");

    // Recording indicator with styled window
    // Demonstrates using layers with offset positioning
    println!("Recording indicator with styled window");
    OsdBuilder::new()
        .dimensions(180.0, 100.0)
        .position(Position::TopRight)
        .margin(20.0)
        .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
        .corner_radius(16.0)
        // Red recording dot (offset left from center)
        .layer("dot", |l| {
            l.circle(40.0)
                .center_offset(-45.0, 0.0)
                .fill(Color::RED)
                .animate(Animation::pulse())
        })
        // REC text (offset right from center)
        .layer("text", |l| {
            l.text("REC")
                .center_offset(25.0, 0.0)
                .font_size(28.0)
                .font_weight(FontWeight::Bold)
                .text_color(Color::WHITE)
        })
        .show_for(3.seconds())?;

    println!("\nDone!");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This example only runs on macOS");
}
