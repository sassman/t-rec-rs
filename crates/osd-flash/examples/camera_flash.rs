//! Simple camera flash example.
//!
//! Demonstrates a camera icon OSD, similar to macOS screenshot feedback.
//!
//! Run with: cargo run -p osd-flash --example camera_flash

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    let size = 120.0;

    println!("Showing camera icon in top-right corner for 3 seconds...");

    // Note: Core Animation uses Y-up coordinates (origin at bottom-left)
    // So positive Y offset moves shapes UP visually
    OsdBuilder::new()
        .size(size)
        .position(Position::TopRight)
        .margin(20.0)
        .background(Color::rgba(0.15, 0.45, 0.9, 0.92))
        .corner_radius(20.0)
        // Camera body (rounded rectangle)
        .layer("body", |l| {
            l.rounded_rect(70.0, 45.0, 8.0).center().fill(Color::WHITE)
        })
        // Viewfinder bump (small rounded rect at top - positive Y)
        .layer("viewfinder", |l| {
            l.rounded_rect(20.0, 10.0, 3.0)
                .center_offset(0.0, 22.0)
                .fill(Color::WHITE)
        })
        // Lens outer ring
        .layer("lens_outer", |l| {
            l.circle(32.0)
                .center()
                .fill(Color::rgba(0.2, 0.3, 0.5, 1.0))
        })
        // Lens inner
        .layer("lens_inner", |l| {
            l.circle(22.0)
                .center()
                .fill(Color::rgba(0.1, 0.15, 0.3, 1.0))
        })
        // Lens highlight (top-left of lens)
        .layer("lens_highlight", |l| {
            l.circle(8.0)
                .center_offset(-4.0, 4.0)
                .fill(Color::rgba(1.0, 1.0, 1.0, 0.4))
        })
        // Flash indicator (top right of camera - positive Y for top)
        .layer("flash", |l| {
            l.circle(10.0)
                .center_offset(22.0, 12.0)
                .fill(Color::rgba(1.0, 0.85, 0.2, 1.0))
        })
        .show_for(3.seconds())?;

    println!("Done!");
    Ok(())
}
