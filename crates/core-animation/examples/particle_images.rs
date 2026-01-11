//! Particle image showcase - demonstrates all ParticleImage types.
//!
//! Shows soft_glow, circle, star, and spark particle images side by side.
//!
//! Run with: cargo run -p core-animation --example particle_images
//! With recording: cargo run -p core-animation --example particle_images --features record

#[cfg(target_os = "macos")]
use std::f64::consts::PI;

#[cfg(target_os = "macos")]
use core_animation::prelude::*;

#[cfg(target_os = "macos")]
#[path = "common/mod.rs"]
mod common;

#[cfg(target_os = "macos")]
fn main() {
    println!("Particle Images Showcase\n");
    println!("Four emitters demonstrating different particle image types.\n");

    let size = 800.0;

    let window = WindowBuilder::new()
        .title("Particle Images")
        .size(size, size)
        .centered()
        .background_rgba(0.02, 0.02, 0.06, 1.0)
        .build();

    // Grid layout: 2x2
    let positions = [
        (size * 0.25, size * 0.75), // top-left: soft_glow
        (size * 0.75, size * 0.75), // top-right: circle
        (size * 0.25, size * 0.25), // bottom-left: star
        (size * 0.75, size * 0.25), // bottom-right: spark
    ];

    let colors = [
        (0.3, 0.8, 1.0), // cyan
        (1.0, 0.4, 0.6), // pink
        (1.0, 0.9, 0.3), // gold
        (1.0, 0.5, 0.2), // orange
    ];

    let labels = ["soft_glow(64)", "circle(48)", "star(64, 5)", "spark(64)"];

    // Soft glow emitter (top-left)
    let soft_glow_emitter = CAEmitterLayerBuilder::new()
        .position(positions[0].0, positions[0].1)
        .shape(EmitterShape::Point)
        .render_mode(RenderMode::Additive)
        .particle(|p| {
            p.birth_rate(30.0)
                .lifetime(3.0)
                .velocity(60.0)
                .emission_range(PI * 2.0)
                .scale(0.15)
                .scale_speed(-0.03)
                .alpha_speed(-0.2)
                .color_rgb(colors[0].0, colors[0].1, colors[0].2)
                .image(ParticleImage::soft_glow(64))
        })
        .build();

    // Circle emitter (top-right)
    let circle_emitter = CAEmitterLayerBuilder::new()
        .position(positions[1].0, positions[1].1)
        .shape(EmitterShape::Point)
        .render_mode(RenderMode::Additive)
        .particle(|p| {
            p.birth_rate(30.0)
                .lifetime(3.0)
                .velocity(60.0)
                .emission_range(PI * 2.0)
                .scale(0.12)
                .scale_speed(-0.02)
                .alpha_speed(-0.2)
                .color_rgb(colors[1].0, colors[1].1, colors[1].2)
                .image(ParticleImage::circle(48))
        })
        .build();

    // Star emitter (bottom-left)
    let star_emitter = CAEmitterLayerBuilder::new()
        .position(positions[2].0, positions[2].1)
        .shape(EmitterShape::Point)
        .render_mode(RenderMode::Additive)
        .particle(|p| {
            p.birth_rate(20.0)
                .lifetime(4.0)
                .velocity(50.0)
                .emission_range(PI * 2.0)
                .scale(0.2)
                .scale_speed(-0.03)
                .alpha_speed(-0.15)
                .spin(1.0) // Rotate stars
                .spin_range(0.5)
                .color_rgb(colors[2].0, colors[2].1, colors[2].2)
                .image(ParticleImage::star(64, 5))
        })
        .build();

    // Spark emitter (bottom-right) - sparks look best with some velocity
    let spark_emitter = CAEmitterLayerBuilder::new()
        .position(positions[3].0, positions[3].1)
        .shape(EmitterShape::Point)
        .render_mode(RenderMode::Additive)
        .particle(|p| {
            p.birth_rate(40.0)
                .lifetime(2.0)
                .velocity(100.0)
                .velocity_range(30.0)
                .emission_range(PI * 2.0)
                .scale(0.3)
                .scale_speed(-0.1)
                .alpha_speed(-0.3)
                .color_rgb(colors[3].0, colors[3].1, colors[3].2)
                .image(ParticleImage::spark(64))
        })
        .build();

    window.container().add_sublayer(&soft_glow_emitter);
    window.container().add_sublayer(&circle_emitter);
    window.container().add_sublayer(&star_emitter);
    window.container().add_sublayer(&spark_emitter);

    // Print labels
    println!("Particle types (clockwise from top-left):");
    for (i, label) in labels.iter().enumerate() {
        println!("  {}: ParticleImage::{}", i + 1, label);
    }
    println!();

    #[cfg(feature = "record")]
    common::show_with_recording(
        &window,
        "crates/core-animation/examples/screenshots/particle_images",
        15.seconds(),
    );

    #[cfg(not(feature = "record"))]
    window.show_for(15.seconds());

    println!("Done!");
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This example only runs on macOS");
}
