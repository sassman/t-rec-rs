//! Notification badge example.
//!
//! Shows how to create notification-style badges at different screen positions.
//!
//! Run with: cargo run -p skylight-osd --example notification_badge

use skylight_osd::prelude::*;

fn create_badge(text_color: Color, bg_color: Color, size: f64) -> Icon {
    let center = size / 2.0;

    IconBuilder::new(size)
        .padding(8.0)
        // Circular background
        .background(bg_color, size / 2.0 - 4.0)
        // Inner highlight (subtle)
        .circle(center, center - 2.0, size * 0.3, text_color.with_alpha(0.3))
        // Number indicator dot
        .circle(center, center, size * 0.15, text_color)
        .build()
}

fn show_badge_at_position(position: FlashPosition, color: Color, label: &str) -> skylight_osd::Result<()> {
    let size = 80.0;
    let icon = create_badge(Color::WHITE, color, size);

    // Calculate frame based on position
    let config = skylight_osd::FlashConfig::new()
        .icon_size(size)
        .position(position)
        .margin(30.0);

    println!("Showing {} badge...", label);

    let mut window = SkylightWindowBuilder::from_config(&config)
        .level(WindowLevel::AboveAll)
        .build()?;

    window.draw(&icon)?;
    window.show(1.0)?;

    Ok(())
}

fn main() -> skylight_osd::Result<()> {
    // Show badges at different corners with different colors
    show_badge_at_position(FlashPosition::TopRight, Color::rgba(0.9, 0.2, 0.2, 0.95), "red (top-right)")?;
    show_badge_at_position(FlashPosition::TopLeft, Color::rgba(0.2, 0.6, 0.9, 0.95), "blue (top-left)")?;
    show_badge_at_position(FlashPosition::BottomRight, Color::rgba(0.9, 0.6, 0.1, 0.95), "orange (bottom-right)")?;
    show_badge_at_position(FlashPosition::BottomLeft, Color::rgba(0.6, 0.2, 0.8, 0.95), "purple (bottom-left)")?;

    println!("Done!");
    Ok(())
}
