#![cfg(target_os = "macos")]
//! Particle emitter example using the builder API.
//!
//! Demonstrates CAEmitterLayerBuilder with the particle closure pattern.
//!
//! ## Equivalent Swift Code
//!
//! For comparison, here's how you'd write this in Swift using Apple's CAEmitterLayer API:
//!
//! ```swift
//! // Create the emitter layer
//! let emitter = CAEmitterLayer()
//! emitter.emitterPosition = CGPoint(x: size / 2.0, y: size / 2.0)
//! emitter.emitterShape = .point
//!
//! // Create and configure the particle cell
//! let cell = CAEmitterCell()
//! cell.birthRate = 100
//! cell.lifetime = 10.0
//! cell.velocity = 100
//! cell.scale = 0.1
//! cell.emissionRange = .pi * 2  // emit in all directions
//! cell.color = CGColor(red: 0.3, green: 0.8, blue: 1.0, alpha: 1.0)
//! cell.contents = createRadialGradientImage(size: 64)
//!
//! // Attach cell to emitter
//! emitter.emitterCells = [cell]
//!
//! // Add to layer hierarchy
//! view.layer.addSublayer(emitter)
//! ```
//!
//! The Rust builder API provides a more ergonomic experience with method chaining
//! and the closure pattern for particle configuration.
//!
//! Run with: cargo run -p core-animation --example emitter
//! With recording: cargo run -p core-animation --example emitter --features record

use std::f64::consts::PI;

use core_animation::prelude::*;

#[path = "common/mod.rs"]
mod common;

fn main() {
    println!("Particle Emitter Example\n");
    println!("Particles burst from a single point in all directions.\n");

    let size = 640.0;

    let window = WindowBuilder::new()
        .title("Particle Emitter")
        .size(size, size)
        .centered()
        .background_color(Color::gray(0.05).with_alpha(1.0))
        .build();

    // Create emitter at center, using the closure pattern for particle configuration
    let emitter = CAEmitterLayerBuilder::new()
        .position(size / 2.0, size / 2.0)
        .shape(EmitterShape::Point)
        .particle(|p| {
            p.birth_rate(100.0) // 100 particles per second
                .lifetime(10.0) // each lives 10 seconds
                .velocity(100.0) // move at 100 points/sec
                .scale(0.1) // scale down the image
                .emission_range(PI * 2.0) // emit in all directions
                .color(Color::CYAN) // using Color preset
                .image(ParticleImage::soft_glow(64))
        })
        .build();

    window.container().add_sublayer(&emitter);

    println!("Emitter stats:");
    println!("  - birth_rate: 100 particles/second");
    println!("  - lifetime: 10 seconds");
    println!("  - At steady state: ~1000 particles on screen\n");

    #[cfg(feature = "record")]
    common::show_with_recording(
        &window,
        "crates/core-animation/examples/screenshots/emitter",
        15.seconds(),
    );

    #[cfg(not(feature = "record"))]
    window.show_for(15.seconds());

    println!("Done!");
}
