//! Audio Equalizer visualization.
//!
//! Displays a stylized audio visualizer using circles.
//! Each "channel" is represented as a pulsing circle.
//!
//! Run with: cargo run -p osd-flash --example equalizer

#[cfg(target_os = "macos")]
use osd_flash::prelude::*;

#[cfg(target_os = "macos")]
fn main() -> osd_flash::Result<()> {
    println!("Showing audio equalizer...\n");

    let size = 200.0;

    OsdBuilder::new()
        .size(size)
        .position(Position::Center)
        .background(Color::rgba(0.05, 0.05, 0.1, 0.95))
        .corner_radius(20.0)
        // Left channel (low frequencies - green)
        .layer("low", |l| {
            l.circle(50.0)
                .center_offset(-50.0, 0.0)
                .fill(Color::rgba(0.2, 1.0, 0.4, 0.9))
                .animate(Animation::pulse_range(0.8, 1.2))
        })
        // Center channel (mid frequencies - yellow)
        .layer("mid", |l| {
            l.circle(60.0)
                .center()
                .fill(Color::rgba(1.0, 0.8, 0.0, 0.9))
                .animate(Animation::pulse_range(0.85, 1.15))
        })
        // Right channel (high frequencies - red)
        .layer("high", |l| {
            l.circle(45.0)
                .center_offset(50.0, 0.0)
                .fill(Color::rgba(1.0, 0.2, 0.2, 0.9))
                .animate(Animation::pulse_range(0.75, 1.25))
        })
        // Label
        .layer("title", |l| {
            l.text("AUDIO")
                .center_offset(0.0, 70.0)
                .font_size(14.0)
                .text_color(Color::WHITE)
        })
        .show_for(5.seconds())?;

    println!("Done!");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This example only runs on macOS");
}
