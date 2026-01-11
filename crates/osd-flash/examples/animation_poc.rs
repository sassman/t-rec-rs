//! Animation POC - Testing GPU-accelerated animations.
//!
//! Demonstrates GPU-accelerated Core Animation features:
//! - Scale pulsing with different ranges
//! - Multiple animated layers
//! - Easing functions for natural motion
//!
//! Run with: cargo run -p osd-flash --example animation_poc

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    println!("POC: GPU-accelerated Core Animation");
    println!("===================================");
    println!();
    println!("Testing smooth 60 FPS animations via CABasicAnimation.");
    println!("Watch for smooth pulsing without flickering.");
    println!();

    let size = 200.0;

    println!("Test 1: Basic pulse animation...");
    OsdBuilder::new()
        .size(size)
        .position(Position::Center)
        .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
        .corner_radius(20.0)
        // Outer glow (larger pulse range)
        .layer("outer_glow", |l| {
            l.circle(140.0)
                .center()
                .fill(Color::rgba(0.2, 0.6, 1.0, 0.2))
                .animate(Animation::pulse_range(0.8, 1.2))
        })
        // Inner glow (medium pulse)
        .layer("inner_glow", |l| {
            l.circle(100.0)
                .center()
                .fill(Color::rgba(0.3, 0.7, 1.0, 0.3))
                .animate(Animation::pulse_range(0.85, 1.15))
        })
        // Main circle (subtle pulse)
        .layer("main", |l| {
            l.circle(60.0)
                .center()
                .fill(Color::rgba(0.4, 0.8, 1.0, 1.0))
                .animate(Animation::pulse())
        })
        // Highlight (no animation - static reference point)
        .layer("highlight", |l| {
            l.circle(15.0)
                .center_offset(-10.0, -10.0)
                .fill(Color::rgba(1.0, 1.0, 1.0, 0.5))
        })
        .show_for(5.seconds())?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    println!("Test 2: Concentric rings animation...");
    OsdBuilder::new()
        .size(size)
        .position(Position::Center)
        .background(Color::rgba(0.05, 0.05, 0.1, 0.95))
        .corner_radius(size / 2.0)
        // Ring 1 (outermost)
        .layer("ring1", |l| {
            l.circle(180.0)
                .center()
                .fill(Color::rgba(1.0, 0.3, 0.5, 0.15))
                .animate(Animation::pulse_range(0.9, 1.1))
        })
        // Ring 2
        .layer("ring2", |l| {
            l.circle(140.0)
                .center()
                .fill(Color::rgba(1.0, 0.4, 0.6, 0.2))
                .animate(Animation::pulse_range(0.92, 1.08))
        })
        // Ring 3
        .layer("ring3", |l| {
            l.circle(100.0)
                .center()
                .fill(Color::rgba(1.0, 0.5, 0.7, 0.25))
                .animate(Animation::pulse_range(0.94, 1.06))
        })
        // Ring 4 (innermost)
        .layer("ring4", |l| {
            l.circle(60.0)
                .center()
                .fill(Color::rgba(1.0, 0.6, 0.8, 0.3))
                .animate(Animation::pulse_range(0.96, 1.04))
        })
        // Center dot
        .layer("center", |l| {
            l.circle(30.0)
                .center()
                .fill(Color::rgba(1.0, 0.8, 0.9, 1.0))
        })
        .show_for(5.seconds())?;

    println!();
    println!("Test complete!");
    println!();
    println!("Results:");
    println!("  - Animations should have been smooth (60 FPS)");
    println!("  - No flickering or tearing visible");
    println!("  - GPU-accelerated via Core Animation");
    println!();

    Ok(())
}
