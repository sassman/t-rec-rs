//! SkyLight private API bindings for macOS overlay windows.
//!
//! SkyLight is a private Apple framework used by window managers like yabai and JankyBorders.
//! It allows creating overlay windows that appear above all other content.

use std::ffi::c_void;
use std::sync::OnceLock;

use core_graphics::display::{CGDisplayBounds, CGMainDisplayID};
use core_graphics::geometry::CGRect;

use libloading::{Library, Symbol};
use log::debug;
use objc2::{class, msg_send};

use super::drawing::Canvas;
use super::geometry::{Point, Rect, Size};
use super::icon::Icon;
use super::{FlashConfig, FlashPosition};

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
        use crate::core_foundation_sys_patches::*;
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

// Link AppKit for NSApplication
#[link(name = "AppKit", kind = "framework")]
extern "C" {}

// Display and window functions
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGDisplayCopyDisplayMode(display: u32) -> *mut c_void;
    fn CGDisplayModeGetPixelWidth(mode: *mut c_void) -> usize;
    fn CGDisplayModeRelease(mode: *mut c_void);
    fn CGContextRelease(context: *mut c_void);
}

// For getting window bounds
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relativeToWindow: u32) -> *const c_void;
}

// Window list options
const K_CG_WINDOW_LIST_OPTION_INCLUDING_WINDOW: u32 = 1 << 3;

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRelease(cf: *const c_void);
}

/// Get or load the SkyLight library.
fn get_skylight_lib() -> Option<&'static Library> {
    SKYLIGHT_LIB
        .get_or_init(|| unsafe { Library::new(SKYLIGHT_PATH).ok() })
        .as_ref()
}

/// Ensure NSApplication is initialized for SkyLight windows.
///
/// SkyLight windows require an active NSApplication. This function lazily
/// initializes NSApplication as a background application (no dock icon).
/// Safe to call multiple times - initialization only happens once.
fn ensure_nsapplication() {
    use std::sync::Once;
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        unsafe {
            // Get or create the shared NSApplication
            let app: *mut objc2::runtime::AnyObject =
                msg_send![class!(NSApplication), sharedApplication];

            // Set activation policy to accessory (no dock icon, can create windows)
            // NSApplicationActivationPolicyAccessory = 1
            let _: bool = msg_send![app, setActivationPolicy: 1i64];

            // Activate the application (required for windows to appear)
            let _: () = msg_send![app, activateIgnoringOtherApps: true];
        }
    });
}

/// Run the NSRunLoop for a given duration.
///
/// Required for SkyLight windows to appear. Automatically initializes
/// NSApplication if not already done.
pub fn run_loop_for_seconds(seconds: f64) {
    ensure_nsapplication();

    unsafe {
        let future_date: *mut objc2::runtime::AnyObject =
            msg_send![class!(NSDate), dateWithTimeIntervalSinceNow: seconds];
        let runloop: *mut objc2::runtime::AnyObject = msg_send![class!(NSRunLoop), currentRunLoop];
        let _: () = msg_send![runloop, runUntilDate: future_date];
    }
}

/// Get display bounds and scale factor for a specific display.
///
/// Returns bounds in the global display coordinate space (points, not pixels).
/// CGDisplayBounds already returns point coordinates, which is what SkyLight expects.
fn get_display_info_for_id(display_id: u32) -> (Rect, f64) {
    unsafe {
        let bounds: CGRect = CGDisplayBounds(display_id);

        // Calculate scale factor for Retina displays (for informational purposes)
        let mode = CGDisplayCopyDisplayMode(display_id);
        let scale = if !mode.is_null() {
            let pixel_width = CGDisplayModeGetPixelWidth(mode);
            let s = pixel_width as f64 / bounds.size.width;
            CGDisplayModeRelease(mode);
            s
        } else {
            1.0
        };

        // CGDisplayBounds returns point coordinates in the global display space
        // No scaling needed - SkyLight works with these coordinates directly
        let display_bounds = Rect::from_xywh(
            bounds.origin.x,
            bounds.origin.y,
            bounds.size.width,
            bounds.size.height,
        );

        (display_bounds, scale)
    }
}

/// Get display bounds and scale factor for the main display.
fn get_display_info() -> (Rect, f64) {
    get_display_info_for_id(unsafe { CGMainDisplayID() })
}

/// Get window bounds by window ID using CGWindowListCopyWindowInfo.
///
/// Returns the window's frame in screen coordinates, or None if not found.
fn get_window_bounds(window_id: u64) -> Option<Rect> {
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

/// Get display bounds based on target.
fn get_target_display_info(target: DisplayTarget) -> (Rect, f64, Option<Rect>) {
    match target {
        DisplayTarget::Main => {
            let (bounds, scale) = get_display_info();
            (bounds, scale, None)
        }
        DisplayTarget::Display(id) => {
            let (bounds, scale) = get_display_info_for_id(id);
            debug!("Using display ID {} with bounds {:?}", id, bounds);
            (bounds, scale, None)
        }
        DisplayTarget::Window(window_id) => {
            let window_bounds = get_window_bounds(window_id);
            // For window target, use the main display but return window bounds for positioning
            let (display_bounds, scale) = get_display_info();
            debug!(
                "Using window ID {window_id} with bounds {window_bounds:?} on display bounds {display_bounds:?} at scale {scale}s",
            );
            (display_bounds, scale, window_bounds)
        }
    }
}

/// Menu bar height in points (approximate).
const MENU_BAR_HEIGHT: f64 = 25.0;

/// Calculate window frame based on position config.
///
/// The `top_inset` parameter accounts for the menu bar when positioning on a display,
/// or should be 0.0 when positioning relative to a window.
fn calculate_frame(config: &FlashConfig, bounds: &Rect, top_inset: f64) -> Rect {
    let size = config.icon_size;
    let margin = config.margin;

    let origin = match config.position {
        FlashPosition::TopRight => Point::new(
            bounds.origin.x / 2.0 + bounds.size.width / 2.0 - size / 2.0 - margin / 2.0,
            bounds.origin.y / 2.0 + margin + top_inset,
        ),
        FlashPosition::TopLeft => Point::new(
            bounds.origin.x / 2.0 + margin / 2.0,
            bounds.origin.y / 2.0 + margin + top_inset,
        ),
        FlashPosition::BottomRight => Point::new(
            bounds.origin.x / 2.0 + bounds.size.width / 2.0 - size / 2.0 - margin / 2.0,
            bounds.origin.y / 2.0 + bounds.size.height / 2.0 - size / 2.0 - margin / 2.0,
        ),
        FlashPosition::BottomLeft => Point::new(
            bounds.origin.x / 2.0 + margin / 2.0,
            bounds.origin.y / 2.0 + bounds.size.height / 2.0 - size / 2.0 - margin / 2.0,
        ),
        FlashPosition::Center => Point::new(
            bounds.origin.x / 2.0 + bounds.size.width / 4.0 - size / 4.0,
            bounds.origin.y / 2.0 + bounds.size.height / 4.0 - size / 4.0,
        ),
        FlashPosition::Custom { x, y } => Point::new(x, y),
    };

    Rect::new(origin, Size::square(size))
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

    /// Build the window from a FlashConfig.
    ///
    /// This is a convenience method that extracts frame from the config.
    pub fn from_config(config: &FlashConfig) -> Self {
        Self::from_config_with_target(config, DisplayTarget::Main)
    }

    /// Build the window from a FlashConfig with a specific display target.
    pub fn from_config_with_target(config: &FlashConfig, target: DisplayTarget) -> Self {
        let (display_bounds, scale, window_bounds) = get_target_display_info(target);

        // Determine bounds and top inset based on target type
        let (bounds, top_inset) = match window_bounds {
            // When targeting a window, use window bounds without menu bar offset
            Some(wb) => (wb, 0.0),
            // When targeting a display, use display bounds with menu bar offset
            None => (display_bounds, MENU_BAR_HEIGHT),
        };

        // Round coordinates to avoid subpixel rendering artifacts
        let frame = calculate_frame(config, &bounds, top_inset).rounded();

        debug!(
            "Calculated window frame {frame:?} for config {config:?} on target {target:?} with scale = {scale}",
        );

        Self::new().frame(frame).display(target)
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

    /// Create a new SkyLight overlay window from a FlashConfig.
    ///
    /// This is a convenience constructor that uses default window settings.
    pub fn new(config: &FlashConfig) -> crate::Result<Self> {
        SkylightWindowBuilder::from_config(config).build()
    }

    /// Internal creation method with all parameters.
    fn create(frame: Rect, level: WindowLevel, sticky: bool, alpha: f32) -> crate::Result<Self> {
        // Ensure NSApplication is initialized before creating SkyLight windows
        ensure_nsapplication();

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
        let mut canvas = unsafe { Canvas::new(self.context, self.size) };
        icon.draw(&mut canvas);
        Ok(())
    }

    /// Show the window for the specified duration.
    pub fn show(&mut self, duration_secs: f64) -> crate::Result<()> {
        unsafe {
            // Show window (order 1 = above)
            (self.order_window)(self.connection_id, self.window_id, 1, 0);

            // Run the NSRunLoop - required for SkyLight windows to appear
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
    fn test_calculate_frame_top_right() {
        let config = FlashConfig {
            icon_size: 100.0,
            position: FlashPosition::TopRight,
            duration_secs: 0.5,
            margin: 20.0,
        };
        let display = Rect::from_xywh(0.0, 0.0, 1920.0, 1080.0);
        let frame = calculate_frame(&config, &display, MENU_BAR_HEIGHT);

        assert_eq!(frame.size.width, 100.0);
        assert_eq!(frame.size.height, 100.0);
        // x = 0/2 + 1920/2 - 100/2 - 20/2 = 900
        assert_eq!((frame.origin.x - 900.0).abs(), 0.0);
        // y = 0/2 + 20 = 20
        assert_eq!((frame.origin.y - 20.0).abs(), 25.0);
    }

    #[test]
    fn test_calculate_frame_top_right_window() {
        let config = FlashConfig {
            icon_size: 100.0,
            position: FlashPosition::TopRight,
            duration_secs: 0.5,
            margin: 20.0,
        };
        // Window on secondary display with negative coordinates
        let window = Rect::from_xywh(-813.0, -670.0, 1062.0, 628.0);
        let frame = calculate_frame(&config, &window, 0.0);

        assert_eq!(frame.size.width, 100.0);
        assert_eq!(frame.size.height, 100.0);
        // x = -813/2 + 1062/2 - 100/2 - 20/2 = -406.5 + 531 - 50 - 10 = 64.5
        assert!((frame.origin.x - 64.5).abs() < 1.0);
        // y = -670/2 + 20 = -335 + 20 = -315
        assert!((frame.origin.y - (-315.0)).abs() < 1.0);
    }

    #[test]
    fn test_calculate_frame_center() {
        let config = FlashConfig {
            icon_size: 100.0,
            position: FlashPosition::Center,
            duration_secs: 0.5,
            margin: 20.0,
        };
        let display = Rect::from_xywh(0.0, 0.0, 1000.0, 1000.0);
        let frame = calculate_frame(&config, &display, 0.0);

        assert_eq!(frame.origin.x, 225.0);
        assert_eq!(frame.origin.y, 225.0);
    }

    #[test]
    fn test_calculate_frame_custom() {
        let config = FlashConfig {
            icon_size: 100.0,
            position: FlashPosition::Custom { x: 50.0, y: 75.0 },
            duration_secs: 0.5,
            margin: 20.0,
        };
        let display = Rect::from_xywh(0.0, 0.0, 1920.0, 1080.0);
        let frame = calculate_frame(&config, &display, 0.0);

        assert_eq!(frame.origin.x, 50.0);
        assert_eq!(frame.origin.y, 75.0);
    }

    #[test]
    fn test_skylight_lib_loading() {
        // This test just verifies the lazy loading mechanism works
        // It may or may not find the library depending on the system
        let _lib = get_skylight_lib();
        // No assertion - just checking it doesn't panic
    }

    #[test]
    fn test_window_level_to_cg_level() {
        use crate::core_foundation_sys_patches::{
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
