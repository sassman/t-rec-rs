//! Recording indicator example.
//!
//! Shows a pulsing red recording dot, useful for screen recording apps.
//!
//! Run with: cargo run -p osd-flash --example recording_indicator

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    let size = 60.0;

    // Use the pre-built RecordingIcon from the library
    let icon = RecordingIcon::new(size).build();

    let config = osd_flash::FlashConfig::new()
        .icon_size(size)
        .position(FlashPosition::TopLeft)
        .margin(15.0);

    println!("Showing recording indicator (top-left)...");
    println!("This simulates a 'recording in progress' indicator.");

    let mut window = SkylightWindowBuilder::from_config(&config)
        .level(WindowLevel::AboveAll)
        .build()?;

    window.draw(&icon)?;
    window.show(3.0)?; // Show for 3 seconds

    println!("Recording stopped!");
    Ok(())
}
