#![cfg(target_os = "macos")]
//! Neon glow - retro neon sign effect with pulsing shadows.
//!
//! Demonstrates shadow property animations (`ShadowRadius`, `ShadowOpacity`)
//! to create a glowing neon effect. Uses bright neon colors (pink, cyan) on
//! a dark background for maximum contrast.
//!
//! Run with: cargo run -p core-animation --example neon_glow
//! With recording: cargo run -p core-animation --example neon_glow --features record

use core_animation::prelude::*;

#[path = "common/mod.rs"]
mod common;

fn main() {
    println!("Neon Glow - Retro Neon Sign Effect\n");

    // Very dark background for neon contrast
    let window = WindowBuilder::new()
        .title("Neon Glow")
        .size(500.0, 400.0)
        .centered()
        .background_color(Color::rgb(0.02, 0.02, 0.05))
        .build();

    let (width, height) = window.size();

    // Neon pink color
    let neon_pink = Color::rgb(1.0, 0.2, 0.6);
    // Neon cyan color
    let neon_cyan = Color::rgb(0.0, 0.95, 1.0);

    // Create a rounded rectangle "neon tube" - main shape
    let rect_width = 200.0;
    let rect_height = 80.0;
    let corner_radius = 40.0; // Pill shape

    let neon_path = unsafe {
        CGPath::with_rounded_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(rect_width, rect_height)),
            corner_radius,
            corner_radius,
            std::ptr::null(),
        )
    };

    // Glow pulse duration
    let glow_duration = 1500u64.millis();

    // Pink neon shape (top)
    let pink_neon = CAShapeLayerBuilder::new()
        .path(neon_path.clone())
        .fill_color(Color::TRANSPARENT)
        .stroke_color(neon_pink)
        .line_width(6.0)
        .bounds(CGRect::new(
            CGPoint::ZERO,
            CGSize::new(rect_width, rect_height),
        ))
        .position(CGPoint::new(width / 2.0, height / 2.0 - 70.0))
        // Shadow properties (base values, will be animated)
        .shadow_color(neon_pink)
        .shadow_offset(0.0, 0.0)
        .shadow_radius(15.0)
        .shadow_opacity(0.7)
        // Animate shadow radius for glow intensity
        .animate("glow_radius", KeyPath::ShadowRadius, |a| {
            a.values(15.0, 35.0)
                .duration(glow_duration)
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        // Animate shadow opacity for pulsing effect
        .animate("glow_opacity", KeyPath::ShadowOpacity, |a| {
            a.values(0.7, 1.0)
                .duration(glow_duration)
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        .build();

    // Cyan neon shape (bottom) - slightly offset phase for visual interest
    let cyan_neon = CAShapeLayerBuilder::new()
        .path(neon_path.clone())
        .fill_color(Color::TRANSPARENT)
        .stroke_color(neon_cyan)
        .line_width(6.0)
        .bounds(CGRect::new(
            CGPoint::ZERO,
            CGSize::new(rect_width, rect_height),
        ))
        .position(CGPoint::new(width / 2.0, height / 2.0 + 70.0))
        // Shadow properties (base values, will be animated)
        .shadow_color(neon_cyan)
        .shadow_offset(0.0, 0.0)
        .shadow_radius(15.0)
        .shadow_opacity(0.7)
        // Same glow animations but offset phase
        .animate("glow_radius", KeyPath::ShadowRadius, |a| {
            a.values(15.0, 35.0)
                .duration(glow_duration)
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
                .phase_offset(0.5) // Offset so they alternate
        })
        .animate("glow_opacity", KeyPath::ShadowOpacity, |a| {
            a.values(0.7, 1.0)
                .duration(glow_duration)
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
                .phase_offset(0.5)
        })
        .build();

    // Create a small circle "dot" that also glows
    let dot_size = 30.0;
    let dot_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(dot_size, dot_size)),
            std::ptr::null(),
        )
    };

    // Create three dots between the neon bars
    let dot_y = height / 2.0;
    let dot_spacing = 50.0;

    for (i, x_offset) in [-dot_spacing, 0.0, dot_spacing].iter().enumerate() {
        let dot_color = if i % 2 == 0 { neon_pink } else { neon_cyan };
        let phase = i as f64 * 0.33;

        let dot = CAShapeLayerBuilder::new()
            .path(dot_path.clone())
            .fill_color(dot_color)
            .bounds(CGRect::new(CGPoint::ZERO, CGSize::new(dot_size, dot_size)))
            .position(CGPoint::new(width / 2.0 + x_offset, dot_y))
            // Shadow for glow - using builder methods
            .shadow_color(dot_color)
            .shadow_offset(0.0, 0.0)
            .shadow_radius(8.0)
            .shadow_opacity(0.9)
            .animate("glow_radius", KeyPath::ShadowRadius, |a| {
                a.values(8.0, 20.0)
                    .duration(800u64.millis())
                    .easing(Easing::InOut)
                    .autoreverses()
                    .repeat(Repeat::Forever)
                    .phase_offset(phase)
            })
            .animate("scale", KeyPath::TransformScale, |a| {
                a.values(0.9, 1.1)
                    .duration(800u64.millis())
                    .easing(Easing::InOut)
                    .autoreverses()
                    .repeat(Repeat::Forever)
                    .phase_offset(phase)
            })
            .build();

        window.container().add_sublayer(&dot);
    }

    // Add neon shapes
    window.container().add_sublayer(&pink_neon);
    window.container().add_sublayer(&cyan_neon);

    println!("Watch the neon glow effect for 10 seconds...");
    println!("Notice the pulsing shadow radius creating the glow.\n");

    // Show with optional recording
    #[cfg(feature = "record")]
    common::show_with_recording(
        &window,
        "crates/core-animation/examples/screenshots/neon_glow",
        10.seconds(),
    );

    #[cfg(not(feature = "record"))]
    window.show_for(10.seconds());

    println!("Done!");
}
