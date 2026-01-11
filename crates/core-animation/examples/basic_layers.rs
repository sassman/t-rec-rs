//! Basic layer example with animated shapes.
//!
//! Demonstrates CALayer and CAShapeLayer with the builder APIs and
//! GPU-accelerated animations using `.animate()`.
//!
//! Run with: cargo run -p core-animation --example basic_layers
//! With recording: cargo run -p core-animation --example basic_layers --features record

#[cfg(target_os = "macos")]
use core_animation::prelude::*;

#[cfg(target_os = "macos")]
#[path = "common/mod.rs"]
mod common;

#[cfg(target_os = "macos")]
fn main() {
    println!("Basic Layers - Animated Shapes\n");

    let window = WindowBuilder::new()
        .title("Basic Layers")
        .size(300.0, 300.0)
        .centered()
        .background_color(Color::rgb(0.12, 0.12, 0.18))
        .build();

    // Circle path (reused for both circles)
    let circle_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(80.0, 80.0)),
            std::ptr::null(),
        )
    };

    // Red circle (left) with ~3Hz pulse
    let red_circle = CAShapeLayerBuilder::new()
        .path(circle_path.clone())
        .fill_color(Color::rgb(0.95, 0.3, 0.3))
        .bounds(CGRect::new(CGPoint::ZERO, CGSize::new(80.0, 80.0)))
        .position(CGPoint::new(80.0, 170.0))
        .animate("pulse", KeyPath::TransformScale, |a| {
            a.values(0.85, 1.15)
                .duration(670.millis())
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        .build();

    // Blue circle (right) with ~3Hz pulse, out of phase
    let blue_circle = CAShapeLayerBuilder::new()
        .path(circle_path)
        .fill_color(Color::rgb(0.3, 0.5, 0.95))
        .bounds(CGRect::new(CGPoint::ZERO, CGSize::new(80.0, 80.0)))
        .position(CGPoint::new(220.0, 170.0))
        .animate("pulse", KeyPath::TransformScale, |a| {
            a.values(0.85, 1.15)
                .duration(670.millis())
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
                .phase_offset(0.5)
        })
        .build();

    // Green rounded rectangle (bottom) with ~2.5Hz pulse
    let rect_path = unsafe {
        CGPath::with_rounded_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(160.0, 60.0)),
            12.0,
            12.0,
            std::ptr::null(),
        )
    };

    let green_rect = CAShapeLayerBuilder::new()
        .path(rect_path)
        .fill_color(Color::rgb(0.3, 0.9, 0.4).with_alpha(0.9))
        .bounds(CGRect::new(CGPoint::ZERO, CGSize::new(160.0, 60.0)))
        .position(CGPoint::new(150.0, 70.0))
        .animate("pulse", KeyPath::TransformScale, |a| {
            a.values(0.90, 1.10)
                .duration(800.millis())
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        .build();

    // Add shapes to window
    window.container().add_sublayer(&red_circle);
    window.container().add_sublayer(&blue_circle);
    window.container().add_sublayer(&green_rect);

    println!("Pulsing shapes for 5 seconds...\n");

    // Show with optional recording
    #[cfg(feature = "record")]
    common::show_with_recording(
        &window,
        "crates/core-animation/examples/screenshots/basic_layers",
        5.seconds(),
    );

    #[cfg(not(feature = "record"))]
    window.show_for(5.seconds());

    println!("Done!");
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This example only runs on macOS");
}
