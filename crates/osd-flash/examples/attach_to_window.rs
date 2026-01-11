//! Attach to window example.
//!
//! Demonstrates attaching the OSD flash to a specific window.
//! The indicator will appear relative to the target window's bounds.
//!
//! Run with: cargo run -p osd-flash --example attach_to_window
//!
//! You can pass a window ID as argument:
//!   cargo run -p osd-flash --example attach_to_window -- 12345

use osd_flash::prelude::*;
use std::env;

fn main() -> osd_flash::Result<()> {
    // Get window ID from command line or use 0 (main display fallback)
    let win_id: u64 = env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| {
            // Try to get current terminal window from WINDOWID env var
            env::var("WINDOWID")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0)
        });

    println!("Target window ID: {}", win_id);
    println!("(Use WINDOWID env var or pass window ID as argument)");
    println!();
    println!("NOTE: attach_to_window() is not yet implemented.");
    println!("Showing at screen positions instead.\n");

    // Show recording indicator at top-right
    // TODO: When attach_to_window is implemented, use:
    // .attach_to_window(win_id)
    println!("Recording indicator (top-right)...");
    OsdBuilder::new()
        .size(100.0)
        .position(Position::TopRight)
        .margin(30.0)
        .level(WindowLevel::AboveAll)
        // .attach_to_window(win_id)  // Future: attach relative to window
        .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
        .corner_radius(14.0)
        .layer("glow", |l| {
            l.circle(60.0)
                .center()
                .fill(Color::rgba(1.0, 0.2, 0.2, 0.3))
                .animate(Animation::pulse_range(0.9, 1.15))
        })
        .layer("dot", |l| {
            l.circle(40.0)
                .center()
                .fill(Color::RED)
                .animate(Animation::pulse())
        })
        .show_for(2.seconds())?;

    std::thread::sleep(std::time::Duration::from_millis(300));

    // Show camera indicator at top-left
    // TODO: When attach_to_window is implemented, use:
    // .attach_to_window(win_id)
    println!("Camera indicator (top-left)...");
    OsdBuilder::new()
        .size(100.0)
        .position(Position::TopLeft)
        .margin(30.0)
        .level(WindowLevel::AboveAll)
        // .attach_to_window(win_id)  // Future: attach relative to window
        .background(Color::rgba(0.15, 0.45, 0.9, 0.92))
        .corner_radius(14.0)
        .layer("body", |l| {
            l.ellipse(60.0, 45.0).center().fill(Color::WHITE)
        })
        .layer("lens", |l| {
            l.circle(28.0)
                .center()
                .fill(Color::rgba(0.3, 0.5, 0.8, 1.0))
        })
        .layer("lens_center", |l| {
            l.circle(12.0)
                .center()
                .fill(Color::rgba(0.1, 0.2, 0.4, 1.0))
        })
        .show_for(2.seconds())?;

    println!("Done!");
    Ok(())
}
