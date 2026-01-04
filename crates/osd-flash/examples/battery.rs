//! Battery Indicator visualization.
//!
//! Displays a stylized battery icon with charge level.
//!
//! Run with: cargo run -p osd-flash --example battery

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    println!("Showing battery indicators...\n");

    // Show different battery levels
    let levels = [(0.85, "85%"), (0.45, "45%"), (0.15, "15%")];
    let positions = [FlashPosition::TopLeft, FlashPosition::Center, FlashPosition::TopRight];

    for ((level, label), position) in levels.iter().zip(positions.iter()) {
        println!("Battery at {}", label);
        show_battery(*level, label, *position)?;
        std::thread::sleep(std::time::Duration::from_millis(800));
    }

    println!("\nDone!");
    Ok(())
}

fn show_battery(level: f64, label: &str, position: FlashPosition) -> osd_flash::Result<()> {
    let padding = 15.0;

    // Battery dimensions
    let batt_width = 80.0;
    let batt_height = 40.0;
    let tip_width = 6.0;
    let tip_height = 16.0;
    let border = 3.0;

    // Content area needs to fit: battery + tip + some margin for text
    let content_width = batt_width + tip_width + 10.0;
    let content_height = batt_height + 25.0; // Room for text below

    // Window = content + padding
    let width = content_width + 2.0 * padding;
    let height = content_height + 2.0 * padding;

    // Center battery in content area
    let batt_x = (content_width - batt_width - tip_width) / 2.0;
    let batt_y = 5.0;

    // Color based on level
    let fill_color = if level > 0.5 {
        Color::rgba(0.2, 0.9, 0.3, 1.0) // Green
    } else if level > 0.2 {
        Color::rgba(1.0, 0.7, 0.0, 1.0) // Orange
    } else {
        Color::rgba(1.0, 0.2, 0.2, 1.0) // Red
    };

    let mut window = OsdFlashBuilder::new()
        .dimensions(Size::new(width, height))
        .position(position)
        .margin(20.0)
        .background(Color::rgba(0.1, 0.1, 0.15, 0.95))
        .corner_radius(16.0)
        .padding(Padding::all(padding))
        .build()?;

    // Battery outline
    window = window.draw(StyledShape::new(
        Shape::rounded_rect(Rect::from_xywh(batt_x, batt_y, batt_width, batt_height), 6.0),
        Color::WHITE,
    ));

    // Battery tip (positive terminal)
    window = window.draw(StyledShape::new(
        Shape::rounded_rect(
            Rect::from_xywh(
                batt_x + batt_width,
                batt_y + (batt_height - tip_height) / 2.0,
                tip_width,
                tip_height,
            ),
            2.0,
        ),
        Color::WHITE,
    ));

    // Battery inner (dark)
    window = window.draw(StyledShape::new(
        Shape::rounded_rect(
            Rect::from_xywh(
                batt_x + border,
                batt_y + border,
                batt_width - border * 2.0,
                batt_height - border * 2.0,
            ),
            3.0,
        ),
        Color::rgba(0.1, 0.1, 0.15, 1.0),
    ));

    // Battery fill (charge level)
    let fill_width = (batt_width - border * 2.0 - 4.0) * level;
    window = window.draw(StyledShape::new(
        Shape::rounded_rect(
            Rect::from_xywh(
                batt_x + border + 2.0,
                batt_y + border + 2.0,
                fill_width,
                batt_height - border * 2.0 - 4.0,
            ),
            2.0,
        ),
        fill_color,
    ));

    // Percentage label (centered below battery)
    let text_x = content_width / 2.0 - 12.0;
    window = window.draw(StyledText::at(label, text_x, batt_y + batt_height + 8.0, 14.0, Color::WHITE));

    window.show_for_seconds(2.0)?;
    Ok(())
}
