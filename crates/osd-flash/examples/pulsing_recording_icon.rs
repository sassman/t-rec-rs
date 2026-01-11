//! Pulsing recording icon example.
//!
//! Demonstrates a smooth pulsing recording indicator using GPU-accelerated
//! animations via Core Animation. The animation features:
//! - Scale pulsing (breathing effect)
//! - Red glow layer (dramatic visibility)
//!
//! Run with: cargo run -p osd-flash --example pulsing_recording_icon

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    let size = 80.0;
    let margin = 30.0;

    println!("Showing pulsing recording indicator for 10 seconds...");
    println!("Using GPU-accelerated Core Animation.");
    println!();
    println!("Features:");
    println!("  - Scale pulse: 0.9 -> 1.1 (visible breathing effect)");
    println!("  - Red glow layer behind dot");
    println!("  - GPU-accelerated 60 FPS animation");
    println!();

    OsdBuilder::new()
        .size(size)
        .position(Position::TopLeft)
        .margin(margin)
        .background(Color::rgba(0.08, 0.08, 0.08, 0.92))
        .corner_radius(14.0)
        // Outer glow layer (pulses more dramatically)
        .layer("outer_glow", |l| {
            l.circle(50.0)
                .center()
                .fill(Color::rgba(1.0, 0.1, 0.1, 0.25))
                .animate(Animation::pulse_range(0.85, 1.2))
        })
        // Inner glow layer
        .layer("inner_glow", |l| {
            l.circle(38.0)
                .center()
                .fill(Color::rgba(1.0, 0.2, 0.2, 0.4))
                .animate(Animation::pulse_range(0.9, 1.15))
        })
        // Main recording dot
        .layer("dot", |l| {
            l.circle(24.0)
                .center()
                .fill(Color::RED)
                .animate(Animation::pulse())
        })
        // Highlight
        .layer("highlight", |l| {
            l.circle(6.0)
                .center_offset(-4.0, -4.0)
                .fill(Color::rgba(1.0, 0.5, 0.5, 0.5))
        })
        .show_for(10.seconds())?;

    println!("Done!");
    Ok(())
}
