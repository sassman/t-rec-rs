//! Ripple rings - water ripple effect with concentric expanding rings.
//!
//! Creates a mesmerizing effect like dropping a pebble in still water.
//! Multiple rings expand outward from a central point, each staggered
//! using `phase_offset` to create continuous ripple emanation.
//!
//! Demonstrates:
//! - `KeyPath::TransformScale` for ring expansion
//! - `KeyPath::Opacity` for rings fading as they expand
//! - `KeyPath::Custom("lineWidth")` for pulsing stroke width
//! - `phase_offset` for staggered timing across multiple rings
//! - Multiple animations per layer
//! - `Easing::Out` for realistic ripple physics (fast start, slow end)
//!
//! Run with: cargo run -p core-animation --example ripple_rings
//! With recording: cargo run -p core-animation --example ripple_rings --features record

#[cfg(target_os = "macos")]
use core_animation::prelude::*;

#[cfg(target_os = "macos")]
#[path = "common/mod.rs"]
mod common;

#[cfg(target_os = "macos")]
fn main() {
    println!("Ripple Rings - Water Ripple Effect\n");

    // Dark water-like background
    let window = WindowBuilder::new()
        .title("Ripple Rings")
        .size(500.0, 500.0)
        .centered()
        .background_color(Color::rgb(0.02, 0.05, 0.1))
        .build();

    let (width, height) = window.size();
    let center = CGPoint::new(width / 2.0, height / 2.0);

    // Color palette - calming cyan/blue water tones
    let water_cyan = Color::rgb(0.0, 0.75, 0.9);
    let water_blue = Color::rgb(0.2, 0.5, 0.85);

    // Central orb - the "impact point" that pulses subtly
    let orb_size = 20.0;
    let orb_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(orb_size, orb_size)),
            std::ptr::null(),
        )
    };

    let pulse_duration = 2500u64.millis();

    let central_orb = CAShapeLayerBuilder::new()
        .path(orb_path)
        .fill_color(water_cyan)
        .bounds(CGRect::new(CGPoint::ZERO, CGSize::new(orb_size, orb_size)))
        .position(center)
        // Subtle scale pulse
        .animate("pulse_scale", KeyPath::TransformScale, |a| {
            a.values(0.8, 1.2)
                .duration(pulse_duration)
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        // Subtle opacity pulse (brighter when contracted)
        .animate("pulse_opacity", KeyPath::Opacity, |a| {
            a.values(1.0, 0.6)
                .duration(pulse_duration)
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
                .phase_offset(0.5)
        })
        .build();

    // Add a soft glow behind the central orb
    let glow_size = 40.0;
    let glow_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(glow_size, glow_size)),
            std::ptr::null(),
        )
    };

    let central_glow = CAShapeLayerBuilder::new()
        .path(glow_path)
        .fill_color(water_cyan.with_alpha(0.3))
        .bounds(CGRect::new(
            CGPoint::ZERO,
            CGSize::new(glow_size, glow_size),
        ))
        .position(center)
        .animate("glow_scale", KeyPath::TransformScale, |a| {
            a.values(0.9, 1.4)
                .duration(pulse_duration)
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        .animate("glow_opacity", KeyPath::Opacity, |a| {
            a.values(0.5, 0.2)
                .duration(pulse_duration)
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        .build();

    // Ring configuration
    let num_rings = 5;
    let ring_cycle_duration = 3000u64.millis(); // Slow, meditative pace
    let max_ring_size = 400.0; // Maximum expansion size
    let initial_ring_size = 30.0; // Starting size (small, near center)

    // Create expanding ripple rings
    for i in 0..num_rings {
        // Phase offset creates the staggered emanation effect
        // Each ring starts at a different point in the cycle
        let phase = i as f64 / num_rings as f64;

        // Alternate between cyan and blue for visual depth
        let ring_color = if i % 2 == 0 { water_cyan } else { water_blue };

        // Create a circle path for this ring
        // We use a fixed path size and animate the scale transform
        let ring_path = unsafe {
            CGPath::with_ellipse_in_rect(
                CGRect::new(
                    CGPoint::ZERO,
                    CGSize::new(initial_ring_size, initial_ring_size),
                ),
                std::ptr::null(),
            )
        };

        // Initial line width - varies slightly per ring for visual interest
        let base_line_width = 2.0 + (i as f64 * 0.3);

        let ring = CAShapeLayerBuilder::new()
            .path(ring_path)
            .fill_color(Color::TRANSPARENT) // No fill - just stroke
            .stroke_color(ring_color)
            .line_width(base_line_width)
            .bounds(CGRect::new(
                CGPoint::ZERO,
                CGSize::new(initial_ring_size, initial_ring_size),
            ))
            .position(center)
            // Scale animation: ring expands outward
            // Using Easing::Out for realistic ripple physics (fast start, slow end)
            .animate("expand", KeyPath::TransformScale, |a| {
                let max_scale = max_ring_size / initial_ring_size;
                a.values(1.0, max_scale)
                    .duration(ring_cycle_duration)
                    .easing(Easing::Out)
                    .repeat(Repeat::Forever)
                    .phase_offset(phase)
            })
            // Opacity animation: fade out as ring expands
            .animate("fade", KeyPath::Opacity, |a| {
                a.values(0.9, 0.0)
                    .duration(ring_cycle_duration)
                    .easing(Easing::Out)
                    .repeat(Repeat::Forever)
                    .phase_offset(phase)
            })
            // Line width animation: pulse the stroke width for extra visual interest
            .animate("stroke_pulse", KeyPath::Custom("lineWidth"), |a| {
                a.values(base_line_width, base_line_width * 0.3)
                    .duration(ring_cycle_duration)
                    .easing(Easing::Out)
                    .repeat(Repeat::Forever)
                    .phase_offset(phase)
            })
            .build();

        // Add rings first (behind the central orb)
        window.container().add_sublayer(&ring);
    }

    // Add central elements on top
    window.container().add_sublayer(&central_glow);
    window.container().add_sublayer(&central_orb);

    println!("Watch the ripple effect for 15 seconds...");
    println!("Like a pebble dropped in still water.\n");
    println!("Notice:");
    println!("  - Rings expand outward from center");
    println!("  - Each ring fades as it expands");
    println!("  - Staggered phase creates continuous emanation");
    println!("  - Easing::Out gives realistic ripple physics\n");

    // Show with optional recording
    #[cfg(feature = "record")]
    common::show_with_recording(
        &window,
        "crates/core-animation/examples/screenshots/ripple_rings",
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
