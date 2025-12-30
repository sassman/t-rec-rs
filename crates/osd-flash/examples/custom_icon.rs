//! Custom icon example.
//!
//! Shows how to build a custom icon using the IconBuilder API.
//!
//! Run with: cargo run -p osd-flash --example custom_icon

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    let size = 120.0;

    // Build a custom "check" icon (green background with white checkmark circle)
    let icon = IconBuilder::new(size)
        .padding(12.0)
        // Green background
        .background(Color::rgba(0.2, 0.8, 0.3, 0.92), 16.0)
        // White circle in center
        .circle(size / 2.0, size / 2.0, size * 0.25, Color::WHITE)
        // Inner green circle (creates ring effect)
        .circle(
            size / 2.0,
            size / 2.0,
            size * 0.15,
            Color::rgba(0.2, 0.8, 0.3, 1.0),
        )
        .build();

    let config = FlashConfig::new()
        .icon_size(size)
        .position(FlashPosition::Center)
        .duration(2.0);

    println!("Showing custom success icon in center...");

    // Create window using builder
    let mut window = SkylightWindowBuilder::from_config(&config)
        .level(WindowLevel::AboveAll)
        .build()?;

    window.draw(&icon)?;
    window.show(config.duration_secs)?;

    println!("Done!");
    Ok(())
}
