//! Pulsing Recording Indicator - POC using Core Animation
//!
//! A visually polished recording indicator with multiple animated layers
//! creating a heartbeat-style pulse effect. This demonstrates the power of
//! Core Animation's GPU-accelerated animations.
//!
//! # Visual Design
//!
//! The indicator consists of five carefully orchestrated layers:
//!
//! 1. **Outer Glow Ring** - A soft, expanding ring that pulses outward
//! 2. **Shadow Pulse** - Colored shadow that intensifies with the beat
//! 3. **Inner Glow** - Semi-transparent red fill behind the main dot
//! 4. **Main Recording Dot** - The primary red circle with scale animation
//! 5. **Highlight Reflection** - A small white dot for 3D depth
//!
//! # Animation Philosophy
//!
//! The heartbeat effect is achieved through:
//! - Asymmetric timing (quick expansion, slower contraction)
//! - Phase offsets between layers for organic feel
//! - Coordinated opacity and scale changes
//! - Shadow radius animation for dramatic glow effect
//!
//! Run with: cargo run -p pulsing-poc

use core_animation::prelude::*;

fn main() {
    println!("Pulsing Recording Indicator - Core Animation POC\n");
    println!("Visual layers (bottom to top):");
    println!("  1. Outer glow ring (expanding pulse)");
    println!("  2. Shadow/glow pulse (shadow animation)");
    println!("  3. Inner glow (semi-transparent red)");
    println!("  4. Main recording dot (heartbeat scale)");
    println!("  5. Highlight reflection (3D depth)\n");

    // ========================================================================
    // Configuration
    // ========================================================================

    // Window dimensions - compact overlay panel
    let window_size = 100.0;
    let center = CGPoint::new(window_size / 2.0, window_size / 2.0);

    // Layer sizes (relative to center)
    let dot_diameter = 32.0; // Main recording dot
    let inner_glow_diameter = 44.0; // Soft glow behind dot
    let outer_ring_diameter = 56.0; // Expanding ring effect
    let highlight_diameter = 8.0; // Small reflection highlight

    // Colors - rich, deep red palette for recording
    let recording_red = Color::rgb(0.95, 0.15, 0.15);
    let deep_red = Color::rgb(0.85, 0.1, 0.1);
    let glow_red = Color::rgba(1.0, 0.2, 0.2, 0.35);
    let ring_red = Color::rgba(1.0, 0.25, 0.25, 0.5);
    let highlight_white = Color::rgba(1.0, 0.95, 0.95, 0.7);

    // Animation timing - heartbeat feel
    // A heartbeat has two quick beats followed by a pause
    // We simulate this with asymmetric timing
    let pulse_duration = 800u64.millis(); // Fast, energetic pulse
    let ring_duration = 1200u64.millis(); // Slower ring expansion

    // ========================================================================
    // Build the window with all layers
    // ========================================================================

    let window = WindowBuilder::new()
        .title("Recording")
        .size(window_size, window_size)
        .centered()
        .transparent()
        .borderless()
        .level(WindowLevel::AboveAll)
        .corner_radius(16.0)
        .background_color(Color::rgba(0.06, 0.06, 0.08, 0.92))
        .border_color(Color::rgba(0.3, 0.3, 0.35, 0.4))
        // Layer 1: Outer glow ring - expands outward with fade
        // This creates a "ripple" effect emanating from the recording dot
        .layer("outer_ring", |s| {
            s.circle(outer_ring_diameter)
                .position(center)
                .fill_color(Color::TRANSPARENT)
                .stroke_color(ring_red)
                .line_width(2.5)
                .opacity(0.6)
                // Scale animation: ring expands outward
                .animate("expand", KeyPath::TransformScale, |a| {
                    a.values(0.8, 1.3)
                        .duration(ring_duration)
                        .easing(Easing::Out) // Quick start, slow end (natural decay)
                        .autoreverses()
                        .repeat(Repeat::Forever)
                })
                // Opacity fades as ring expands
                .animate("fade", KeyPath::Opacity, |a| {
                    a.values(0.7, 0.15)
                        .duration(ring_duration)
                        .easing(Easing::Out)
                        .autoreverses()
                        .repeat(Repeat::Forever)
                })
        })
        // Layer 2: Inner glow - soft red fill behind main dot
        // Provides depth and enhances the glow effect
        .layer("inner_glow", |s| {
            s.circle(inner_glow_diameter)
                .position(center)
                .fill_color(glow_red)
                .opacity(0.7)
                // Shadow for extra glow effect
                .shadow_color(recording_red)
                .shadow_offset(0.0, 0.0)
                .shadow_radius(12.0)
                .shadow_opacity(0.6)
                // Subtle scale pulse, offset from main dot
                .animate("pulse", KeyPath::TransformScale, |a| {
                    a.values(0.9, 1.08)
                        .duration(pulse_duration)
                        .easing(Easing::InOut)
                        .autoreverses()
                        .repeat(Repeat::Forever)
                        .phase_offset(0.15) // Slightly trails the main dot
                })
                // Shadow radius pulses for glow intensity
                .animate("glow", KeyPath::ShadowRadius, |a| {
                    a.values(10.0, 22.0)
                        .duration(pulse_duration)
                        .easing(Easing::InOut)
                        .autoreverses()
                        .repeat(Repeat::Forever)
                })
        })
        // Layer 3: Main recording dot - the hero element
        // This is the primary visual with the heartbeat pulse
        .layer("main_dot", |s| {
            s.circle(dot_diameter)
                .position(center)
                .fill_color(recording_red)
                // Add a subtle shadow for depth
                .shadow_color(deep_red)
                .shadow_offset(0.0, 2.0)
                .shadow_radius(6.0)
                .shadow_opacity(0.4)
                // Heartbeat-style scale animation
                // The range 0.88 to 1.12 gives a visible but not jarring pulse
                .animate("heartbeat", KeyPath::TransformScale, |a| {
                    a.values(0.88, 1.12)
                        .duration(pulse_duration)
                        .easing(Easing::InOut)
                        .autoreverses()
                        .repeat(Repeat::Forever)
                })
                // Subtle opacity variation adds life
                .animate("breathe", KeyPath::Opacity, |a| {
                    a.values(1.0, 0.85)
                        .duration(pulse_duration)
                        .easing(Easing::InOut)
                        .autoreverses()
                        .repeat(Repeat::Forever)
                        .phase_offset(0.25) // Offset from scale for organic feel
                })
        })
        // Layer 4: Highlight reflection - adds 3D depth
        // Positioned in upper-left quadrant of the main dot
        .layer("highlight", |s| {
            let highlight_offset = dot_diameter * 0.18;
            s.circle(highlight_diameter)
                .position(CGPoint::new(
                    center.x - highlight_offset,
                    center.y + highlight_offset, // Note: Core Animation Y is up
                ))
                .fill_color(highlight_white)
                .opacity(0.75)
                // Highlight pulses opposite to main dot for depth effect
                .animate("pulse", KeyPath::Opacity, |a| {
                    a.values(0.8, 0.4)
                        .duration(pulse_duration)
                        .easing(Easing::InOut)
                        .autoreverses()
                        .repeat(Repeat::Forever)
                        .phase_offset(0.5) // Opposite phase to main dot opacity
                })
                // Scale slightly with main dot
                .animate("scale", KeyPath::TransformScale, |a| {
                    a.values(0.9, 1.1)
                        .duration(pulse_duration)
                        .easing(Easing::InOut)
                        .autoreverses()
                        .repeat(Repeat::Forever)
                })
        })
        .build();

    // ========================================================================
    // Show the indicator
    // ========================================================================

    println!("Animation running for 15 seconds...");
    println!("Features demonstrated:");
    println!("  - GPU-accelerated CABasicAnimation");
    println!("  - Multiple coordinated layer animations");
    println!("  - Shadow radius/opacity animation for glow");
    println!("  - Phase offsets for organic timing");
    println!("  - Fluent WindowBuilder API with .layer()");
    println!();

    window.show_for(15.seconds());

    println!("Done!");
}
