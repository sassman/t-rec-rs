//! WindowBuilder example - creating layer-backed windows with a builder API.
//!
//! This example demonstrates the fully fluent API for creating windows with
//! animated layers inline, from window to layers to animations.
//!
//! Run with: cargo run -p core-animation --example window_builder
//! With recording: cargo run -p core-animation --example window_builder --features record

#[cfg(target_os = "macos")]
use core_animation::prelude::*;

#[cfg(target_os = "macos")]
#[path = "common/mod.rs"]
mod common;

#[cfg(target_os = "macos")]
fn main() {
    println!("WindowBuilder Example - Fluent API\n");

    let window_width = 400.0;
    let window_height = 400.0;
    let circle_diameter = 80.0;

    // Create a window using the fully fluent builder pattern:
    // window -> layers -> animations all in one chain
    let window = WindowBuilder::new()
        .title("Fluent Animation API")
        .size(window_width, window_height)
        .centered()
        .borderless()
        .transparent()
        .level(WindowLevel::AboveAll)
        .corner_radius(20.0)
        .background_color(Color::rgba(0.1, 0.1, 0.15, 0.85))
        .border_color(Color::rgba(0.3, 0.3, 0.35, 0.5))
        .layer("circle", |s| {
            s.circle(circle_diameter)
                .position(CGPoint::new(window_width / 2.0, window_height / 2.0))
                .fill_color(Color::CYAN)
                .animate("pulse", KeyPath::TransformScale, |a| {
                    a.values(0.85, 1.15)
                        .duration(2.seconds())
                        .autoreverses()
                        .repeat(Repeat::Forever)
                })
                .animate("breathe", KeyPath::Opacity, |a| {
                    a.values(0.7, 1.0)
                        .duration(2.seconds())
                        .autoreverses()
                        .repeat(Repeat::Forever)
                        .phase_offset(0.5)
                })
        })
        .build();

    println!("Window size: {:?}", window.size());
    println!("Showing pulsing circle for 10 seconds...\n");

    #[cfg(feature = "record")]
    common::show_with_recording(
        &window,
        "crates/core-animation/examples/screenshots/window_builder",
        10.seconds(),
    );

    #[cfg(not(feature = "record"))]
    window.show_for(10.seconds());

    println!("Done!");
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This example only runs on macOS");
}
