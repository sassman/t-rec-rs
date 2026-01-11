//! Hex colors example.
//!
//! Demonstrates using hex color codes to create branded icons.
//!
//! Run with: cargo run -p osd-flash --example hex_colors

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    let size = 100.0;

    // GitHub colors
    println!("Showing GitHub-themed icon...");
    let github_primary = Color::from_hex("#24292e").unwrap_or(Color::BLUE);
    let github_secondary = Color::from_hex("#ffffff").unwrap_or(Color::WHITE);

    OsdBuilder::new()
        .size(size)
        .position(Position::TopRight)
        .margin(25.0)
        .level(WindowLevel::AboveAll)
        .background(github_primary.with_alpha(0.95))
        .corner_radius(18.0)
        .layer("dot1", |l| {
            l.circle(24.0)
                .center_offset(-15.0, -10.0)
                .fill(github_secondary.with_alpha(0.9))
        })
        .layer("dot2", |l| {
            l.circle(24.0)
                .center_offset(15.0, -10.0)
                .fill(github_secondary.with_alpha(0.9))
        })
        .layer("dot3", |l| {
            l.circle(28.0)
                .center_offset(0.0, 12.0)
                .fill(github_secondary.with_alpha(0.9))
        })
        .show_for(1200.millis())?;

    std::thread::sleep(std::time::Duration::from_millis(300));

    // Spotify colors
    println!("Showing Spotify-themed icon...");
    let spotify_primary = Color::from_hex("#1DB954").unwrap_or(Color::GREEN);
    let spotify_secondary = Color::from_hex("#191414").unwrap_or(Color::BLACK);

    OsdBuilder::new()
        .size(size)
        .position(Position::Center)
        .background(spotify_primary.with_alpha(0.95))
        .corner_radius(18.0)
        .layer("dot1", |l| {
            l.circle(24.0)
                .center_offset(-15.0, -10.0)
                .fill(spotify_secondary.with_alpha(0.9))
        })
        .layer("dot2", |l| {
            l.circle(24.0)
                .center_offset(15.0, -10.0)
                .fill(spotify_secondary.with_alpha(0.9))
        })
        .layer("dot3", |l| {
            l.circle(28.0)
                .center_offset(0.0, 12.0)
                .fill(spotify_secondary.with_alpha(0.9))
        })
        .show_for(1200.millis())?;

    std::thread::sleep(std::time::Duration::from_millis(300));

    // Discord colors
    println!("Showing Discord-themed icon...");
    let discord_primary = Color::from_hex("#5865F2").unwrap_or(Color::BLUE);
    let discord_secondary = Color::from_hex("#ffffff").unwrap_or(Color::WHITE);

    OsdBuilder::new()
        .size(size)
        .position(Position::BottomRight)
        .margin(25.0)
        .level(WindowLevel::AboveAll)
        .background(discord_primary.with_alpha(0.95))
        .corner_radius(18.0)
        .layer("dot1", |l| {
            l.circle(24.0)
                .center_offset(-15.0, -10.0)
                .fill(discord_secondary.with_alpha(0.9))
        })
        .layer("dot2", |l| {
            l.circle(24.0)
                .center_offset(15.0, -10.0)
                .fill(discord_secondary.with_alpha(0.9))
        })
        .layer("dot3", |l| {
            l.circle(28.0)
                .center_offset(0.0, 12.0)
                .fill(discord_secondary.with_alpha(0.9))
        })
        .show_for(1200.millis())?;

    println!("Done!");
    Ok(())
}
