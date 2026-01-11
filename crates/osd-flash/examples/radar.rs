//! Radar / Sonar display visualization.
//!
//! Displays concentric circles like a radar sweep with detected blips.
//!
//! Run with: cargo run -p osd-flash --example radar

#[cfg(target_os = "macos")]
use osd_flash::prelude::*;

#[cfg(target_os = "macos")]
fn main() -> osd_flash::Result<()> {
    println!("Showing radar display...\n");

    let size = 250.0;

    OsdBuilder::new()
        .size(size)
        .position(Position::Center)
        .background(Color::rgba(0.0, 0.05, 0.0, 0.95))
        .corner_radius(size / 2.0) // Circular window
        // Outer ring
        .layer("ring4", |l| {
            l.circle(220.0)
                .center()
                .fill(Color::rgba(0.0, 0.4, 0.0, 0.3))
        })
        // Ring 3
        .layer("ring3", |l| {
            l.circle(170.0)
                .center()
                .fill(Color::rgba(0.0, 0.05, 0.0, 1.0))
        })
        // Ring 2
        .layer("ring2", |l| {
            l.circle(160.0)
                .center()
                .fill(Color::rgba(0.0, 0.4, 0.0, 0.3))
        })
        // Ring 1
        .layer("ring1", |l| {
            l.circle(110.0)
                .center()
                .fill(Color::rgba(0.0, 0.05, 0.0, 1.0))
        })
        // Inner ring
        .layer("ring0", |l| {
            l.circle(100.0)
                .center()
                .fill(Color::rgba(0.0, 0.4, 0.0, 0.3))
        })
        // Center background
        .layer("center_bg", |l| {
            l.circle(50.0)
                .center()
                .fill(Color::rgba(0.0, 0.05, 0.0, 1.0))
        })
        // Center dot
        .layer("center", |l| {
            l.circle(10.0)
                .center()
                .fill(Color::rgba(0.0, 1.0, 0.0, 1.0))
        })
        // Blip 1 (pulsing)
        .layer("blip1_glow", |l| {
            l.circle(18.0)
                .center_offset(50.0, -40.0)
                .fill(Color::rgba(0.0, 1.0, 0.0, 0.3))
                .animate(Animation::pulse_range(0.8, 1.3))
        })
        .layer("blip1", |l| {
            l.circle(10.0)
                .center_offset(50.0, -40.0)
                .fill(Color::rgba(0.3, 1.0, 0.3, 1.0))
        })
        // Blip 2
        .layer("blip2_glow", |l| {
            l.circle(14.0)
                .center_offset(-60.0, 30.0)
                .fill(Color::rgba(0.0, 1.0, 0.0, 0.3))
                .animate(Animation::pulse_range(0.85, 1.25))
        })
        .layer("blip2", |l| {
            l.circle(8.0)
                .center_offset(-60.0, 30.0)
                .fill(Color::rgba(0.3, 1.0, 0.3, 1.0))
        })
        // Blip 3
        .layer("blip3_glow", |l| {
            l.circle(20.0)
                .center_offset(20.0, 70.0)
                .fill(Color::rgba(0.0, 1.0, 0.0, 0.3))
                .animate(Animation::pulse_range(0.9, 1.2))
        })
        .layer("blip3", |l| {
            l.circle(12.0)
                .center_offset(20.0, 70.0)
                .fill(Color::rgba(0.3, 1.0, 0.3, 1.0))
        })
        // Title
        .layer("title", |l| {
            l.text("RADAR")
                .center_offset(0.0, -100.0)
                .font_size(14.0)
                .text_color(Color::rgba(0.0, 0.8, 0.0, 1.0))
        })
        .show_for(5.seconds())?;

    println!("Done!");
    Ok(())
}


#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This example only runs on macOS");
}
