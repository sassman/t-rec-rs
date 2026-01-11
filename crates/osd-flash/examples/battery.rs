//! Battery Indicator visualization.
//!
//! Displays a stylized battery icon with charge level.
//!
//! Run with: cargo run -p osd-flash --example battery

#[cfg(target_os = "macos")]
use osd_flash::composition::FontWeight;
#[cfg(target_os = "macos")]
use osd_flash::prelude::*;

#[cfg(target_os = "macos")]
fn main() -> osd_flash::Result<()> {
    println!("Showing battery indicators...\n");

    // Show different battery levels
    let levels = [
        (0.85, "85%", Position::TopLeft),
        (0.45, "45%", Position::Center),
        (0.15, "15%", Position::TopRight),
        (0.95, "95%", Position::BottomRight),
        (0.30, "30%", Position::BottomLeft),
    ];

    for (level, label, position) in levels {
        println!("Battery at {}", label);
        show_battery(level, label, position)?;
        std::thread::sleep(std::time::Duration::from_millis(800));
    }

    println!("\nDone!");
    Ok(())
}

#[cfg(target_os = "macos")]
fn show_battery(level: f64, label: &str, position: Position) -> osd_flash::Result<()> {
    // Battery dimensions
    let batt_width = 80.0;
    let batt_height = 40.0;
    let tip_width = 8.0;
    let tip_height = 20.0;
    let border = 4.0;

    // Padding around battery (extra at bottom for text)
    let padding = 20.0;
    let text_area_height = 16.0;

    // Window size - includes space for percentage text below battery
    let width = batt_width + tip_width + padding * 2.0;
    let height = batt_height + padding * 2.0 + text_area_height;

    // Color based on level
    let fill_color = if level > 0.5 {
        Color::rgba(0.2, 0.9, 0.3, 1.0) // Green
    } else if level > 0.2 {
        Color::rgba(1.0, 0.7, 0.0, 1.0) // Orange
    } else {
        Color::rgba(1.0, 0.2, 0.2, 1.0) // Red
    };

    // Calculate fill width based on charge level
    let inner_width = batt_width - border * 2.0 - 4.0;
    let fill_width = inner_width * level;

    // Offset to center the battery+tip combo horizontally
    let batt_offset_x = -tip_width / 2.0;
    // Offset battery up to make room for text
    let batt_offset_y = text_area_height / 2.0;

    OsdBuilder::new()
        .dimensions(width, height)
        .position(position)
        .margin(20.0)
        .background(Color::rgba(0.1, 0.1, 0.15, 0.95))
        .corner_radius(12.0)
        // Battery outline (white rounded rect)
        .layer("outline", |l| {
            l.rounded_rect(batt_width, batt_height, 6.0)
                .center_offset(batt_offset_x, batt_offset_y)
                .fill(Color::WHITE)
        })
        // Battery tip (positive terminal)
        .layer("tip", |l| {
            l.rounded_rect(tip_width, tip_height, 2.0)
                .center_offset(
                    batt_offset_x + batt_width / 2.0 + tip_width / 2.0,
                    batt_offset_y,
                )
                .fill(Color::WHITE)
        })
        // Battery inner (dark background)
        .layer("inner", |l| {
            l.rounded_rect(batt_width - border * 2.0, batt_height - border * 2.0, 4.0)
                .center_offset(batt_offset_x, batt_offset_y)
                .fill(Color::rgba(0.1, 0.1, 0.15, 1.0))
        })
        // Battery fill (charge level) - positioned from left edge
        .layer("fill", |l| {
            // Calculate position to align fill with left edge of inner area
            let fill_offset_x = batt_offset_x - (inner_width - fill_width) / 2.0;
            l.rounded_rect(fill_width, batt_height - border * 2.0 - 4.0, 3.0)
                .center_offset(fill_offset_x, batt_offset_y)
                .fill(fill_color)
        })
        // Percentage label (centered in the text area below battery)
        .layer("label", |l| {
            l.text(label)
                .font_size(16.0)
                .font_weight(FontWeight::Medium)
                .text_color(Color::WHITE)
                .center_offset(0.0, -(batt_height / 2.0 + text_area_height / 2.0))
        })
        .show_for(2.seconds())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This example only runs on macOS");
}
