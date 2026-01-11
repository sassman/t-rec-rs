//! Recording indicator example.
//!
//! Shows a pulsing red recording dot, useful for screen recording apps.
//!
//! Run with: cargo run -p osd-flash --example recording_indicator

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    let size = 80.0;
    let margin = 30.0;

    println!("Showing recording indicator (top-left) for 5 seconds...");
    println!("This simulates a 'recording in progress' indicator.");

    OsdBuilder::new()
        .size(size)
        .position(Position::TopLeft)
        .margin(margin)
        .background(Color::rgba(0.1, 0.1, 0.1, 0.88))
        .corner_radius(14.0)
        // Glow layer (pulses larger)
        .layer("glow", |l| {
            l.circle(44.0)
                .center()
                .fill(Color::rgba(1.0, 0.2, 0.2, 0.35))
                .animate(Animation::pulse_range(0.9, 1.15))
        })
        // Main recording dot
        .layer("dot", |l| {
            l.circle(28.0)
                .center()
                .fill(Color::RED)
                .animate(Animation::pulse())
        })
        // Highlight
        .layer("highlight", |l| {
            l.circle(8.0)
                .center_offset(-5.0, -5.0)
                .fill(Color::rgba(1.0, 0.5, 0.5, 0.5))
        })
        .show_for(5.seconds())?;

    println!("Recording stopped!");
    Ok(())
}
