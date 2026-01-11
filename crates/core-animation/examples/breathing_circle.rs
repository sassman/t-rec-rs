//! Breathing circle - a soft pulsing orb with coordinated scale and opacity.
//!
//! Demonstrates multiple animations on one layer with `phase_offset` coordination.
//! The opacity peaks when scale is smallest, creating a "breathing in/out" effect
//! where the circle brightens as it contracts and dims as it expands.
//!
//! Run with: cargo run -p core-animation --example breathing_circle
//! With recording: cargo run -p core-animation --example breathing_circle --features record

use core_animation::prelude::*;

#[path = "common/mod.rs"]
mod common;

fn main() {
    println!("Breathing Circle - Soft Pulsing Orb\n");

    // Dark, calming background
    let window = WindowBuilder::new()
        .title("Breathing Circle")
        .size(400.0, 400.0)
        .centered()
        .background_color(Color::rgb(0.05, 0.05, 0.12))
        .build();

    let (width, height) = window.size();
    let center = CGPoint::new(width / 2.0, height / 2.0);

    // Large circle path
    let circle_size = 160.0;
    let circle_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(circle_size, circle_size)),
            std::ptr::null(),
        )
    };

    // Soft purple/blue gradient-like color
    let orb_color = Color::rgb(0.4, 0.5, 0.95);

    // The breathing cycle - 3 seconds for a calm, meditative feel
    let breath_duration = 3000u64.millis();

    // Create the breathing orb with coordinated animations:
    // - Scale: grows from 0.85 to 1.15 (30% range)
    // - Opacity: peaks at 1.0 when scale is smallest (phase_offset = 0.5)
    //            dips to 0.5 when scale is largest
    let breathing_orb = CAShapeLayerBuilder::new()
        .path(circle_path)
        .fill_color(orb_color)
        .bounds(CGRect::new(
            CGPoint::ZERO,
            CGSize::new(circle_size, circle_size),
        ))
        .position(center)
        // Scale animation: breathe out (expand) then in (contract)
        .animate("scale", KeyPath::TransformScale, |a| {
            a.values(0.85, 1.15)
                .duration(breath_duration)
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        // Opacity animation: dim when expanded, bright when contracted
        // Phase offset of 0.5 means opacity is at its peak when scale is at minimum
        .animate("opacity", KeyPath::Opacity, |a| {
            a.values(1.0, 0.5)
                .duration(breath_duration)
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
                .phase_offset(0.5)
        })
        .build();

    // Add a subtle outer glow ring that pulses opposite to the main orb
    let glow_size = 200.0;
    let glow_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(glow_size, glow_size)),
            std::ptr::null(),
        )
    };

    let glow_ring = CAShapeLayerBuilder::new()
        .path(glow_path)
        .fill_color(Color::TRANSPARENT)
        .stroke_color(orb_color.with_alpha(0.3))
        .line_width(3.0)
        .bounds(CGRect::new(
            CGPoint::ZERO,
            CGSize::new(glow_size, glow_size),
        ))
        .position(center)
        // Scale opposite to main orb (phase_offset = 0.5)
        .animate("scale", KeyPath::TransformScale, |a| {
            a.values(0.9, 1.1)
                .duration(breath_duration)
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
                .phase_offset(0.5)
        })
        // Opacity also offset
        .animate("opacity", KeyPath::Opacity, |a| {
            a.values(0.6, 0.2)
                .duration(breath_duration)
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        .build();

    // Add layers (glow behind main orb)
    window.container().add_sublayer(&glow_ring);
    window.container().add_sublayer(&breathing_orb);

    println!("Watch the breathing orb for 10 seconds...");
    println!("Notice how opacity peaks when the circle is smallest.\n");

    // Show with optional recording
    #[cfg(feature = "record")]
    common::show_with_recording(
        &window,
        "crates/core-animation/examples/screenshots/breathing_circle",
        10.seconds(),
    );

    #[cfg(not(feature = "record"))]
    window.show_for(10.seconds());

    println!("Done!");
}
