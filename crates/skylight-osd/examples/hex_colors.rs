//! Hex colors example.
//!
//! Demonstrates using hex color codes to create branded icons.
//!
//! Run with: cargo run -p skylight-osd --example hex_colors

use skylight_osd::prelude::*;

fn branded_icon(size: f64, primary: &str, secondary: &str) -> Icon {
    let primary_color = Color::from_hex(primary).unwrap_or(Color::BLUE);
    let secondary_color = Color::from_hex(secondary).unwrap_or(Color::WHITE);
    let center = size / 2.0;

    IconBuilder::new(size)
        .padding(12.0)
        .background(primary_color.with_alpha(0.95), 18.0)
        // Decorative circles
        .circle(center - 15.0, center - 10.0, 12.0, secondary_color.with_alpha(0.9))
        .circle(center + 15.0, center - 10.0, 12.0, secondary_color.with_alpha(0.9))
        .circle(center, center + 12.0, 14.0, secondary_color.with_alpha(0.9))
        .build()
}

fn main() -> skylight_osd::Result<()> {
    let size = 100.0;

    // GitHub colors
    println!("Showing GitHub-themed icon...");
    let github_icon = branded_icon(size, "#24292e", "#ffffff");
    show_icon(&github_icon, size, FlashPosition::TopRight)?;

    // Spotify colors
    println!("Showing Spotify-themed icon...");
    let spotify_icon = branded_icon(size, "#1DB954", "#191414");
    show_icon(&spotify_icon, size, FlashPosition::Center)?;

    // Custom gradient-like effect with Discord colors
    println!("Showing Discord-themed icon...");
    let discord_icon = branded_icon(size, "#5865F2", "#ffffff");
    show_icon(&discord_icon, size, FlashPosition::BottomRight)?;

    println!("Done!");
    Ok(())
}

fn show_icon(icon: &Icon, size: f64, position: FlashPosition) -> skylight_osd::Result<()> {
    let config = FlashConfig::new()
        .icon_size(size)
        .position(position)
        .margin(25.0);

    let mut window = SkylightWindowBuilder::from_config(&config)
        .level(WindowLevel::AboveAll)
        .build()?;

    window.draw(icon)?;
    window.show(1.2)?;

    Ok(())
}
