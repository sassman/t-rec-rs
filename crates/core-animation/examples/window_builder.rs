//! WindowBuilder example - creating layer-backed windows with a builder API.
//!
//! Run with: cargo run -p core-animation --example window_builder
//! With recording: cargo run -p core-animation --example window_builder --features record

use core_animation::prelude::*;

#[path = "common/mod.rs"]
mod common;

fn main() {
    println!("WindowBuilder Example\n");

    // Create a window using the builder pattern
    let window = WindowBuilder::new()
        .title("WindowBuilder Example")
        .size(640.0, 480.0)
        .centered()
        // Screen::Main is the default and could be omitted - shown here to illustrate the API.
        // Use Screen::Index(n) to target a specific display.
        .on_screen(Screen::Main)
        .background_color(Color::rgb(0.1, 0.1, 0.2))
        .build();

    // Add a simple shape to the window's root layer
    let circle_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(100.0, 100.0)),
            std::ptr::null(),
        )
    };

    let circle = CAShapeLayerBuilder::new()
        .path(circle_path)
        .fill_color(Color::CYAN)
        .bounds(CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(100.0, 100.0)))
        .position(CGPoint::new(320.0, 240.0))
        .build();

    window.container().add_sublayer(&circle);

    println!("Window size: {:?}", window.size());
    println!("Showing for 5 seconds...\n");

    #[cfg(feature = "record")]
    common::show_with_recording(
        &window,
        "crates/core-animation/examples/screenshots/window_builder",
        5.seconds(),
    );

    #[cfg(not(feature = "record"))]
    window.show_for(5.seconds());

    println!("Done!");
}
