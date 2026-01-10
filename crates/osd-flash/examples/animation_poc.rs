//! POC: Testing CABasicAnimation with SkyLight windows.
//!
//! This POC tests whether CABasicAnimation works with SkyLight windows.
//! The hypothesis is that renderInContext() only captures static snapshots,
//! but let's verify this experimentally.
//!
//! The test:
//! 1. Create a CAShapeLayer with a CABasicAnimation attached
//! 2. Render it to a SkyLight window via renderInContext()
//! 3. Observe if the animation plays (GPU-driven) or if we only see static frames
//!
//! Run with: cargo run -p osd-flash --example animation_poc

use std::time::{Duration, Instant};

use core_animation::prelude::*;

fn main() -> osd_flash::Result<()> {
    use osd_flash::backends::skylight::{SkylightWindowBuilder, SkylightWindowLevel};

    println!("POC: Testing CABasicAnimation with SkyLight windows");
    println!("====================================================");
    println!();
    println!("Hypothesis: CABasicAnimation does NOT work with SkyLight because");
    println!("renderInContext() only captures static snapshots.");
    println!();
    println!("Test: Attach a scale animation to a CAShapeLayer and render it.");
    println!("If the animation works, the circle should pulse smoothly.");
    println!("If not, it will remain static or jump between frames.");
    println!();
    println!("Watch for 10 seconds...");
    println!();

    let size = 200.0;
    let center = size / 2.0;

    // Create SkyLight window
    let frame = osd_flash::geometry::Rect::from_xywh(30.0, 55.0, size, size);
    let window = SkylightWindowBuilder::new()
        .frame(frame)
        .level(SkylightWindowLevel::AboveAll)
        .build()?;

    // Create root layer
    let root_bounds = CGRect::new(CGPoint::ZERO, CGSize::new(size, size));
    let root = CALayerBuilder::new()
        .bounds(root_bounds)
        .position(CGPoint::new(center, center))
        .background_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
        .corner_radius(12.0)
        .build();

    // Create a circle with CABasicAnimation attached
    let circle_size = 60.0;
    let circle_bounds = CGRect::new(CGPoint::ZERO, CGSize::new(circle_size, circle_size));
    let circle_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(circle_size, circle_size)),
            std::ptr::null(),
        )
    };

    // Use the builder with .animate() to attach CABasicAnimation
    let animated_circle = CAShapeLayerBuilder::new()
        .bounds(circle_bounds)
        .position(CGPoint::new(center, center))
        .path(circle_path)
        .fill_color(Color::rgba(0.95, 0.2, 0.2, 1.0))
        // Attach a CABasicAnimation for scale pulsing
        .animate("scale_pulse", KeyPath::TransformScale, |a| {
            a.values(0.85, 1.15)
                .duration(1.seconds())
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        .build();

    root.addSublayer(&animated_circle);

    // Show window
    window.show_visible()?;

    // Render loop - just calling renderInContext repeatedly
    let start = Instant::now();
    let total_duration = Duration::from_secs(10);
    let frame_duration = Duration::from_secs_f64(1.0 / 60.0); // 60 FPS

    while start.elapsed() < total_duration {
        let frame_start = Instant::now();

        // Get context
        let ctx_ptr = window.context_ptr();
        let ctx = unsafe {
            std::ptr::NonNull::new(ctx_ptr.cast::<CGContext>())
                .expect("CGContext pointer must not be null")
                .as_ref()
        };

        // Clear and render
        let clear_rect = CGRect::new(CGPoint::ZERO, CGSize::new(size, size));
        CGContext::clear_rect(Some(ctx), clear_rect);
        root.renderInContext(ctx);
        CGContext::flush(Some(ctx));

        // Frame timing
        let render_time = frame_start.elapsed();
        if render_time < frame_duration {
            unsafe {
                CFRunLoop::run_in_mode(
                    kCFRunLoopDefaultMode,
                    (frame_duration - render_time).as_secs_f64(),
                    false,
                );
            }
        }
    }

    window.hide()?;

    println!();
    println!("Test complete!");
    println!();
    println!("Results:");
    println!("  - If the circle pulsed smoothly: CABasicAnimation WORKS with SkyLight");
    println!("  - If the circle was static: CABasicAnimation does NOT work");
    println!();

    Ok(())
}
