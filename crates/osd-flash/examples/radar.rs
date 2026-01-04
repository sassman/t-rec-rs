//! Radar / Sonar display visualization.
//!
//! Displays concentric circles like a radar sweep with detected blips.
//!
//! Run with: cargo run -p osd-flash --example radar

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    println!("Showing radar display...\n");

    let size = 300.0;
    let padding = 20.0;

    let mut window = OsdFlashBuilder::new()
        .dimensions(size)
        .position(FlashPosition::Center)
        .background(Color::rgba(0.0, 0.05, 0.0, 0.95))
        .corner_radius(size / 2.0) // Make it circular!
        .padding(Padding::all(padding))
        .build()?;

    let content_center = (size - 2.0 * padding) / 2.0;

    // Draw concentric radar rings (from outside in)
    let ring_radii = [120.0, 90.0, 60.0, 30.0];
    for radius in ring_radii {
        // Draw ring as a circle outline (filled circle with smaller dark circle on top)
        window = window.draw(StyledShape::new(
            Shape::circle_at(content_center, content_center, radius),
            Color::rgba(0.0, 0.4, 0.0, 0.6),
        ));
        window = window.draw(StyledShape::new(
            Shape::circle_at(content_center, content_center, radius - 2.0),
            Color::rgba(0.0, 0.05, 0.0, 1.0),
        ));
    }

    // Draw crosshairs
    let cross_len = 115.0;
    let cross_width = 2.0;
    // Horizontal line
    window = window.draw(StyledShape::new(
        Shape::rounded_rect(
            Rect::from_xywh(
                content_center - cross_len,
                content_center - cross_width / 2.0,
                cross_len * 2.0,
                cross_width,
            ),
            1.0,
        ),
        Color::rgba(0.0, 0.5, 0.0, 0.5),
    ));
    // Vertical line
    window = window.draw(StyledShape::new(
        Shape::rounded_rect(
            Rect::from_xywh(
                content_center - cross_width / 2.0,
                content_center - cross_len,
                cross_width,
                cross_len * 2.0,
            ),
            1.0,
        ),
        Color::rgba(0.0, 0.5, 0.0, 0.5),
    ));

    // Draw radar blips (detected objects)
    let blips = [
        (content_center + 50.0, content_center - 40.0, 8.0),  // Blip 1
        (content_center - 70.0, content_center + 30.0, 6.0),  // Blip 2
        (content_center + 20.0, content_center + 80.0, 10.0), // Blip 3
        (content_center - 30.0, content_center - 60.0, 5.0),  // Blip 4
    ];

    for (x, y, size) in blips {
        // Glow effect
        window = window.draw(StyledShape::new(
            Shape::circle_at(x, y, size + 4.0),
            Color::rgba(0.0, 1.0, 0.0, 0.3),
        ));
        // Bright center
        window = window.draw(StyledShape::new(
            Shape::circle_at(x, y, size),
            Color::rgba(0.3, 1.0, 0.3, 1.0),
        ));
    }

    // Center dot
    window = window.draw(StyledShape::new(
        Shape::circle_at(content_center, content_center, 5.0),
        Color::rgba(0.0, 1.0, 0.0, 1.0),
    ));

    // Title
    window = window.draw(StyledText::at("RADAR", content_center - 25.0, 10.0, 14.0, Color::rgba(0.0, 0.8, 0.0, 1.0)));

    window.show_for_seconds(4.0)?;

    println!("Done!");
    Ok(())
}
