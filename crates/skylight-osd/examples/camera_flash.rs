//! Simple camera flash example.
//!
//! Demonstrates the built-in camera icon flash, similar to macOS screenshot feedback.
//!
//! Run with: cargo run -p skylight-osd --example camera_flash

use skylight_osd::prelude::*;

fn main() {
    // Use the built-in camera icon with default settings
    let config = FlashConfig::new()
        .icon_size(120.0)
        .position(FlashPosition::TopRight)
        .duration(1.5)
        .margin(20.0);

    println!("Showing camera flash in top-right corner...");

    // flash_screenshot requires a window ID; use 0 for main display
    skylight_osd::flash_screenshot(&config, 0);

    println!("Done!");
}
