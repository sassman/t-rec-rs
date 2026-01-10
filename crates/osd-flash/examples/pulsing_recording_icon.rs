//! Pulsing recording icon example.
//!
//! Demonstrates a smooth pulsing recording indicator using the high-level
//! `PulsingRecordingIcon` API. The animation uses a software animation loop
//! with CALayer-based rendering for smooth visual effects including:
//! - Scale pulsing (breathing effect)
//! - Red glow/shadow pulsing (dramatic visibility)
//!
//! Run with: cargo run -p osd-flash --example pulsing_recording_icon

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    let size = 80.0;
    let margin = 30.0;

    println!("Showing pulsing recording indicator...");
    println!("This uses a software animation loop with CALayer rendering.");
    println!();
    println!("Features:");
    println!("  - Scale pulse: 0.85 -> 1.15 (visible breathing effect)");
    println!("  - Red glow ring: actual shape layer behind dot, opacity 0.4 -> 1.0");
    println!("  - Glow ring is CLEARLY visible (not shadow-based)");
    println!("  - Smooth 60 FPS software animation");
    println!();

    // Create the pulsing recording icon with defaults
    // The defaults are now designed for high visibility:
    //   - Scale: 0.85 to 1.15 (pronounced breathing)
    //   - Glow ring: 0.4 to 1.0 opacity (actual circle layer, not shadow)
    //   - Glow ring radius: ~28% of icon size (larger than the dot)

    let window = OsdFlashBuilder::new()
        .level(WindowLevel::AboveAll)
        .position(FlashPosition::TopLeft)
        .dimensions(size)
        .margin(margin)
        .corner_radius(14.0)
        .background(Color::rgba(0.08, 0.08, 0.08, 0.92))
        .container(PulsingRecordingIcon::new(size))
        .build()?;

    window.show_for(10.seconds());

    Ok(())
}
