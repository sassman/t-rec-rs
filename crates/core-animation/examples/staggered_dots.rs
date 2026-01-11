//! Staggered dots - classic loading indicator with phase-offset animations.
//!
//! Demonstrates using `phase_offset` to create staggered timing across multiple
//! elements. All dots share the same animation parameters but start at different
//! points in the cycle, creating a smooth wave effect.
//!
//! Run with: cargo run -p core-animation --example staggered_dots
//! With recording: cargo run -p core-animation --example staggered_dots --features record

use core_animation::prelude::*;

#[path = "common/mod.rs"]
mod common;

fn main() {
    println!("Staggered Dots - Loading Indicator\n");

    // Clean dark background
    let window = WindowBuilder::new()
        .title("Staggered Dots")
        .size(400.0, 200.0)
        .centered()
        .background_color(Color::rgb(0.08, 0.08, 0.12))
        .build();

    let (width, height) = window.size();
    let center_y = height / 2.0;

    // Dot configuration
    let num_dots = 5;
    let dot_size = 24.0;
    let dot_spacing = 50.0;

    // Calculate starting X to center the dots
    let total_width = (num_dots as f64 - 1.0) * dot_spacing;
    let start_x = (width - total_width) / 2.0;

    // Gradient of colors from purple to cyan
    let colors = [
        Color::rgb(0.6, 0.3, 0.9),  // Purple
        Color::rgb(0.4, 0.4, 0.95), // Blue-purple
        Color::rgb(0.2, 0.6, 0.95), // Blue
        Color::rgb(0.1, 0.8, 0.9),  // Cyan-blue
        Color::rgb(0.0, 0.9, 0.85), // Cyan
    ];

    // Animation timing
    let pulse_duration = 600u64.millis();

    // Create the dot path
    let dot_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(dot_size, dot_size)),
            std::ptr::null(),
        )
    };

    // Create each dot with staggered phase offset
    for (i, &color) in colors.iter().enumerate().take(num_dots) {
        let x = start_x + (i as f64 * dot_spacing);

        // Phase offset creates the wave effect
        // Each dot is offset by 1/num_dots of the cycle
        let phase = i as f64 / num_dots as f64;

        let dot = CAShapeLayerBuilder::new()
            .path(dot_path.clone())
            .fill_color(color)
            .bounds(CGRect::new(CGPoint::ZERO, CGSize::new(dot_size, dot_size)))
            .position(CGPoint::new(x, center_y))
            // Shadow for depth - using builder methods
            .shadow_color(color)
            .shadow_offset(0.0, 2.0)
            .shadow_radius(8.0)
            .shadow_opacity(0.5)
            // Scale animation with staggered phase
            .animate("scale", KeyPath::TransformScale, |a| {
                a.values(0.5, 1.2)
                    .duration(pulse_duration)
                    .easing(Easing::InOut)
                    .autoreverses()
                    .repeat(Repeat::Forever)
                    .phase_offset(phase)
            })
            // Opacity animation synchronized with scale
            .animate("opacity", KeyPath::Opacity, |a| {
                a.values(0.4, 1.0)
                    .duration(pulse_duration)
                    .easing(Easing::InOut)
                    .autoreverses()
                    .repeat(Repeat::Forever)
                    .phase_offset(phase)
            })
            .build();

        window.container().add_sublayer(&dot);
    }

    // Add a second row with opposite phase for visual interest
    let row2_y = center_y + 60.0;
    for i in 0..num_dots {
        let x = start_x + (i as f64 * dot_spacing);
        let color = colors[num_dots - 1 - i]; // Reverse color order

        // Opposite phase offset (start from end)
        let phase = (num_dots - 1 - i) as f64 / num_dots as f64;

        let dot = CAShapeLayerBuilder::new()
            .path(dot_path.clone())
            .fill_color(color.with_alpha(0.7))
            .bounds(CGRect::new(
                CGPoint::ZERO,
                CGSize::new(dot_size * 0.7, dot_size * 0.7),
            ))
            .position(CGPoint::new(x, row2_y))
            // Smaller shadow for secondary row - using builder methods
            .shadow_color(color)
            .shadow_offset(0.0, 1.0)
            .shadow_radius(4.0)
            .shadow_opacity(0.3)
            .animate("scale", KeyPath::TransformScale, |a| {
                a.values(0.6, 1.1)
                    .duration(pulse_duration)
                    .easing(Easing::InOut)
                    .autoreverses()
                    .repeat(Repeat::Forever)
                    .phase_offset(phase)
            })
            .animate("opacity", KeyPath::Opacity, |a| {
                a.values(0.3, 0.8)
                    .duration(pulse_duration)
                    .easing(Easing::InOut)
                    .autoreverses()
                    .repeat(Repeat::Forever)
                    .phase_offset(phase)
            })
            .build();

        window.container().add_sublayer(&dot);
    }

    println!("Watch the staggered dots for 10 seconds...");
    println!("Notice how phase_offset creates the wave effect.\n");

    // Show with optional recording
    #[cfg(feature = "record")]
    common::show_with_recording(
        &window,
        "crates/core-animation/examples/screenshots/staggered_dots",
        10.seconds(),
    );

    #[cfg(not(feature = "record"))]
    window.show_for(10.seconds());

    println!("Done!");
}
