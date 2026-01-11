//! Notification badge example.
//!
//! Shows how to create notification-style badges at different screen positions.
//!
//! Run with: cargo run -p osd-flash --example notification_badge

use osd_flash::prelude::*;

fn show_badge_at_position(
    position: Position,
    color: Color,
    label: &str,
) -> osd_flash::Result<()> {
    let size = 80.0;

    println!("Showing {} badge...", label);

    OsdBuilder::new()
        .size(size)
        .position(position)
        .margin(30.0)
        .level(WindowLevel::AboveAll)
        .background(color)
        .corner_radius(size / 2.0 - 4.0)
        // Inner highlight
        .layer("highlight", |l| {
            l.circle(size * 0.6)
                .center_offset(0.0, -2.0)
                .fill(Color::WHITE.with_alpha(0.3))
        })
        // Center dot indicator
        .layer("dot", |l| {
            l.circle(size * 0.3)
                .center()
                .fill(Color::WHITE)
        })
        .show_for(1.seconds())?;

    Ok(())
}

fn main() -> osd_flash::Result<()> {
    // Show badges at different corners with different colors
    show_badge_at_position(
        Position::TopRight,
        Color::rgba(0.9, 0.2, 0.2, 0.95),
        "red (top-right)",
    )?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    show_badge_at_position(
        Position::TopLeft,
        Color::rgba(0.2, 0.6, 0.9, 0.95),
        "blue (top-left)",
    )?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    show_badge_at_position(
        Position::BottomRight,
        Color::rgba(0.9, 0.6, 0.1, 0.95),
        "orange (bottom-right)",
    )?;

    std::thread::sleep(std::time::Duration::from_millis(200));

    show_badge_at_position(
        Position::BottomLeft,
        Color::rgba(0.6, 0.2, 0.8, 0.95),
        "purple (bottom-left)",
    )?;

    println!("Done!");
    Ok(())
}
