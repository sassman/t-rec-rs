//! Test window for running examples.
//!
//! For production use, integrate layers with your own window management.

#[cfg(feature = "screenshot")]
use std::path::Path;
use std::time::Duration;

use crate::color::Color;
use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSScreen, NSWindow,
    NSWindowStyleMask,
};
use objc2_core_foundation::{kCFRunLoopDefaultMode, CFRunLoop, CFTimeInterval};
use objc2_core_graphics::CGColor;
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use objc2_quartz_core::CALayer;

#[cfg(feature = "screenshot")]
use objc2::AnyThread;
#[cfg(feature = "screenshot")]
use objc2_app_kit::NSBitmapImageRep;
#[cfg(feature = "screenshot")]
use objc2_core_graphics::{
    CGWindowImageOption, CGWindowListCreateImage, CGWindowListOption,
};
#[cfg(feature = "screenshot")]
use objc2_foundation::NSDictionary;

/// Specifies which screen to use for window placement.
#[derive(Clone, Debug, Default)]
pub enum Screen {
    /// The main screen (default). This is typically the screen with the menu bar.
    #[default]
    Main,
    /// A specific screen by index (0-based). Index 0 is typically the main screen.
    Index(usize),
}

/// Window style options.
#[derive(Clone, Debug)]
pub struct WindowStyle {
    pub titled: bool,
    pub closable: bool,
    pub resizable: bool,
    pub miniaturizable: bool,
    pub borderless: bool,
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            titled: true,
            closable: true,
            resizable: true,
            miniaturizable: true,
            borderless: false,
        }
    }
}

/// Builder for test windows.
///
/// ```ignore
/// let window = WindowBuilder::new()
///     .title("Demo")
///     .size(640.0, 480.0)
///     .centered()
///     .background_color(Color::gray(0.1))
///     .build();
///
/// window.container().add_sublayer(&my_layer);
/// window.show_for(5.seconds());
/// ```
pub struct WindowBuilder {
    title: String,
    size: (f64, f64),
    position: Option<(f64, f64)>,
    centered: bool,
    screen: Screen,
    style: WindowStyle,
    background: Option<Color>,
    activation_policy: NSApplicationActivationPolicy,
}

impl WindowBuilder {
    /// Create a new window builder with default settings.
    pub fn new() -> Self {
        Self {
            title: String::from("Window"),
            size: (640.0, 480.0),
            position: None,
            centered: false,
            screen: Screen::Main,
            style: WindowStyle::default(),
            background: None,
            activation_policy: NSApplicationActivationPolicy::Accessory,
        }
    }

    /// Set the window title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the window size in points.
    pub fn size(mut self, width: f64, height: f64) -> Self {
        self.size = (width, height);
        self
    }

    /// Set the window position in screen coordinates.
    /// This is mutually exclusive with `centered()`.
    pub fn position(mut self, x: f64, y: f64) -> Self {
        self.position = Some((x, y));
        self.centered = false;
        self
    }

    /// Center the window on the selected screen.
    /// This is mutually exclusive with `position()`.
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self.position = None;
        self
    }

    /// Select which screen to place the window on.
    /// Defaults to `Screen::Main`.
    pub fn on_screen(mut self, screen: Screen) -> Self {
        self.screen = screen;
        self
    }

    /// Set the background color of the root container layer.
    ///
    /// Accepts any type that implements `Into<Color>`, including:
    /// - `Color::RED`, `Color::rgb(0.1, 0.1, 0.2)`
    /// - `Color::WHITE.with_alpha(0.5)`
    pub fn background_color(mut self, color: impl Into<Color>) -> Self {
        self.background = Some(color.into());
        self
    }

    /// Set the background color of the root container layer (RGBA, 0.0-1.0).
    pub fn background_rgba(mut self, r: f64, g: f64, b: f64, a: f64) -> Self {
        self.background = Some(Color::rgba(r, g, b, a));
        self
    }

    /// Set the background color of the root container layer (RGB with alpha=1.0).
    pub fn background_rgb(mut self, r: f64, g: f64, b: f64) -> Self {
        self.background = Some(Color::rgb(r, g, b));
        self
    }

    /// Configure window style.
    pub fn style(mut self, style: WindowStyle) -> Self {
        self.style = style;
        self
    }

    /// Make the window borderless (no title bar, not resizable).
    pub fn borderless(mut self) -> Self {
        self.style.borderless = true;
        self.style.titled = false;
        self
    }

    /// Set whether the window has a title bar.
    pub fn titled(mut self, titled: bool) -> Self {
        self.style.titled = titled;
        self
    }

    /// Set whether the window is closable.
    pub fn closable(mut self, closable: bool) -> Self {
        self.style.closable = closable;
        self
    }

    /// Set whether the window is resizable.
    pub fn resizable(mut self, resizable: bool) -> Self {
        self.style.resizable = resizable;
        self
    }

    /// Set the application activation policy.
    /// Defaults to `Accessory` (no dock icon, no menu bar).
    pub fn activation_policy(mut self, policy: NSApplicationActivationPolicy) -> Self {
        self.activation_policy = policy;
        self
    }

    /// Build the window.
    ///
    /// # Panics
    ///
    /// Panics if not called from the main thread.
    pub fn build(self) -> Window {
        let mtm = MainThreadMarker::new().expect("WindowBuilder::build() must be called from the main thread");

        // Initialize application
        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(self.activation_policy);

        // Get the target screen
        let screen = self.get_screen(mtm);
        let screen_frame = screen.frame();

        // Calculate window frame
        let window_size = NSSize::new(self.size.0, self.size.1);
        let window_origin = if self.centered {
            NSPoint::new(
                (screen_frame.size.width - window_size.width) / 2.0 + screen_frame.origin.x,
                (screen_frame.size.height - window_size.height) / 2.0 + screen_frame.origin.y,
            )
        } else if let Some((x, y)) = self.position {
            NSPoint::new(x, y)
        } else {
            // Default: top-left with some margin
            NSPoint::new(
                screen_frame.origin.x + 100.0,
                screen_frame.origin.y + screen_frame.size.height - window_size.height - 100.0,
            )
        };
        let content_rect = NSRect::new(window_origin, window_size);

        // Build style mask
        let mut style_mask = NSWindowStyleMask::empty();
        if self.style.borderless {
            style_mask |= NSWindowStyleMask::Borderless;
        } else {
            if self.style.titled {
                style_mask |= NSWindowStyleMask::Titled;
            }
            if self.style.closable {
                style_mask |= NSWindowStyleMask::Closable;
            }
            if self.style.resizable {
                style_mask |= NSWindowStyleMask::Resizable;
            }
            if self.style.miniaturizable {
                style_mask |= NSWindowStyleMask::Miniaturizable;
            }
        }

        // Create window
        let ns_window = unsafe {
            let window = NSWindow::alloc(mtm);
            let window = NSWindow::initWithContentRect_styleMask_backing_defer(
                window,
                content_rect,
                style_mask,
                NSBackingStoreType::Buffered,
                false,
            );
            window.setReleasedWhenClosed(false);
            window
        };

        // Set title
        let title = NSString::from_str(&self.title);
        ns_window.setTitle(&title);

        // Enable layer backing
        let content_view = ns_window.contentView().expect("Window has no content view");
        content_view.setWantsLayer(true);

        let root_layer = content_view.layer().expect("View has no layer");

        // Create container layer with background
        let container = CALayer::new();
        container.setBounds(objc2_core_foundation::CGRect::new(
            objc2_core_foundation::CGPoint::new(0.0, 0.0),
            objc2_core_foundation::CGSize::new(self.size.0, self.size.1),
        ));
        container.setPosition(objc2_core_foundation::CGPoint::new(
            self.size.0 / 2.0,
            self.size.1 / 2.0,
        ));

        if let Some(color) = self.background {
            let cgcolor: objc2_core_foundation::CFRetained<CGColor> = color.into();
            container.setBackgroundColor(Some(&cgcolor));
        }

        root_layer.addSublayer(&container);

        Window {
            ns_window,
            container,
            size: self.size,
            mtm,
        }
    }

    /// Get the NSScreen for the selected screen.
    fn get_screen(&self, mtm: MainThreadMarker) -> Retained<NSScreen> {
        match &self.screen {
            Screen::Main => NSScreen::mainScreen(mtm).expect("No main screen available"),
            Screen::Index(idx) => {
                let screens = NSScreen::screens(mtm);
                if *idx < screens.len() {
                    screens.objectAtIndex(*idx)
                } else {
                    // Fall back to main screen if index is out of bounds
                    NSScreen::mainScreen(mtm).expect("No main screen available")
                }
            }
        }
    }
}

impl Default for WindowBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A window with a layer container for adding sublayers.
pub struct Window {
    ns_window: Retained<NSWindow>,
    container: Retained<CALayer>,
    size: (f64, f64),
    mtm: MainThreadMarker,
}

impl Window {
    /// Get the container layer.
    ///
    /// Add your layers as sublayers of this container.
    pub fn container(&self) -> &CALayer {
        &self.container
    }

    /// Get the window size in points.
    pub fn size(&self) -> (f64, f64) {
        self.size
    }

    /// Get the underlying NSWindow.
    pub fn ns_window(&self) -> &NSWindow {
        &self.ns_window
    }

    /// Returns the CGWindowID for this window.
    ///
    /// This is useful for screen recording with t-rec's HeadlessRecorder.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use core_animation::prelude::*;
    /// use t_rec::HeadlessRecorder;
    ///
    /// let window = WindowBuilder::new()
    ///     .title("Demo")
    ///     .size(400.0, 300.0)
    ///     .build();
    ///
    /// let mut recorder = HeadlessRecorder::builder()
    ///     .window_id(window.window_id())
    ///     .fps(30)
    ///     .output_gif("demo.gif")
    ///     .build()?;
    /// ```
    pub fn window_id(&self) -> u64 {
        // NSWindow.windowNumber returns NSInteger (isize on macOS)
        // CGWindowID is u32 but we use u64 for t-rec compatibility
        self.ns_window.windowNumber() as u64
    }

    /// Show the window and bring it to the front.
    pub fn show(&self) {
        self.ns_window.makeKeyAndOrderFront(None);
        #[allow(deprecated)]
        NSApplication::sharedApplication(self.mtm).activateIgnoringOtherApps(true);
    }

    /// Show the window for a specified duration.
    ///
    /// This shows the window and processes events for the given duration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use core_animation::prelude::*;
    ///
    /// window.show_for(5.seconds());
    /// window.show_for(500.millis());
    /// window.show_for(1.5.seconds());
    /// ```
    pub fn show_for(&self, duration: Duration) {
        self.show();

        let start = std::time::Instant::now();
        while start.elapsed() < duration {
            self.run_loop_tick();
        }
    }

    /// Run a single iteration of the event loop.
    ///
    /// This processes pending events and returns immediately.
    /// Useful for custom animation loops.
    pub fn run_loop_tick(&self) {
        let mode = unsafe { kCFRunLoopDefaultMode };
        CFRunLoop::run_in_mode(mode, 1.0 / 60.0 as CFTimeInterval, false);
    }

    /// Run the event loop indefinitely until the window is closed.
    pub fn run(&self) {
        self.show();

        // Run until window is closed
        while self.ns_window.isVisible() {
            self.run_loop_tick();
        }
    }

    /// Show the window for a duration and capture a screenshot at the midpoint.
    ///
    /// This is useful for generating example screenshots automatically.
    /// The screenshot is saved as PNG to the specified path.
    ///
    /// # Example
    ///
    /// ```ignore
    /// window.show_for_with_screenshot(
    ///     10.seconds(),
    ///     Path::new("examples/screenshots/my_example.png"),
    /// );
    /// ```
    #[cfg(feature = "screenshot")]
    pub fn show_for_with_screenshot(&self, duration: Duration, path: &Path) {
        self.show();

        let start = std::time::Instant::now();
        let midpoint = duration / 2;
        let mut screenshot_taken = false;

        while start.elapsed() < duration {
            self.run_loop_tick();

            // Capture screenshot at midpoint
            if !screenshot_taken && start.elapsed() >= midpoint {
                if let Err(e) = self.capture_screenshot(path) {
                    eprintln!("Failed to capture screenshot: {}", e);
                } else {
                    println!("Screenshot saved to: {}", path.display());
                }
                screenshot_taken = true;
            }
        }
    }

    /// Capture a screenshot of the window and save it to the specified path.
    ///
    /// The image is saved as PNG format.
    #[cfg(feature = "screenshot")]
    #[allow(deprecated)] // CGWindowListCreateImage is deprecated but still works
    pub fn capture_screenshot(&self, path: &Path) -> Result<(), String> {
        use objc2_core_foundation::{CGPoint as CFGPoint, CGRect as CFGRect, CGSize as CFGSize};

        // Get the window ID
        let window_id = self.ns_window.windowNumber() as u32;

        // kCGWindowListOptionIncludingWindow = 1 << 3
        // kCGWindowImageDefault = 0
        let list_option = CGWindowListOption(1 << 3);
        let image_option = CGWindowImageOption(0);

        // Use CGRectNull equivalent (all zeros means capture the minimum rect that contains the window)
        let null_rect = CFGRect::new(CFGPoint::new(0.0, 0.0), CFGSize::new(0.0, 0.0));

        // Capture the window image
        let image = CGWindowListCreateImage(null_rect, list_option, window_id, image_option);

        let Some(image) = image else {
            return Err("Failed to capture window image".to_string());
        };

        // Create NSBitmapImageRep from CGImage
        let bitmap_rep = NSBitmapImageRep::initWithCGImage(NSBitmapImageRep::alloc(), &image);

        // Get PNG data
        // SAFETY: The bitmap_rep is valid and we're passing valid parameters
        let png_data = unsafe {
            bitmap_rep.representationUsingType_properties(
                objc2_app_kit::NSBitmapImageFileType::PNG,
                &NSDictionary::new(),
            )
        };

        let Some(png_data) = png_data else {
            return Err("Failed to create PNG data".to_string());
        };

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Write to file using NSData's writeToFile
        let ns_path = NSString::from_str(&path.to_string_lossy());
        let success = png_data.writeToFile_atomically(&ns_path, true);

        if success {
            Ok(())
        } else {
            Err("Failed to write file".to_string())
        }
    }
}
