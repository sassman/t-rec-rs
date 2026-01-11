//! Sierpiński Triangle Fractal (simplified).
//!
//! A representation of the classic fractal using layered circles.
//! Demonstrates the triangular pattern with color-coded vertices.
//!
//! Run with: cargo run -p osd-flash --example sierpinski

#[cfg(target_os = "macos")]
use osd_flash::prelude::*;

#[cfg(target_os = "macos")]
fn main() -> osd_flash::Result<()> {
    println!("Showing Sierpinski triangle pattern...\n");

    let size = 300.0;

    // Colors for each vertex region
    let pink = Color::rgba(1.0, 0.3, 0.5, 0.9);
    let cyan = Color::rgba(0.3, 0.8, 1.0, 0.9);
    let green = Color::rgba(0.5, 1.0, 0.4, 0.9);

    OsdBuilder::new()
        .size(size)
        .position(Position::Center)
        .background(Color::rgba(0.02, 0.02, 0.05, 0.98))
        .corner_radius(24.0)
        // Top vertex marker
        .layer("top_glow", |l| {
            l.circle(30.0)
                .center_offset(0.0, -80.0)
                .fill(pink.with_alpha(0.3))
                .animate(Animation::pulse_range(0.9, 1.1))
        })
        .layer("top", |l| {
            l.circle(16.0).center_offset(0.0, -80.0).fill(Color::WHITE)
        })
        // Bottom-left vertex marker
        .layer("bl_glow", |l| {
            l.circle(30.0)
                .center_offset(-90.0, 70.0)
                .fill(cyan.with_alpha(0.3))
                .animate(Animation::pulse_range(0.9, 1.1))
        })
        .layer("bl", |l| {
            l.circle(16.0).center_offset(-90.0, 70.0).fill(Color::WHITE)
        })
        // Bottom-right vertex marker
        .layer("br_glow", |l| {
            l.circle(30.0)
                .center_offset(90.0, 70.0)
                .fill(green.with_alpha(0.3))
                .animate(Animation::pulse_range(0.9, 1.1))
        })
        .layer("br", |l| {
            l.circle(16.0).center_offset(90.0, 70.0).fill(Color::WHITE)
        })
        // Inner triangular pattern (represented with circles at midpoints)
        // Top-left midpoint
        .layer("mid_tl", |l| {
            l.circle(12.0)
                .center_offset(-45.0, -5.0)
                .fill(pink.with_alpha(0.7))
        })
        // Top-right midpoint
        .layer("mid_tr", |l| {
            l.circle(12.0)
                .center_offset(45.0, -5.0)
                .fill(green.with_alpha(0.7))
        })
        // Bottom midpoint
        .layer("mid_b", |l| {
            l.circle(12.0)
                .center_offset(0.0, 70.0)
                .fill(cyan.with_alpha(0.7))
        })
        // Central void (dark)
        .layer("center_void", |l| {
            l.circle(40.0)
                .center_offset(0.0, 20.0)
                .fill(Color::rgba(0.02, 0.02, 0.05, 0.9))
        })
        // Smaller fractal points
        .layer("p1", |l| {
            l.circle(6.0)
                .center_offset(-22.0, -42.0)
                .fill(pink.with_alpha(0.6))
        })
        .layer("p2", |l| {
            l.circle(6.0)
                .center_offset(22.0, -42.0)
                .fill(green.with_alpha(0.6))
        })
        .layer("p3", |l| {
            l.circle(6.0)
                .center_offset(-67.0, 33.0)
                .fill(cyan.with_alpha(0.6))
        })
        .layer("p4", |l| {
            l.circle(6.0)
                .center_offset(67.0, 33.0)
                .fill(green.with_alpha(0.6))
        })
        .layer("p5", |l| {
            l.circle(6.0)
                .center_offset(-22.0, 70.0)
                .fill(cyan.with_alpha(0.6))
        })
        .layer("p6", |l| {
            l.circle(6.0)
                .center_offset(22.0, 70.0)
                .fill(green.with_alpha(0.6))
        })
        // Title
        .layer("title", |l| {
            l.text("SIERPINSKI")
                .center_offset(0.0, 115.0)
                .font_size(14.0)
                .text_color(Color::rgba(0.7, 0.7, 0.9, 0.9))
        })
        .show_for(6.seconds())?;

    println!("Done!");
    Ok(())
}


#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This example only runs on macOS");
}
