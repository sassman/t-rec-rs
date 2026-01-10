//! Test window for running examples.
//!
//! For production use, integrate layers with your own window management.

#[cfg(feature = "screenshot")]
use std::path::Path;
use std::time::Duration;

use crate::color::Color;
use crate::shape_layer_builder::CAShapeLayerBuilder;
use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSColor, NSScreen, NSWindow,
    NSWindowStyleMask,
};
use objc2_core_foundation::{kCFRunLoopDefaultMode, CFRunLoop, CFTimeInterval};
use objc2_core_graphics::CGColor;
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use objc2_quartz_core::{CALayer, CAShapeLayer};

#[cfg(feature = "screenshot")]
use objc2::AnyThread;
#[cfg(feature = "screenshot")]
use objc2_app_kit::NSBitmapImageRep;
#[cfg(feature = "screenshot")]
use objc2_core_graphics::{CGWindowImageOption, CGWindowListCreateImage, CGWindowListOption};
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

/// Window level determining the z-order of the overlay window.
///
/// Controls where the window appears in the window stack relative to other windows.
///
/// # Common levels
///
/// - `Normal` (0): Standard application windows
/// - `Floating` (3): Floating palettes, tool windows
/// - `ModalPanel` (8): Modal panels that block interaction
/// - `ScreenSaver` (1000): Screen saver level
/// - `AboveAll` (1001): Above all windows including fullscreen apps
///
/// # Example
///
/// ```ignore
/// use core_animation::prelude::*;
///
/// let window = WindowBuilder::new()
///     .size(200.0, 200.0)
///     .level(WindowLevel::AboveAll)
///     .build();
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowLevel {
    /// Normal window level (0), appears with regular application windows.
    Normal,
    /// Floating window level (3), appears above normal windows.
    Floating,
    /// Modal panel level (8), appears above floating windows.
    ModalPanel,
    /// Screen saver level (1000).
    ScreenSaver,
    /// Above all other windows (1001), including fullscreen apps and the Dock.
    #[default]
    AboveAll,
    /// Custom window level value (platform-specific).
    Custom(isize),
}

impl WindowLevel {
    /// Returns the raw window level value for NSWindow.
    pub fn raw_level(&self) -> isize {
        match self {
            WindowLevel::Normal => 0,
            WindowLevel::Floating => 3,
            WindowLevel::ModalPanel => 8,
            WindowLevel::ScreenSaver => 1000,
            WindowLevel::AboveAll => 1001,
            WindowLevel::Custom(level) => *level,
        }
    }
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
    transparent: bool,
    corner_radius: Option<f64>,
    level: Option<WindowLevel>,
    border_color: Option<Color>,
    layers: Vec<(String, Retained<CAShapeLayer>)>,
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
            transparent: false,
            corner_radius: None,
            level: None,
            border_color: None,
            layers: Vec::new(),
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

    /// Make the window fully transparent for overlay effects.
    ///
    /// When enabled, this sets the NSWindow to be non-opaque with a clear background,
    /// allowing the content to be drawn on a fully transparent background.
    /// This is useful for creating floating overlay windows.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use core_animation::prelude::*;
    ///
    /// let window = WindowBuilder::new()
    ///     .size(200.0, 200.0)
    ///     .transparent()
    ///     .borderless()
    ///     .build();
    /// ```
    pub fn transparent(mut self) -> Self {
        self.transparent = true;
        self
    }

    /// Set the corner radius on the container layer.
    ///
    /// This applies rounded corners to the container layer that holds your content.
    /// Combine with `.background_color()` for a rounded panel effect.
    ///
    /// # Arguments
    ///
    /// * `radius` - The corner radius in points
    ///
    /// # Example
    ///
    /// ```ignore
    /// use core_animation::prelude::*;
    ///
    /// let window = WindowBuilder::new()
    ///     .size(200.0, 200.0)
    ///     .transparent()
    ///     .borderless()
    ///     .corner_radius(20.0)
    ///     .background_color(Color::gray(0.1).with_alpha(0.85))
    ///     .build();
    /// ```
    pub fn corner_radius(mut self, radius: f64) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Set the window level (z-order).
    ///
    /// Controls where the window appears in the window stack relative to other windows.
    /// Higher levels appear above lower levels.
    ///
    /// # Arguments
    ///
    /// * `level` - The window level to set
    ///
    /// # Example
    ///
    /// ```ignore
    /// use core_animation::prelude::*;
    ///
    /// // Create an overlay that floats above all windows
    /// let window = WindowBuilder::new()
    ///     .size(200.0, 200.0)
    ///     .level(WindowLevel::AboveAll)
    ///     .build();
    ///
    /// // Or use a custom level
    /// let window = WindowBuilder::new()
    ///     .level(WindowLevel::Custom(500))
    ///     .build();
    /// ```
    pub fn level(mut self, level: WindowLevel) -> Self {
        self.level = Some(level);
        self
    }

    /// Set the border color on the container layer.
    ///
    /// When set, this also applies a border width of 1.0 to the container layer.
    /// This creates a subtle visible border around the window content.
    ///
    /// # Arguments
    ///
    /// * `color` - The border color
    ///
    /// # Example
    ///
    /// ```ignore
    /// use core_animation::prelude::*;
    ///
    /// // Create a window with a subtle gray border
    /// let window = WindowBuilder::new()
    ///     .size(200.0, 200.0)
    ///     .transparent()
    ///     .borderless()
    ///     .corner_radius(20.0)
    ///     .background_color(Color::gray(0.1).with_alpha(0.85))
    ///     .border_color(Color::rgba(0.3, 0.3, 0.35, 0.5))
    ///     .build();
    /// ```
    pub fn border_color(mut self, color: impl Into<Color>) -> Self {
        self.border_color = Some(color.into());
        self
    }

    /// Add a shape layer to the window.
    ///
    /// The closure receives a [`CAShapeLayerBuilder`] for configuration.
    /// The built layer will be added to [`Window::container()`] when [`build()`](Self::build) is called.
    ///
    /// This allows configuring shape layers inline in the fluent API,
    /// creating a fully fluent flow from window to layers to animations.
    ///
    /// # Arguments
    ///
    /// * `name` - A unique identifier for this layer
    /// * `configure` - A closure that configures the layer builder
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use core_animation::prelude::*;
    ///
    /// let window = WindowBuilder::new()
    ///     .title("Animation Demo")
    ///     .size(400.0, 400.0)
    ///     .centered()
    ///     .transparent()
    ///     .borderless()
    ///     .background_color(Color::rgba(0.1, 0.1, 0.15, 0.85))
    ///     .layer("circle", |s| {
    ///         s.circle(80.0)
    ///             .position(CGPoint::new(200.0, 200.0))
    ///             .fill_color(Color::CYAN)
    ///             .animate("pulse", KeyPath::TransformScale, |a| {
    ///                 a.values(0.85, 1.15)
    ///                     .duration(2.seconds())
    ///                     .autoreverses()
    ///                     .repeat(Repeat::Forever)
    ///             })
    ///     })
    ///     .build();
    ///
    /// window.show_for(10.seconds());
    /// ```
    ///
    /// Multiple layers can be added:
    ///
    /// ```ignore
    /// WindowBuilder::new()
    ///     .size(400.0, 400.0)
    ///     .layer("background_ring", |s| {
    ///         s.circle(200.0)
    ///             .position(CGPoint::new(200.0, 200.0))
    ///             .fill_color(Color::TRANSPARENT)
    ///             .stroke_color(Color::WHITE.with_alpha(0.3))
    ///             .line_width(2.0)
    ///     })
    ///     .layer("main_circle", |s| {
    ///         s.circle(80.0)
    ///             .position(CGPoint::new(200.0, 200.0))
    ///             .fill_color(Color::CYAN)
    ///     })
    ///     .build();
    /// ```
    pub fn layer<F>(mut self, name: &str, configure: F) -> Self
    where
        F: FnOnce(CAShapeLayerBuilder) -> CAShapeLayerBuilder,
    {
        let builder = CAShapeLayerBuilder::new();
        let configured = configure(builder);
        let layer = configured.build();
        self.layers.push((name.to_string(), layer));
        self
    }

    /// Build the window.
    ///
    /// # Panics
    ///
    /// Panics if not called from the main thread.
    pub fn build(self) -> Window {
        let mtm = MainThreadMarker::new()
            .expect("WindowBuilder::build() must be called from the main thread");

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

        // Apply transparency settings
        if self.transparent {
            ns_window.setOpaque(false);
            let clear_color = NSColor::clearColor();
            ns_window.setBackgroundColor(Some(&clear_color));
        }

        // Apply window level
        if let Some(level) = self.level {
            ns_window.setLevel(level.raw_level().into());
        }

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

        // Apply corner radius to container layer
        if let Some(radius) = self.corner_radius {
            container.setCornerRadius(radius);
        }

        // Apply border color and width to container layer
        if let Some(color) = self.border_color {
            let cgcolor: objc2_core_foundation::CFRetained<CGColor> = color.into();
            container.setBorderColor(Some(&cgcolor));
            container.setBorderWidth(1.0);
        }

        root_layer.addSublayer(&container);

        // Add all configured layers to the container
        for (_name, layer) in &self.layers {
            container.addSublayer(layer);
        }

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
}
