//! Recording indicator example.
//!
//! Shows a pulsing red recording dot, useful for screen recording apps.
//!
//! Run with: cargo run -p osd-flash --example recording_indicator_pulse

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    let size = 80.0;
    let center = size / 2.0;
    let margin = 30.0;

    println!("Showing recording indicator that pulses (top-left)...");

    // Recording dot with highlight
    let icon = IconBuilder::new(size)
        .circle(
            center,
            center,
            size * 0.18,
            Color::rgba(0.95, 0.12, 0.12, 1.0),
        )
        .circle(
            center - size * 0.05,
            center - size * 0.05,
            size * 0.04,
            Color::rgba(1.0, 0.5, 0.5, 0.5),
        )
        .build();

    OsdFlashBuilder::new()
        .level(WindowLevel::AboveAll)
        .position(FlashPosition::TopLeft)
        .dimensions(size)
        .margin(margin)
        .corner_radius(14.0)
        .background(Color::rgba(0.1, 0.1, 0.1, 0.88))
        .build()?
        .draw(icon)
        .animate("pulse", 2.seconds())
        .easing(Easing::EaseInOut)
        .keyframe(0.0, |k| {
            k.scale(0.9)
                .circle(center, center, size * 0.26, Color::rgba(1.0, 0.2, 0.2, 0.4))
        })
        .keyframe(0.5, |k| {
            k.scale(1.2)
                .circle(center, center, size * 0.26, Color::rgba(1.0, 0.2, 0.2, 0.5))
        })
        .keyframe(1.0, |k| {
            k.scale(0.9)
                .circle(center, center, size * 0.26, Color::rgba(1.0, 0.2, 0.2, 0.2))
        })
        .show(10.0.seconds())?;

    println!("Recording stopped!");
    Ok(())
}
