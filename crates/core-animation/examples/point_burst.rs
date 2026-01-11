//! PointBurstBuilder example - simplified API for common particle burst pattern.
//!
//! Demonstrates the convenience builder for particles bursting from a point.
//!
//! Run with: cargo run -p core-animation --example point_burst
//! With recording: cargo run -p core-animation --example point_burst --features record

#[cfg(target_os = "macos")]
use core_animation::prelude::*;

#[cfg(target_os = "macos")]
#[path = "common/mod.rs"]
mod common;

#[cfg(target_os = "macos")]
fn main() {
    println!("PointBurstBuilder Example\n");
    println!("Simplified API for particles bursting from a point.\n");

    let size = 640.0;

    let window = WindowBuilder::new()
        .title("Point Burst")
        .size(size, size)
        .centered()
        .background_color(Color::gray(0.02))
        .build();

    // Compare: CAEmitterLayerBuilder requires more setup
    // PointBurstBuilder provides sensible defaults for the common case

    // Simple burst - just position, velocity, lifetime, and color
    let simple_burst = PointBurstBuilder::new(size / 2.0, size / 2.0)
        .birth_rate(80.0)
        .velocity(120.0)
        .lifetime(4.0)
        .color(Color::CYAN)
        .build();

    window.container().add_sublayer(&simple_burst);

    // Additional bursts around the edges with different colors
    let corner_positions = [
        (size * 0.2, size * 0.2),
        (size * 0.8, size * 0.2),
        (size * 0.2, size * 0.8),
        (size * 0.8, size * 0.8),
    ];

    // Using Color presets and constructors
    let colors = [
        Color::PINK,               // pink
        Color::rgb(0.5, 1.0, 0.4), // green
        Color::ORANGE,             // orange/gold
        Color::PURPLE,             // purple
    ];

    for (i, (x, y)) in corner_positions.iter().enumerate() {
        let burst = PointBurstBuilder::new(*x, *y)
            .birth_rate(40.0)
            .velocity(80.0)
            .lifetime(3.0)
            .scale(0.08)
            .alpha_speed(-0.2) // fade out
            .color(colors[i])
            .image(ParticleImage::star(48, 4)) // use star particles
            .build();

        window.container().add_sublayer(&burst);
    }

    println!("Center: cyan soft glow burst");
    println!("Corners: colored star bursts with fade\n");
    println!("PointBurstBuilder simplifies the common pattern of");
    println!("particles exploding outward from a single point.\n");

    #[cfg(feature = "record")]
    common::show_with_recording(
        &window,
        "crates/core-animation/examples/screenshots/point_burst",
        12.seconds(),
    );

    #[cfg(not(feature = "record"))]
    window.show_for(12.seconds());

    println!("Done!");
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This example only runs on macOS");
}
