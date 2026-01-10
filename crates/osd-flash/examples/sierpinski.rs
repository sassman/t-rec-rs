//! Sierpiński Triangle Fractal.
//!
//! A beautiful recursive fractal rendered with circles using the chaos game.
//! This creates a stunning point-cloud visualization of the classic fractal.
//!
//! Run with: cargo run -p osd-flash --example sierpinski

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    println!("Generating Sierpinski triangle fractal (chaos game)...\n");

    let size = 520.0;
    let padding = 30.0;
    let content_size = size - 2.0 * padding;

    // Triangle vertices (equilateral, pointing up)
    let margin = 15.0;
    let tri_width = content_size - 2.0 * margin;
    let tri_height = tri_width * 0.866; // sqrt(3)/2 for equilateral

    let vertices = [
        (content_size / 2.0, margin),                 // Top
        (margin, margin + tri_height),                // Bottom left
        (content_size - margin, margin + tri_height), // Bottom right
    ];

    // Colors for each vertex region
    let colors = [
        Color::rgba(1.0, 0.3, 0.5, 0.9), // Pink/red for top
        Color::rgba(0.3, 0.8, 1.0, 0.9), // Cyan for bottom left
        Color::rgba(0.5, 1.0, 0.4, 0.9), // Green for bottom right
    ];

    // Chaos game: start at random point, repeatedly jump halfway to a random vertex
    let mut x = content_size / 2.0;
    let mut y = content_size / 2.0;

    // Simple pseudo-random using linear congruential generator
    let mut seed: u64 = 12345;
    let lcg_next = |s: &mut u64| -> usize {
        *s = s.wrapping_mul(1103515245).wrapping_add(12345);
        ((*s >> 16) % 3) as usize
    };

    // Generate points
    let num_points = 3000;
    let mut points: Vec<(f64, f64, usize)> = Vec::with_capacity(num_points);

    for _ in 0..num_points {
        let vertex_idx = lcg_next(&mut seed);
        let (vx, vy) = vertices[vertex_idx];

        // Move halfway toward the chosen vertex
        x = (x + vx) / 2.0;
        y = (y + vy) / 2.0;

        points.push((x, y, vertex_idx));
    }

    // Collect all shapes
    let mut shapes: Vec<StyledShape> = Vec::new();

    // Draw all points (skip first few iterations for convergence)
    for (px, py, color_idx) in points.iter().skip(20) {
        let color = colors[*color_idx];
        shapes.push(StyledShape::new(Shape::circle_at(*px, *py, 1.5), color));
    }

    // Draw vertex markers
    for (i, (vx, vy)) in vertices.iter().enumerate() {
        // Outer glow
        shapes.push(StyledShape::new(
            Shape::circle_at(*vx, *vy, 8.0),
            colors[i].with_alpha(0.3),
        ));
        // Inner bright
        shapes.push(StyledShape::new(
            Shape::circle_at(*vx, *vy, 4.0),
            Color::WHITE,
        ));
    }

    OsdFlashBuilder::new()
        .dimensions(size)
        .position(FlashPosition::Center)
        .background(Color::rgba(0.02, 0.02, 0.05, 0.98))
        .corner_radius(24.0)
        .padding(Padding::all(padding))
        .build()?
        .draw(shapes)
        // Title
        .draw(StyledText::at(
            "SIERPINSKI FRACTAL",
            content_size / 2.0 - 80.0,
            content_size - 24.0,
            14.0,
            Color::rgba(0.7, 0.7, 0.9, 0.9),
        ))
        .show_for_seconds(6.0)?;

    println!("Done!");
    Ok(())
}
