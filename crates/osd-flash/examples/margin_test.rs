//! Test margin symmetry between TopLeft and TopRight.
//!
//! This example shows two identical windows simultaneously to compare margins.
//!
//! Run with: cargo run -p osd-flash --example margin_test

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    println!("=== Margin Symmetry Test ===\n");
    println!("Showing TopLeft and TopRight windows simultaneously.");
    println!("Both should have equal margin from screen edges.\n");

    // Use identical settings for both windows
    let size = Size::new(100.0, 100.0);
    let margin = 30.0;
    let corner_radius = 12.0;

    // Create TopLeft window
    println!("Creating TopLeft window...");
    let top_left = OsdFlashBuilder::new()
        .dimensions(size)
        .position(FlashPosition::TopLeft)
        .margin(margin)
        .background(Color::rgba(0.2, 0.8, 0.2, 0.95)) // Green
        .corner_radius(corner_radius)
        .build()?;

    // Create TopRight window
    println!("Creating TopRight window...");
    let top_right = OsdFlashBuilder::new()
        .dimensions(size)
        .position(FlashPosition::TopRight)
        .margin(margin)
        .background(Color::rgba(0.8, 0.2, 0.2, 0.95)) // Red
        .corner_radius(corner_radius)
        .build()?;

    // Show both windows
    top_left.show_window()?;
    top_right.show_window()?;

    println!("\nBoth windows visible. Check if margins are equal.");
    println!("Green = TopLeft, Red = TopRight");
    println!("Waiting 5 seconds...\n");

    // Keep running the event loop to display windows
    #[link(name = "CoreFoundation", kind = "framework")]
    extern "C" {
        fn CFRunLoopRunInMode(
            mode: *const std::ffi::c_void,
            seconds: f64,
            return_after_source_handled: bool,
        ) -> i32;
        static kCFRunLoopDefaultMode: *const std::ffi::c_void;
    }

    unsafe {
        CFRunLoopRunInMode(kCFRunLoopDefaultMode, 5.0, false);
    }

    top_left.hide_window()?;
    top_right.hide_window()?;

    println!("Done!");
    Ok(())
}
