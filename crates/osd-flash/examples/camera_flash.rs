//! Simple camera flash example.
//!
//! Demonstrates the built-in camera icon flash, similar to macOS screenshot feedback.
//!
//! Run with: cargo run -p osd-flash --example camera_flash

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    let size = 120.0;

    println!("Showing camera flash in top-right corner...");

    OsdFlashBuilder::new()
        .dimensions(size)
        .position(FlashPosition::TopRight)
        .margin(20.0)
        .level(WindowLevel::AboveAll)
        .build()?
        .draw(CameraIcon::new(size).build())
        .show_for_seconds(1.5)?;

    println!("Done!");
    Ok(())
}
