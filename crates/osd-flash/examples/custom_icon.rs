//! Custom icon example.
//!
//! Shows how to build a custom icon using layered composition.
//!
//! Run with: cargo run -p osd-flash --example custom_icon

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    let size = 120.0;

    println!("Showing custom success icon in center for 3 seconds...");

    // Build a custom "check" icon (green background with white checkmark circle)
    OsdBuilder::new()
        .size(size)
        .position(Position::Center)
        .background(Color::rgba(0.2, 0.8, 0.3, 0.92))
        .corner_radius(16.0)
        // White circle ring (outer)
        .layer("ring_outer", |l| {
            l.circle(size * 0.5)
                .center()
                .fill(Color::WHITE)
        })
        // Inner green circle (creates ring effect)
        .layer("ring_inner", |l| {
            l.circle(size * 0.3)
                .center()
                .fill(Color::rgba(0.2, 0.8, 0.3, 1.0))
        })
        .show_for(3.seconds())?;

    println!("Done!");
    Ok(())
}
