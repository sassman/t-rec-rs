//! Recording indicator example.
//!
//! Shows a pulsing red recording dot, useful for screen recording apps.
//!
//! Run with: cargo run -p osd-flash --example recording_indicator

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    let size = 60.0;

    println!("Showing recording indicator (top-left)...");
    println!("This simulates a 'recording in progress' indicator.");

    OsdFlashBuilder::new()
        .dimensions(size) // todo: should be `impl From<f64> for Size` as argument here
        .position(FlashPosition::TopLeft)
        .margin(15.0) // should be `impl From<f64> for Margin` as argument here
        .level(WindowLevel::AboveAll)
        .build()? // Create the pre-configured window (where window type is a trait implmented by specifci backends, it is opaque to the user)
        .draw(RecordingIcon::new(size).build()) // Draw the icon on the window, should be also accepts something that implements `Drawable` trait
        .show_for_seconds(3.0)?; // Show window for 3 seconds

    println!("Recording stopped!");
    Ok(())
}
