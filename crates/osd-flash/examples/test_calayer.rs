//! Test CALayer rendering to SkyLight CGContext with animation.
//!
//! Replicates the recording_indicator_pulse animation using CALayer.
//! Tests whether CALayer.renderInContext: eliminates flickering.
//!
//! Run with: cargo run -p osd-flash --example test_calayer

use std::ffi::c_void;
use std::time::{Duration, Instant};

use objc2::encode::{Encode, Encoding};
use objc2::runtime::AnyObject;
use objc2::{class, msg_send};

/// Keyframe data for animation
struct Keyframe {
    progress: f64,
    scale: f64,
    glow_alpha: f64,
}

/// Easing function (EaseInOut: smooth S-curve)
/// Slow at start and end, fast in middle - feels natural for pulsing
fn ease_in_out(t: f64) -> f64 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
    }
}

/// Linear interpolation
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

/// Find surrounding keyframes and interpolate
fn interpolate_keyframes(keyframes: &[Keyframe], progress: f64) -> (f64, f64) {
    // Find the two keyframes we're between
    let mut from_idx = 0;
    let mut to_idx = 0;

    for (i, kf) in keyframes.iter().enumerate() {
        if kf.progress <= progress {
            from_idx = i;
        }
        if kf.progress >= progress && to_idx == 0 {
            to_idx = i;
            break;
        }
    }

    // Handle edge case where we're at or past the last keyframe
    if to_idx == 0 {
        to_idx = keyframes.len() - 1;
    }

    let from = &keyframes[from_idx];
    let to = &keyframes[to_idx];

    // Calculate segment progress
    let segment_t = if (to.progress - from.progress).abs() < 0.0001 {
        0.0
    } else {
        (progress - from.progress) / (to.progress - from.progress)
    };

    // Apply easing
    let eased_t = ease_in_out(segment_t);

    // Interpolate values
    let scale = lerp(from.scale, to.scale, eased_t);
    let glow_alpha = lerp(from.glow_alpha, to.glow_alpha, eased_t);

    (scale, glow_alpha)
}

// Core Graphics types with Encode implementations for objc2
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CGPoint {
    x: f64,
    y: f64,
}

unsafe impl Encode for CGPoint {
    const ENCODING: Encoding = Encoding::Struct("CGPoint", &[Encoding::Double, Encoding::Double]);
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CGSize {
    width: f64,
    height: f64,
}

unsafe impl Encode for CGSize {
    const ENCODING: Encoding = Encoding::Struct("CGSize", &[Encoding::Double, Encoding::Double]);
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CGRect {
    origin: CGPoint,
    size: CGSize,
}

unsafe impl Encode for CGRect {
    const ENCODING: Encoding = Encoding::Struct("CGRect", &[CGPoint::ENCODING, CGSize::ENCODING]);
}

// CATransform3D for Core Animation transforms
#[repr(C)]
#[derive(Copy, Clone)]
struct CATransform3D {
    m11: f64,
    m12: f64,
    m13: f64,
    m14: f64,
    m21: f64,
    m22: f64,
    m23: f64,
    m24: f64,
    m31: f64,
    m32: f64,
    m33: f64,
    m34: f64,
    m41: f64,
    m42: f64,
    m43: f64,
    m44: f64,
}

unsafe impl Encode for CATransform3D {
    const ENCODING: Encoding = Encoding::Struct(
        "CATransform3D",
        &[
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
            Encoding::Double,
        ],
    );
}

// Core Graphics bindings
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGColorCreateGenericRGB(r: f64, g: f64, b: f64, a: f64) -> *mut c_void;
    fn CGColorRelease(color: *mut c_void);
    fn CGPathCreateWithEllipseInRect(rect: CGRect, transform: *const c_void) -> *mut c_void;
    fn CGPathRelease(path: *mut c_void);
    fn CGContextClearRect(ctx: *mut c_void, rect: CGRect);
    fn CGContextFlush(ctx: *mut c_void);
}

// CFRunLoop
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRunLoopRunInMode(mode: *const c_void, seconds: f64, return_after: bool) -> i32;
    static kCFRunLoopDefaultMode: *const c_void;
}

/// Wrapper for CALayer operations
struct CALayerTree {
    root: *mut AnyObject,
    shape_layer: *mut AnyObject,
    glow_layer: *mut AnyObject,
    highlight_layer: *mut AnyObject,
}

impl CALayerTree {
    fn new(width: f64, height: f64) -> Self {
        unsafe {
            let center = width / 2.0;

            // Create root layer
            let ca_layer_class = class!(CALayer);
            let root: *mut AnyObject = msg_send![ca_layer_class, layer];

            // Set bounds
            let bounds = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize { width, height },
            };
            let _: () = msg_send![root, setBounds: bounds];

            // Set position (anchor at center)
            let position = CGPoint {
                x: width / 2.0,
                y: height / 2.0,
            };
            let _: () = msg_send![root, setPosition: position];

            // Set background color
            let bg_color = CGColorCreateGenericRGB(0.1, 0.1, 0.1, 0.88);
            let _: () = msg_send![root, setBackgroundColor: bg_color];
            CGColorRelease(bg_color);

            // Set corner radius
            let _: () = msg_send![root, setCornerRadius: 14.0f64];

            // === Glow layer (behind the dot, animated) ===
            let ca_shape_layer_class = class!(CAShapeLayer);
            let glow_layer: *mut AnyObject = msg_send![ca_shape_layer_class, layer];

            let glow_bounds = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize { width, height },
            };
            let _: () = msg_send![glow_layer, setBounds: glow_bounds];
            let glow_position = CGPoint {
                x: center,
                y: center,
            };
            let _: () = msg_send![glow_layer, setPosition: glow_position];

            // Glow circle (size * 0.26 radius from keyframes)
            let glow_radius = width * 0.26;
            let glow_rect = CGRect {
                origin: CGPoint {
                    x: center - glow_radius,
                    y: center - glow_radius,
                },
                size: CGSize {
                    width: glow_radius * 2.0,
                    height: glow_radius * 2.0,
                },
            };
            let glow_path = CGPathCreateWithEllipseInRect(glow_rect, std::ptr::null());
            let _: () = msg_send![glow_layer, setPath: glow_path];
            CGPathRelease(glow_path);

            // Initial glow color (alpha 0.4 at keyframe 0.0)
            let glow_color = CGColorCreateGenericRGB(1.0, 0.2, 0.2, 0.4);
            let _: () = msg_send![glow_layer, setFillColor: glow_color];
            CGColorRelease(glow_color);

            let _: () = msg_send![root, addSublayer: glow_layer];

            // === Main recording dot (shape_layer) ===
            let shape_layer: *mut AnyObject = msg_send![ca_shape_layer_class, layer];

            let shape_bounds = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize { width, height },
            };
            let _: () = msg_send![shape_layer, setBounds: shape_bounds];
            let shape_position = CGPoint {
                x: center,
                y: center,
            };
            let _: () = msg_send![shape_layer, setPosition: shape_position];

            // Main dot (size * 0.18 radius)
            let dot_radius = width * 0.18;
            let dot_rect = CGRect {
                origin: CGPoint {
                    x: center - dot_radius,
                    y: center - dot_radius,
                },
                size: CGSize {
                    width: dot_radius * 2.0,
                    height: dot_radius * 2.0,
                },
            };
            let dot_path = CGPathCreateWithEllipseInRect(dot_rect, std::ptr::null());
            let _: () = msg_send![shape_layer, setPath: dot_path];
            CGPathRelease(dot_path);

            let fill_color = CGColorCreateGenericRGB(0.95, 0.12, 0.12, 1.0);
            let _: () = msg_send![shape_layer, setFillColor: fill_color];
            CGColorRelease(fill_color);

            let _: () = msg_send![root, addSublayer: shape_layer];

            // === Highlight layer (small white dot for 3D effect) ===
            let highlight_layer: *mut AnyObject = msg_send![ca_shape_layer_class, layer];

            let highlight_bounds = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize { width, height },
            };
            let _: () = msg_send![highlight_layer, setBounds: highlight_bounds];
            let highlight_position = CGPoint {
                x: center,
                y: center,
            };
            let _: () = msg_send![highlight_layer, setPosition: highlight_position];

            // Highlight (offset from center, size * 0.04 radius)
            let highlight_radius = width * 0.04;
            let highlight_offset = width * 0.05;
            let highlight_rect = CGRect {
                origin: CGPoint {
                    x: center - highlight_offset - highlight_radius,
                    y: center - highlight_offset - highlight_radius,
                },
                size: CGSize {
                    width: highlight_radius * 2.0,
                    height: highlight_radius * 2.0,
                },
            };
            let highlight_path = CGPathCreateWithEllipseInRect(highlight_rect, std::ptr::null());
            let _: () = msg_send![highlight_layer, setPath: highlight_path];
            CGPathRelease(highlight_path);

            let highlight_color = CGColorCreateGenericRGB(1.0, 0.5, 0.5, 0.5);
            let _: () = msg_send![highlight_layer, setFillColor: highlight_color];
            CGColorRelease(highlight_color);

            let _: () = msg_send![root, addSublayer: highlight_layer];

            Self {
                root,
                shape_layer,
                glow_layer,
                highlight_layer,
            }
        }
    }

    /// Render the layer tree to a CGContext
    fn render_to_context(&self, ctx: *mut c_void) {
        unsafe {
            let _: () = msg_send![self.root, renderInContext: ctx];
        }
    }

    /// Update the scale of animated layers (glow, shape, highlight)
    fn set_scale(&self, scale: f64) {
        unsafe {
            let transform = CATransform3D {
                m11: scale,
                m12: 0.0,
                m13: 0.0,
                m14: 0.0,
                m21: 0.0,
                m22: scale,
                m23: 0.0,
                m24: 0.0,
                m31: 0.0,
                m32: 0.0,
                m33: scale,
                m34: 0.0,
                m41: 0.0,
                m42: 0.0,
                m43: 0.0,
                m44: 1.0,
            };

            // Apply scale to all animated layers
            let _: () = msg_send![self.glow_layer, setTransform: transform];
            let _: () = msg_send![self.shape_layer, setTransform: transform];
            let _: () = msg_send![self.highlight_layer, setTransform: transform];
        }
    }

    /// Update the glow circle's opacity
    fn set_glow_alpha(&self, alpha: f64) {
        unsafe {
            // Update fill color with new alpha
            let glow_color = CGColorCreateGenericRGB(1.0, 0.2, 0.2, alpha);
            let _: () = msg_send![self.glow_layer, setFillColor: glow_color];
            CGColorRelease(glow_color);
        }
    }

    /// Update animation state for a given progress (0.0 to 1.0)
    fn update_animation(&self, scale: f64, glow_alpha: f64) {
        self.set_scale(scale);
        self.set_glow_alpha(glow_alpha);
    }
}

impl Drop for CALayerTree {
    fn drop(&mut self) {
        // Layers are autoreleased, but we should release our references
        // In practice, the autorelease pool will handle this
    }
}

fn main() -> osd_flash::Result<()> {
    use osd_flash::backends::skylight::{SkylightWindowBuilder, SkylightWindowLevel};

    let size = 80.0;

    println!("Testing CALayer.renderInContext: with pulse animation...");
    println!("Replicating recording_indicator_pulse keyframes.");
    println!("Watch for flickering during the animation.");

    // Define keyframes - smoother breathing pulse
    // Duration: 2 seconds, Easing: EaseInOut
    // Gentler scale range (0.95 to 1.08) for natural feel
    let keyframes = [
        Keyframe {
            progress: 0.0,
            scale: 0.95,
            glow_alpha: 0.25,
        },
        Keyframe {
            progress: 0.5,
            scale: 1.08,
            glow_alpha: 0.55,
        },
        Keyframe {
            progress: 1.0,
            scale: 0.95,
            glow_alpha: 0.25,
        },
    ];
    let animation_duration = Duration::from_secs(2);

    // Create SkyLight window directly
    let frame = osd_flash::geometry::Rect::from_xywh(30.0, 55.0, size, size);
    let window = SkylightWindowBuilder::new()
        .frame(frame)
        .level(SkylightWindowLevel::AboveAll)
        .build()?;

    // Create CALayer tree
    let layer_tree = CALayerTree::new(size, size);

    // Show window
    window.show_visible()?;

    // Animation loop
    let start = Instant::now();
    let total_duration = Duration::from_secs(10);
    let frame_duration = Duration::from_secs_f64(1.0 / 60.0);

    while start.elapsed() < total_duration {
        let frame_start = Instant::now();

        // Calculate animation progress (0.0 to 1.0, looping)
        let elapsed = start.elapsed();
        let cycle_progress = (elapsed.as_secs_f64() % animation_duration.as_secs_f64())
            / animation_duration.as_secs_f64();

        // Interpolate keyframes
        let (scale, glow_alpha) = interpolate_keyframes(&keyframes, cycle_progress);

        // Update layer properties
        layer_tree.update_animation(scale, glow_alpha);

        // Clear and render
        let ctx = window.context_ptr();
        unsafe {
            let clear_rect = CGRect {
                origin: CGPoint { x: 0.0, y: 0.0 },
                size: CGSize {
                    width: size,
                    height: size,
                },
            };
            CGContextClearRect(ctx, clear_rect);
        }

        // Render layer tree to context
        layer_tree.render_to_context(ctx);

        // Flush
        unsafe {
            CGContextFlush(ctx);
        }

        // Frame timing
        let render_time = frame_start.elapsed();
        if render_time < frame_duration {
            unsafe {
                CFRunLoopRunInMode(
                    kCFRunLoopDefaultMode,
                    (frame_duration - render_time).as_secs_f64(),
                    false,
                );
            }
        }
    }

    window.hide()?;
    println!("Done!");

    Ok(())
}
