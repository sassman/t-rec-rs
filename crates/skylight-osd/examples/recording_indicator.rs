//! Recording indicator example.
//!
//! Shows a pulsing red recording dot, useful for screen recording apps.
//!
//! Run with: cargo run -p skylight-osd --example recording_indicator

use skylight_osd::prelude::*;

fn recording_icon(size: f64) -> Icon {
    let center = size / 2.0;

    IconBuilder::new(size)
        .padding(10.0)
        // Dark semi-transparent background
        .background(Color::rgba(0.1, 0.1, 0.1, 0.85), 14.0)
        // Outer red glow
        .circle(center, center, size * 0.28, Color::rgba(1.0, 0.2, 0.2, 0.4))
        // Main red recording dot
        .circle(center, center, size * 0.2, Color::rgba(1.0, 0.15, 0.15, 1.0))
        // Highlight
        .circle(
            center - size * 0.06,
            center - size * 0.06,
            size * 0.06,
            Color::rgba(1.0, 0.5, 0.5, 0.6),
        )
        .build()
}

fn main() -> skylight_osd::Result<()> {
    let size = 60.0;
    let icon = recording_icon(size);

    let config = skylight_osd::FlashConfig::new()
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
