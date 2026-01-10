//! NSWindow Animation POC - Testing NSWindow as an alternative to SkyLight windows.
//!
//! This proof-of-concept demonstrates using NSWindow (via objc2-app-kit) for
//! creating overlay windows with Core Animation, as an alternative to the
//! SkyLight private API.
//!
//! Features demonstrated:
//! 1. Frameless, borderless panel (no decorations)
//! 2. Always-on-top window (above all other windows including fullscreen apps)
//! 3. Semi-transparent window with semi-transparent background
//! 4. GPU-accelerated CABasicAnimation on the root layer
//!
//! Run with: cargo run -p nswindow-poc
//!
//! Expected result: A semi-transparent dark panel with a pulsing cyan circle
//! that floats above all other windows for 10 seconds.

use core_animation::prelude::*;

fn main() {
    println!("NSWindow Animation POC");
    println!("======================\n");
    println!("Testing NSWindow as alternative to SkyLight windows.\n");
    println!("Features:");
    println!("  - Borderless, frameless panel");
    println!("  - Above all windows (using WindowLevel::AboveAll)");
    println!("  - Semi-transparent window background");
    println!("  - GPU-accelerated CABasicAnimation");
    println!("\nWatch for a floating panel with a pulsing circle...\n");

    // Window configuration
    let window_width = 200.0;
    let window_height = 200.0;
    let circle_diameter = 80.0;

    // Build window with layers using fully fluent API
    let window = WindowBuilder::new()
        .title("NSWindow Animation POC")
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
                // Pulsing scale animation
                .animate("pulse", KeyPath::TransformScale, |a| {
                    a.values(0.85, 1.15)
                        .duration(2.seconds())
                        .autoreverses()
                        .repeat(Repeat::Forever)
                })
                // Breathing opacity animation (phase offset creates inverse correlation with scale)
                .animate("breathe", KeyPath::Opacity, |a| {
                    a.values(0.7, 1.0)
                        .duration(2.seconds())
                        .autoreverses()
                        .repeat(Repeat::Forever)
                        .phase_offset(0.5) // Start at midpoint for inverse effect
                })
        })
        .build();

    println!("Window created and visible.");
    println!("Running animation for 10 seconds...\n");

    // Show window for 10 seconds
    window.show_for(10.seconds());

    println!("Done!");
}
