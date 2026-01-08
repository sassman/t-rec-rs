//! Loading spinner - smooth rotating indicator with linear easing.
//!
//! Demonstrates `TransformRotation` animation with `Easing::Linear` for constant
//! rotational speed. Uses multiple elements arranged in a circle to create
//! a professional-looking spinner.
//!
//! Run with: cargo run -p core-animation --example loading_spinner
//! With screenshot: cargo run -p core-animation --example loading_spinner --features screenshot

use core_animation::prelude::*;
use std::f64::consts::PI;

fn main() {
    println!("Loading Spinner - Smooth Rotation\n");

    // Clean, modern dark background
    let window = WindowBuilder::new()
        .title("Loading Spinner")
        .size(300.0, 300.0)
        .centered()
        .background_color(Color::rgb(0.1, 0.1, 0.15))
        .build();

    let (width, height) = window.size();
    let center = CGPoint::new(width / 2.0, height / 2.0);

    // Spinner configuration
    let spinner_radius = 50.0;
    let dot_size = 12.0;
    let num_dots = 8;

    // Modern blue accent color
    let spinner_color = Color::rgb(0.2, 0.6, 1.0);

    // Create a container layer for the spinner that will rotate
    let spinner_container = CALayerBuilder::new()
        .bounds(CGRect::new(
            CGPoint::ZERO,
            CGSize::new(
                spinner_radius * 2.0 + dot_size,
                spinner_radius * 2.0 + dot_size,
            ),
        ))
        .position(center)
        .build();

    // Add rotation animation to the container with LINEAR easing
    let rotation_duration = 1200u64.millis();
    let spin_anim = CABasicAnimationBuilder::new(KeyPath::TransformRotation)
        .values(0.0, PI * 2.0)
        .duration(rotation_duration)
        .easing(Easing::Linear) // KEY: Linear for constant rotation
        .repeat(Repeat::Forever)
        .build();

    spinner_container.addAnimation_forKey(&spin_anim, Some(objc2_foundation::ns_string!("spin")));

    // Create dots arranged in a circle
    let container_center = CGPoint::new(
        spinner_radius + dot_size / 2.0,
        spinner_radius + dot_size / 2.0,
    );

    let dot_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(dot_size, dot_size)),
            std::ptr::null(),
        )
    };

    for i in 0..num_dots {
        let angle = (i as f64 / num_dots as f64) * 2.0 * PI - PI / 2.0;
        let x = container_center.x + spinner_radius * angle.cos();
        let y = container_center.y + spinner_radius * angle.sin();

        // Opacity fades around the circle for a trailing effect
        let opacity = 0.2 + 0.8 * (1.0 - i as f64 / num_dots as f64);
        let scale = 0.6 + 0.4 * (1.0 - i as f64 / num_dots as f64);

        let dot = CAShapeLayerBuilder::new()
            .path(dot_path.clone())
            .fill_color(spinner_color.with_alpha(opacity))
            .bounds(CGRect::new(CGPoint::ZERO, CGSize::new(dot_size, dot_size)))
            .position(CGPoint::new(x, y))
            .opacity(opacity as f32)
            // Scale the dots - largest at the "head" of the spinner
            .scale(scale)
            .build();

        spinner_container.addSublayer(&dot);
    }

    // Create a second spinner rotating opposite direction (inner)
    let inner_radius = 25.0;
    let inner_dot_size = 8.0;
    let inner_num_dots = 6;

    let inner_container = CALayerBuilder::new()
        .bounds(CGRect::new(
            CGPoint::ZERO,
            CGSize::new(
                inner_radius * 2.0 + inner_dot_size,
                inner_radius * 2.0 + inner_dot_size,
            ),
        ))
        .position(center)
        .build();

    // Rotate opposite direction
    let inner_spin_anim = CABasicAnimationBuilder::new(KeyPath::TransformRotation)
        .values(0.0, -PI * 2.0) // Negative for reverse
        .duration(1800u64.millis()) // Slower
        .easing(Easing::Linear)
        .repeat(Repeat::Forever)
        .build();

    // todo: a method like `.add_animation("spin", inner_spin_anim)` should hide the details
    inner_container
        .addAnimation_forKey(&inner_spin_anim, Some(objc2_foundation::ns_string!("spin")));

    let inner_container_center = CGPoint::new(
        inner_radius + inner_dot_size / 2.0,
        inner_radius + inner_dot_size / 2.0,
    );

    let inner_dot_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(inner_dot_size, inner_dot_size)),
            std::ptr::null(),
        )
    };

    for i in 0..inner_num_dots {
        let angle = (i as f64 / inner_num_dots as f64) * 2.0 * PI - PI / 2.0;
        let x = inner_container_center.x + inner_radius * angle.cos();
        let y = inner_container_center.y + inner_radius * angle.sin();
        let opacity = 0.3 + 0.7 * (1.0 - i as f64 / inner_num_dots as f64);

        let dot = CAShapeLayerBuilder::new()
            .path(inner_dot_path.clone())
            .fill_color(spinner_color.with_alpha(opacity * 0.6))
            .bounds(CGRect::new(
                CGPoint::ZERO,
                CGSize::new(inner_dot_size, inner_dot_size),
            ))
            .position(CGPoint::new(x, y))
            .opacity((opacity * 0.6) as f32)
            .build();

        inner_container.addSublayer(&dot);
    }

    // Add a pulsing center dot
    let center_dot_size = 14.0;
    let center_dot_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(center_dot_size, center_dot_size)),
            std::ptr::null(),
        )
    };

    let center_dot = CAShapeLayerBuilder::new()
        .path(center_dot_path)
        .fill_color(spinner_color)
        .bounds(CGRect::new(
            CGPoint::ZERO,
            CGSize::new(center_dot_size, center_dot_size),
        ))
        .position(center)
        // Shadow for glow effect - using builder methods
        .shadow_color(spinner_color)
        .shadow_offset(0.0, 0.0)
        .shadow_radius(8.0)
        .shadow_opacity(0.8)
        .animate("pulse", KeyPath::TransformScale, |a| {
            a.values(0.8, 1.3)
                .duration(800u64.millis())
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        .animate("opacity", KeyPath::Opacity, |a| {
            a.values(0.6, 1.0)
                .duration(800u64.millis())
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        .build();

    // Add layers in order (back to front)
    window.container().add_sublayer(&inner_container);
    window.container().add_sublayer(&spinner_container);
    window.container().add_sublayer(&center_dot);

    println!("Watch the loading spinner for 10 seconds...");
    println!("Notice the constant speed from Easing::Linear.\n");

    // Show with optional screenshot
    #[cfg(feature = "screenshot")]
    {
        use std::path::Path;
        window.show_for_with_screenshot(
            10.seconds(),
            Path::new("crates/core-animation/examples/screenshots/loading_spinner.png"),
        );
    }

    #[cfg(not(feature = "screenshot"))]
    window.show_for(10.seconds());

    println!("Done!");
}
