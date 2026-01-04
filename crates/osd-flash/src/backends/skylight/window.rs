//! SkyLight private API bindings for macOS overlay windows.
//!
//! SkyLight is a private Apple framework used by window managers like yabai and JankyBorders.
//! It allows creating overlay windows that appear above all other content.

use std::ffi::c_void;
use std::sync::OnceLock;

use core_graphics::geometry::CGRect;

use libloading::{Library, Symbol};

use super::canvas::SkylightCanvas;
// Import geometry extensions for CG type conversions
#[allow(unused_imports)]
use super::geometry_ext;
use crate::geometry::{Rect, Size};
use crate::icon::Icon;
use crate::Drawable;

// Private API types
type CGSConnectionID = i32;
type CGSWindowID = u32;

/// Window level determining the z-order of the overlay window.
///
/// Controls where the window appears in the window stack relative to other windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowLevel {
    /// Normal window level, appears with regular application windows.
    Normal,
    /// Floating window level, appears above normal windows but below screen savers.
    Floating,
    /// Modal panel level, appears above floating windows.
    ModalPanel,
    /// Above all other windows, including fullscreen apps and the Dock.
    /// This is the maximum possible z-index (`i32::MAX`).
    #[default]
    AboveAll,
    /// Custom window level value.
    Custom(i32),
}

impl WindowLevel {
    /// Convert to the raw Core Graphics window level value.
    pub fn to_cg_level(self) -> i32 {
        use super::cg_patches::*;
        match self {
            WindowLevel::Normal => kCGMinimumWindowLevel,
            WindowLevel::Floating => kCGFloatingWindowLevel,
            WindowLevel::ModalPanel => kCGModalPanelWindowLevel,
            WindowLevel::AboveAll => kCGMaximumWindowLevel,
            WindowLevel::Custom(level) => level.clamp(kCGMinimumWindowLevel, kCGMaximumWindowLevel),
        }
    }
}

// SkyLight framework path
const SKYLIGHT_PATH: &str = "/System/Library/PrivateFrameworks/SkyLight.framework/SkyLight";

// Global SkyLight library handle
static SKYLIGHT_LIB: OnceLock<Option<Library>> = OnceLock::new();

/// Target display for the overlay window.
///
/// Controls which display the flash appears on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayTarget {
    /// The main/primary display (default).
    #[default]
    Main,
    /// A specific display by its Core Graphics display ID.
    Display(u32),
    /// The display containing a specific window.
    /// The flash will appear relative to this window's position.
    Window(u64),
    // Future: All displays (would require creating multiple windows)
    // All,
}

// Window list options
const K_CG_WINDOW_LIST_OPTION_INCLUDING_WINDOW: u32 = 1 << 3;

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    // Display and window functions
    fn CGContextRelease(context: *mut c_void);
    // For getting window bounds
    fn CGWindowListCopyWindowInfo(option: u32, relativeToWindow: u32) -> *const c_void;
    fn CFRelease(cf: *const c_void);
    fn CFRunLoopRunInMode(
        mode: *const c_void,
        seconds: f64,
        return_after_source_handled: bool,
    ) -> i32;
    static kCFRunLoopDefaultMode: *const c_void;
}

// AppKit for NSApplicationLoad
#[link(name = "AppKit", kind = "framework")]
extern "C" {
    fn NSApplicationLoad() -> bool;
}

/// Ensure Cocoa framework is initialized.
/// This must be called before SLSMainConnectionID() for proper SkyLight operation.
fn ensure_cocoa_initialized() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        unsafe { NSApplicationLoad() };
    });
}

/// Get or load the SkyLight library.
fn get_skylight_lib() -> Option<&'static Library> {
    // Ensure Cocoa is initialized before loading SkyLight
    ensure_cocoa_initialized();

    SKYLIGHT_LIB
        .get_or_init(|| unsafe { Library::new(SKYLIGHT_PATH).ok() })
        .as_ref()
}

/// Run the CFRunLoop for a given duration.
///
/// Uses `CFRunLoopRunInMode` to process events for the specified duration.
/// This is required for SkyLight windows to appear on screen.
fn run_loop_for_seconds(seconds: f64) {
    unsafe {
        CFRunLoopRunInMode(kCFRunLoopDefaultMode, seconds, false);
    }
}

/// Get window bounds by window ID using CGWindowListCopyWindowInfo.
///
/// Returns the window's frame in screen coordinates, or None if not found.
pub(super) fn get_window_bounds(window_id: u64) -> Option<Rect> {
    use core_foundation::array::CFArray;
    use core_foundation::base::{CFType, TCFType};
    use core_foundation::dictionary::CFDictionary;
    use core_foundation::number::CFNumber;
    use core_foundation::string::CFString;

    unsafe {
        let info_list =
            CGWindowListCopyWindowInfo(K_CG_WINDOW_LIST_OPTION_INCLUDING_WINDOW, window_id as u32);
        if info_list.is_null() {
            return None;
        }

        let array: CFArray<CFDictionary<CFString, CFType>> =
            CFArray::wrap_under_create_rule(info_list as *const _);

        for dict in array.iter() {
            // Get the window bounds dictionary
            let bounds_key = CFString::new("kCGWindowBounds");
            if let Some(bounds_dict) = dict.find(&bounds_key) {
                // The bounds is a CFDictionary with X, Y, Width, Height
                let bounds_dict: CFDictionary<CFString, CFNumber> =
                    CFDictionary::wrap_under_get_rule(bounds_dict.as_CFTypeRef() as *const _);

                let x = bounds_dict
                    .find(CFString::new("X"))
                    .and_then(|n| n.to_f64())
                    .unwrap_or(0.0);
                let y = bounds_dict
                    .find(CFString::new("Y"))
                    .and_then(|n| n.to_f64())
                    .unwrap_or(0.0);
                let width = bounds_dict
                    .find(CFString::new("Width"))
                    .and_then(|n| n.to_f64())
                    .unwrap_or(0.0);
                let height = bounds_dict
                    .find(CFString::new("Height"))
                    .and_then(|n| n.to_f64())
                    .unwrap_or(0.0);

                return Some(Rect::from_xywh(x, y, width, height));
            }
        }
        None
    }
}

/// Builder for creating SkyLight overlay windows.
///
/// # Example
///
/// ```ignore
/// let window = SkylightWindowBuilder::new()
///     .frame(Rect::from_xywh(100.0, 100.0, 120.0, 120.0))
///     .level(WindowLevel::AboveAll)
///     .display(DisplayTarget::Main)
///     .sticky(true)
///     .build()?;
/// ```
#[derive(Debug, Clone)]
pub struct SkylightWindowBuilder {
    frame: Option<Rect>,
    level: WindowLevel,
    sticky: bool,
    alpha: f32,
    display: DisplayTarget,
}

impl Default for SkylightWindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SkylightWindowBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            frame: None,
            level: WindowLevel::AboveAll,
            sticky: true,
            alpha: 1.0,
            display: DisplayTarget::Main,
        }
    }

    /// Set the window frame (position and size).
    pub fn frame(mut self, frame: Rect) -> Self {
        self.frame = Some(frame);
        self
    }

    /// Set the window level (z-order).
    pub fn level(mut self, level: WindowLevel) -> Self {
        self.level = level;
        self
    }

    /// Set whether the window appears on all spaces (sticky).
    pub fn sticky(mut self, sticky: bool) -> Self {
        self.sticky = sticky;
        self
    }

    /// Set the window alpha (opacity).
    pub fn alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    /// Set the target display.
    pub fn display(mut self, target: DisplayTarget) -> Self {
        self.display = target;
        self
    }

    /// Build the SkyLight window.
    pub fn build(self) -> crate::Result<SkylightWindow> {
        let frame = self
            .frame
            .ok_or_else(|| anyhow::anyhow!("Window frame is required"))?;

        SkylightWindow::create(frame, self.level, self.sticky, self.alpha)
    }
}

/// A SkyLight overlay window.
pub struct SkylightWindow {
    connection_id: CGSConnectionID,
    window_id: CGSWindowID,
    context: *mut c_void,
    size: Size,
    // Store function pointers to avoid repeated lookups
    release_window: unsafe extern "C" fn(CGSConnectionID, CGSWindowID) -> i32,
    order_window: unsafe extern "C" fn(CGSConnectionID, CGSWindowID, i32, CGSWindowID) -> i32,
}

impl SkylightWindow {
    /// Create a new SkyLight overlay window using the builder.
    pub fn builder() -> SkylightWindowBuilder {
        SkylightWindowBuilder::new()
    }

    /// Internal creation method with all parameters.
    fn create(frame: Rect, level: WindowLevel, sticky: bool, alpha: f32) -> crate::Result<Self> {
        let lib = get_skylight_lib()
            .ok_or_else(|| anyhow::anyhow!("Failed to load SkyLight framework"))?;

        unsafe {
            // Load all required functions
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
            type SLSSetWindowAlphaFn =
                unsafe extern "C" fn(CGSConnectionID, CGSWindowID, f32) -> i32;
            type SLSSetWindowTagsFn =
                unsafe extern "C" fn(CGSConnectionID, CGSWindowID, *const u64, i32) -> i32;
            type CGSNewRegionWithRectFn =
                unsafe extern "C" fn(*const CGRect, *mut *const c_void) -> i32;
            type SLWindowContextCreateFn =
                unsafe extern "C" fn(CGSConnectionID, CGSWindowID, *const c_void) -> *mut c_void;

            let sls_main_connection_id: Symbol<SLSMainConnectionIDFn> = lib
                .get(b"SLSMainConnectionID")
                .map_err(|e| anyhow::anyhow!("Failed to load SLSMainConnectionID: {}", e))?;
            let sls_new_window: Symbol<SLSNewWindowFn> = lib
                .get(b"SLSNewWindow")
                .map_err(|e| anyhow::anyhow!("Failed to load SLSNewWindow: {}", e))?;
            let sls_release_window: Symbol<SLSReleaseWindowFn> = lib
                .get(b"SLSReleaseWindow")
                .map_err(|e| anyhow::anyhow!("Failed to load SLSReleaseWindow: {}", e))?;
            let sls_set_window_level: Symbol<SLSSetWindowLevelFn> =
                lib.get(b"SLSSetWindowLevel")
                    .map_err(|e| anyhow::anyhow!("Failed to load SLSSetWindowLevel: {}", e))?;
            let sls_order_window: Symbol<SLSOrderWindowFn> = lib
                .get(b"SLSOrderWindow")
                .map_err(|e| anyhow::anyhow!("Failed to load SLSOrderWindow: {}", e))?;
            let sls_set_window_opacity: Symbol<SLSSetWindowOpacityFn> = lib
                .get(b"SLSSetWindowOpacity")
                .map_err(|e| anyhow::anyhow!("Failed to load SLSSetWindowOpacity: {}", e))?;
            let sls_set_window_alpha: Symbol<SLSSetWindowAlphaFn> =
                lib.get(b"SLSSetWindowAlpha")
                    .map_err(|e| anyhow::anyhow!("Failed to load SLSSetWindowAlpha: {}", e))?;
            let sls_set_window_tags: Symbol<SLSSetWindowTagsFn> = lib
                .get(b"SLSSetWindowTags")
                .map_err(|e| anyhow::anyhow!("Failed to load SLSSetWindowTags: {}", e))?;
            let cgs_new_region_with_rect: Symbol<CGSNewRegionWithRectFn> = lib
                .get(b"CGSNewRegionWithRect")
                .map_err(|e| anyhow::anyhow!("Failed to load CGSNewRegionWithRect: {}", e))?;
            let sl_window_context_create: Symbol<SLWindowContextCreateFn> = lib
                .get(b"SLWindowContextCreate")
                .map_err(|e| anyhow::anyhow!("Failed to load SLWindowContextCreate: {}", e))?;

            // Get connection
            let cid = sls_main_connection_id();
            if cid == 0 {
                anyhow::bail!("Failed to get SkyLight connection");
            }

            // Convert frame to CGRect
            let cg_frame: CGRect = frame.into();

            // Create region
            let mut region: *const c_void = std::ptr::null();
            let region_result = cgs_new_region_with_rect(&cg_frame, &mut region);
            if region_result != 0 || region.is_null() {
                anyhow::bail!("Failed to create window region");
            }

            // Create window (type 2 = kCGBackingStoreBuffered)
            let mut wid: CGSWindowID = 0;
            let result = sls_new_window(
                cid,
                2,
                cg_frame.origin.x as f32,
                cg_frame.origin.y as f32,
                region,
                &mut wid,
            );
            CFRelease(region);

            if result != 0 || wid == 0 {
                anyhow::bail!("Failed to create SkyLight window");
            }

            // Configure window
            sls_set_window_opacity(cid, wid, false);
            sls_set_window_alpha(cid, wid, alpha);
            sls_set_window_level(cid, wid, level.to_cg_level());

            // Set window tags based on sticky setting
            // Bit 0 = sticky (appear on all spaces), Bit 11 = ignore cycle
            let tags: u64 = if sticky {
                (1 << 0) | (1 << 11)
            } else {
                1 << 11
            };
            sls_set_window_tags(cid, wid, &tags, 64);

            // Create drawing context
            let ctx = sl_window_context_create(cid, wid, std::ptr::null());
            if ctx.is_null() {
                sls_release_window(cid, wid);
                anyhow::bail!("Failed to create window context");
            }

            Ok(Self {
                connection_id: cid,
                window_id: wid,
                context: ctx,
                size: frame.size,
                release_window: *sls_release_window,
                order_window: *sls_order_window,
            })
        }
    }

    /// Draw an icon onto the window.
    pub fn draw(&mut self, icon: &Icon) -> crate::Result<()> {
        let mut canvas = unsafe { SkylightCanvas::new(self.context, self.size) };
        icon.draw(
            &mut canvas,
            &Rect::from_xywh(0.0, 0.0, self.size.width, self.size.height),
        );
        Ok(())
    }

    /// Get the raw context pointer for direct drawing.
    ///
    /// # Safety
    /// The returned pointer is only valid for the lifetime of this window.
    pub fn context_ptr(&self) -> *mut c_void {
        self.context
    }

    /// Get the window size.
    pub fn size(&self) -> Size {
        self.size
    }

    /// Show the window for the specified duration.
    ///
    /// Uses CFRunLoop to process events while the window is visible.
    pub fn show(&mut self, duration_secs: f64) -> crate::Result<()> {
        unsafe {
            // Show window (order 1 = above)
            (self.order_window)(self.connection_id, self.window_id, 1, 0);

            // Run CFRunLoop - required for SkyLight windows to appear
            run_loop_for_seconds(duration_secs);

            // Hide window (order 0 = out)
            (self.order_window)(self.connection_id, self.window_id, 0, 0);
        }
        Ok(())
    }
}

impl Drop for SkylightWindow {
    fn drop(&mut self) {
        unsafe {
            if !self.context.is_null() {
                CGContextRelease(self.context);
            }
            (self.release_window)(self.connection_id, self.window_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skylight_lib_loading() {
        // This test just verifies the lazy loading mechanism works
        // It may or may not find the library depending on the system
        let _lib = get_skylight_lib();
        // No assertion - just checking it doesn't panic
    }

    #[test]
    fn test_window_level_to_cg_level() {
        use crate::backends::skylight::cg_patches::{
            kCGFloatingWindowLevel, kCGMaximumWindowLevel, kCGMinimumWindowLevel,
            kCGModalPanelWindowLevel,
        };

        assert_eq!(WindowLevel::Normal.to_cg_level(), kCGMinimumWindowLevel);
        assert_eq!(WindowLevel::Floating.to_cg_level(), kCGFloatingWindowLevel);
        assert_eq!(
            WindowLevel::ModalPanel.to_cg_level(),
            kCGModalPanelWindowLevel
        );
        assert_eq!(WindowLevel::AboveAll.to_cg_level(), kCGMaximumWindowLevel);
        assert_eq!(WindowLevel::Custom(42).to_cg_level(), 42);
    }

    #[test]
    fn test_window_level_default() {
        assert_eq!(WindowLevel::default(), WindowLevel::AboveAll);
    }

    #[test]
    fn test_builder_defaults() {
        let builder = SkylightWindowBuilder::new();
        assert!(builder.frame.is_none());
        assert_eq!(builder.level, WindowLevel::AboveAll);
        assert!(builder.sticky);
        assert!((builder.alpha - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_builder_chain() {
        let frame = Rect::from_xywh(100.0, 100.0, 200.0, 200.0);
        let builder = SkylightWindowBuilder::new()
            .frame(frame)
            .level(WindowLevel::Floating)
            .sticky(false)
            .alpha(0.8);

        assert_eq!(builder.frame, Some(frame));
        assert_eq!(builder.level, WindowLevel::Floating);
        assert!(!builder.sticky);
        assert!((builder.alpha - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_builder_requires_frame() {
        let result = SkylightWindowBuilder::new().build();
        assert!(result.is_err());
    }
}
