//! Recording indicator example.
//!
//! Shows a pulsing red recording dot, useful for screen recording apps.
//!
//! Run with: cargo run -p osd-flash --example recording_indicator

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    let size = 80.0;
    let margin = 30.0;

    println!("Showing recording indicator (top-left)...");
    println!("This simulates a 'recording in progress' indicator.");

    OsdFlashBuilder::new()
        .dimensions(size)
        .position(FlashPosition::TopLeft)
        .margin(margin)
        .level(WindowLevel::AboveAll)
        .build()?
        .draw(RecordingIcon::new(size).build())
        .show_for_seconds(3.0)?;

    println!("Recording stopped!");
    Ok(())
}
