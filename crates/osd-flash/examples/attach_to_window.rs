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

    println!("Attaching OSD flash to window ID: {}", win_id);
    println!("(Use WINDOWID env var or pass window ID as argument)");

    // Show camera icon attached to the window
    println!("\nShowing camera icon (top-right of window)...");
    OsdFlashBuilder::new()
        .dimensions(100.0)
        .position(FlashPosition::TopRight)
        .margin(30.0)
        .level(WindowLevel::AboveAll)
        .attach_to_window(win_id)
        .build()?
        .draw(CameraIcon::new(100.0).build())
        .show_for_seconds(2.0)?;

    // Brief pause between indicators
    std::thread::sleep(std::time::Duration::from_millis(300));

    // Show recording icon at a different position
    println!("Showing recording icon (top-left of window)...");
    OsdFlashBuilder::new()
        .dimensions(80.0)
        .position(FlashPosition::TopLeft)
        .margin(30.0)
        .level(WindowLevel::AboveAll)
        .attach_to_window(win_id)
        .build()?
        .draw(RecordingIcon::new(80.0).build())
        .show_for_seconds(2.0)?;

    println!("Done!");
    Ok(())
}
