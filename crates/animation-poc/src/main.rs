//! POC: Testing CABasicAnimation with SkyLight windows.
//!
//! This POC tests whether CABasicAnimation works with SkyLight windows.
//! The hypothesis is that renderInContext() only captures static snapshots,
//! but let's verify this experimentally.
//!
//! The test shows TWO circles side by side:
//! - LEFT (RED): Uses CABasicAnimation (GPU-driven animation)
//! - RIGHT (CYAN): Uses software animation (manually updating properties)
//!
//! If CABasicAnimation works with SkyLight, both circles should pulse.
//! If not, only the RIGHT (cyan) circle will pulse.
//!
//! Run with: cargo run -p animation-poc

use std::ffi::c_void;
use std::time::{Duration, Instant};

use core_animation::prelude::*;
use libloading::{Library, Symbol};

// ============================================================================
// SkyLight bindings (minimal, just what we need)
// ============================================================================

type CGSConnectionID = i32;
type CGSWindowID = u32;

const SKYLIGHT_PATH: &str = "/System/Library/PrivateFrameworks/SkyLight.framework/SkyLight";

// Core Graphics window level constants
const K_CG_MAXIMUM_WINDOW_LEVEL: i32 = i32::MAX;

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
}

/// A minimal SkyLight window for testing.
struct SkyLightWindow {
    connection_id: CGSConnectionID,
    window_id: CGSWindowID,
    context: *mut c_void,
    release_window: unsafe extern "C" fn(CGSConnectionID, CGSWindowID) -> i32,
    order_window: unsafe extern "C" fn(CGSConnectionID, CGSWindowID, i32, CGSWindowID) -> i32,
}

impl SkyLightWindow {
    fn create(x: f64, y: f64, width: f64, height: f64) -> anyhow::Result<Self> {
        // Ensure NSApplication is loaded first
        NSApplication::load();

        let lib = unsafe { Library::new(SKYLIGHT_PATH) }
            .map_err(|e| anyhow::anyhow!("Failed to load SkyLight: {}", e))?;

        unsafe {
            // Load functions
            type SLSMainConnectionIDFn = unsafe extern "C" fn() -> CGSConnectionID;
            type SLSNewWindowFn = unsafe extern "C" fn(
                CGSConnectionID,
                i32,
                f32,
                f32,
                *const c_void,
                *mut CGSWindowID,
            ) -> i32;
            type SLSReleaseWindowFn = unsafe extern "C" fn(CGSConnectionID, CGSWindowID) -> i32;
            type SLSSetWindowLevelFn =
                unsafe extern "C" fn(CGSConnectionID, CGSWindowID, i32) -> i32;
            type SLSOrderWindowFn =
                unsafe extern "C" fn(CGSConnectionID, CGSWindowID, i32, CGSWindowID) -> i32;
            type SLSSetWindowOpacityFn =
                unsafe extern "C" fn(CGSConnectionID, CGSWindowID, bool) -> i32;
            type SLSSetWindowTagsFn =
                unsafe extern "C" fn(CGSConnectionID, CGSWindowID, *const u64, i32) -> i32;
            type CGSNewRegionWithRectFn =
                unsafe extern "C" fn(*const CGRect, *mut *const c_void) -> i32;
            type SLWindowContextCreateFn =
                unsafe extern "C" fn(CGSConnectionID, CGSWindowID, *const c_void) -> *mut c_void;

            let sls_main_connection_id: Symbol<SLSMainConnectionIDFn> =
                lib.get(b"SLSMainConnectionID")?;
            let sls_new_window: Symbol<SLSNewWindowFn> = lib.get(b"SLSNewWindow")?;
            let sls_release_window: Symbol<SLSReleaseWindowFn> = lib.get(b"SLSReleaseWindow")?;
            let sls_set_window_level: Symbol<SLSSetWindowLevelFn> =
                lib.get(b"SLSSetWindowLevel")?;
            let sls_order_window: Symbol<SLSOrderWindowFn> = lib.get(b"SLSOrderWindow")?;
            let sls_set_window_opacity: Symbol<SLSSetWindowOpacityFn> =
                lib.get(b"SLSSetWindowOpacity")?;
            let sls_set_window_tags: Symbol<SLSSetWindowTagsFn> = lib.get(b"SLSSetWindowTags")?;
            let cgs_new_region_with_rect: Symbol<CGSNewRegionWithRectFn> =
                lib.get(b"CGSNewRegionWithRect")?;
            let sl_window_context_create: Symbol<SLWindowContextCreateFn> =
                lib.get(b"SLWindowContextCreate")?;

            // Get connection
            let cid = sls_main_connection_id();
            if cid == 0 {
                anyhow::bail!("Failed to get SkyLight connection");
            }

            // Create frame rect
            let frame = CGRect::new(CGPoint::new(x, y), CGSize::new(width, height));

            // Create region
            let mut region: *const c_void = std::ptr::null();
            let region_result = cgs_new_region_with_rect(&frame, &mut region);
            if region_result != 0 || region.is_null() {
                anyhow::bail!("Failed to create window region");
            }

            // Create window
            let mut wid: CGSWindowID = 0;
            let result = sls_new_window(cid, 2, x as f32, y as f32, region, &mut wid);
            CFRelease(region);

            if result != 0 || wid == 0 {
                anyhow::bail!("Failed to create SkyLight window");
            }

            // Configure window
            sls_set_window_opacity(cid, wid, false);
            sls_set_window_level(cid, wid, K_CG_MAXIMUM_WINDOW_LEVEL);

            // Set sticky tags
            let tags: u64 = (1 << 0) | (1 << 11);
            sls_set_window_tags(cid, wid, &tags, 64);

            // Create context
            let ctx = sl_window_context_create(cid, wid, std::ptr::null());
            if ctx.is_null() {
                sls_release_window(cid, wid);
                anyhow::bail!("Failed to create window context");
            }

            Ok(Self {
                connection_id: cid,
                window_id: wid,
                context: ctx,
                release_window: *sls_release_window,
                order_window: *sls_order_window,
            })
        }
    }

    fn show(&self) {
        unsafe {
            (self.order_window)(self.connection_id, self.window_id, 1, 0);
        }
    }

    fn hide(&self) {
        unsafe {
            (self.order_window)(self.connection_id, self.window_id, 0, 0);
        }
    }

    fn context_ptr(&self) -> *mut c_void {
        self.context
    }
}

impl Drop for SkyLightWindow {
    fn drop(&mut self) {
        unsafe {
            (self.release_window)(self.connection_id, self.window_id);
        }
    }
}

// ============================================================================
// Animation helpers
// ============================================================================

/// Easing function: EaseInOut (smooth S-curve)
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

/// Calculate animation progress (0.0 to 1.0) with autoreverses
fn calc_progress(elapsed: Duration, duration_secs: f64) -> f64 {
    let cycle = duration_secs * 2.0; // forward + backward
    let t = elapsed.as_secs_f64() % cycle;
    let progress = if t < duration_secs {
        t / duration_secs
    } else {
        1.0 - (t - duration_secs) / duration_secs
    };
    ease_in_out(progress)
}

// ============================================================================
// Main
// ============================================================================

fn main() -> anyhow::Result<()> {
    println!("POC: Testing CABasicAnimation with SkyLight windows");
    println!("====================================================");
    println!();
    println!("This test shows TWO circles side by side:");
    println!("  - LEFT (RED): Uses CABasicAnimation (GPU-driven)");
    println!("  - RIGHT (CYAN): Uses software animation (manual update)");
    println!();
    println!("If CABasicAnimation works, BOTH circles should pulse.");
    println!("If not, only the RIGHT (cyan) circle will pulse.");
    println!();
    println!("Watch for 10 seconds...");
    println!();

    let width = 300.0;
    let height = 150.0;

    // Create SkyLight window
    let window = SkyLightWindow::create(30.0, 55.0, width, height)?;

    // Create root layer
    let root_bounds = CGRect::new(CGPoint::ZERO, CGSize::new(width, height));
    let root = CALayerBuilder::new()
        .bounds(root_bounds)
        .position(CGPoint::new(width / 2.0, height / 2.0))
        .background_color(Color::rgba(0.1, 0.1, 0.1, 0.9))
        .corner_radius(12.0)
        .build();

    // Animation parameters
    let circle_size = 50.0;
    let anim_duration = 1.0; // seconds
    let scale_from = 0.85;
    let scale_to = 1.15;

    // =========================================================================
    // LEFT CIRCLE: CABasicAnimation (GPU-driven)
    // =========================================================================
    let left_x = width * 0.25;
    let left_bounds = CGRect::new(CGPoint::ZERO, CGSize::new(circle_size, circle_size));
    let left_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(circle_size, circle_size)),
            std::ptr::null(),
        )
    };

    let left_circle = CAShapeLayerBuilder::new()
        .bounds(left_bounds)
        .position(CGPoint::new(left_x, height / 2.0))
        .path(left_path)
        .fill_color(Color::rgba(0.95, 0.2, 0.2, 1.0)) // RED
        // Attach CABasicAnimation
        .animate("scale_pulse", KeyPath::TransformScale, |a| {
            a.values(scale_from, scale_to)
                .duration(Duration::from_secs_f64(anim_duration))
                .easing(Easing::InOut)
                .autoreverses()
                .repeat(Repeat::Forever)
        })
        .build();

    root.addSublayer(&left_circle);

    // =========================================================================
    // RIGHT CIRCLE: Software animation (manual property update)
    // =========================================================================
    let right_x = width * 0.75;
    let right_bounds = CGRect::new(CGPoint::ZERO, CGSize::new(circle_size, circle_size));
    let right_path = unsafe {
        CGPath::with_ellipse_in_rect(
            CGRect::new(CGPoint::ZERO, CGSize::new(circle_size, circle_size)),
            std::ptr::null(),
        )
    };

    let right_circle = CAShapeLayerBuilder::new()
        .bounds(right_bounds)
        .position(CGPoint::new(right_x, height / 2.0))
        .path(right_path)
        .fill_color(Color::rgba(0.0, 0.9, 1.0, 1.0)) // CYAN
        // NO CABasicAnimation - we'll update manually
        .build();

    root.addSublayer(&right_circle);

    // Show window
    window.show();

    // Render loop
    let start = Instant::now();
    let total_duration = Duration::from_secs(10);
    let frame_duration = Duration::from_secs_f64(1.0 / 60.0); // 60 FPS

    while start.elapsed() < total_duration {
        let frame_start = Instant::now();
        let elapsed = start.elapsed();

        // =====================================================================
        // Software animation: manually update RIGHT circle's scale
        // =====================================================================
        let progress = calc_progress(elapsed, anim_duration);
        let scale = lerp(scale_from, scale_to, progress);
        let transform = CATransform3D::new_scale(scale, scale, 1.0);
        right_circle.setTransform(transform);

        // Get context
        let ctx_ptr = window.context_ptr();
        let ctx = unsafe {
            std::ptr::NonNull::new(ctx_ptr.cast::<CGContext>())
                .expect("CGContext pointer must not be null")
                .as_ref()
        };

        // Clear and render
        let clear_rect = CGRect::new(CGPoint::ZERO, CGSize::new(width, height));
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

    window.hide();

    println!();
    println!("Test complete!");
    println!();
    println!("Results:");
    println!("  - If BOTH circles pulsed: CABasicAnimation WORKS with SkyLight!");
    println!("  - If only the RIGHT (cyan) pulsed: CABasicAnimation does NOT work");
    println!("    (renderInContext captures static snapshots, ignoring animations)");
    println!();

    Ok(())
}
