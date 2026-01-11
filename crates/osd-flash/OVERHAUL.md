# osd-flash Overhaul Plan

## Executive Summary

The osd-flash crate needs a comprehensive refactoring to transition from the current SkyLight-based software animation approach to a GPU-accelerated Core Animation backend. The goal is to create a **platform-independent library** where backends are thin glue layers to capable platform APIs.

**Key Changes:**

1. Remove SkyLight backend and software animation system
2. Use `core-animation` crate as the macOS backend foundation
3. Simplify API with builder patterns matching `core-animation::WindowBuilder`
4. Refocus icon/library module as "layer compositions"
5. Keep geometry primitives platform-agnostic

**Guiding Principle:** osd-flash is a platform-independent API; platform-specific backends (like `core-animation` on macOS) do the heavy lifting.

---

## Current State Analysis

### What Exists

```
crates/osd-flash/
  src/
    lib.rs                    # Exports, prelude
    window.rs                 # OsdFlashBuilder, OsdWindow trait, GpuAnimationConfig
    flash.rs                  # FlashPosition enum
    shape.rs                  # Shape enum (RoundedRect, Circle, Ellipse)
    color.rs                  # Re-exports core-animation::Color + presets
    canvas.rs                 # Canvas trait for platform-agnostic drawing
    duration_ext.rs           # DurationExt trait
    geometry/                 # Platform-agnostic geometry (Point, Size, Rect)
    layout/                   # Margin, Padding, Border, LayoutBox
    style/                    # Paint, TextStyle, FontWeight, TextAlignment
    icon/
      mod.rs                  # Icon, IconBuilder, StyledShape, StyledText
      library/                # CameraIcon, RecordingIcon, PulsingRecordingIcon
    animation/
      mod.rs                  # Animation system docs
      animated_window.rs      # AnimatedWindow wrapper
      builder.rs              # AnimationBuilder, Animation, Repeat
      interpolation.rs        # lerp, interpolate_color
      easing.rs               # Easing enum
      transform.rs            # Transform struct (scale only)
      runner.rs               # AnimationRunner (60fps software loop)
    backends/
      skylight/
        mod.rs                # Module exports
        window.rs             # SkylightWindowBuilder, SkylightWindow
        osd_window.rs         # SkylightOsdWindow (OsdWindow impl)
        canvas.rs             # SkylightCanvas (Canvas impl via CGContext)
        geometry_ext.rs       # Conversions to CG types
        cg_patches.rs         # Missing CG constants
```

### What is Problematic

1. **SkyLight Backend is Heavyweight:**
   - Uses Apple's private SkyLight framework (undocumented, may break)
   - Software rendering via CGContext (CPU-bound)
   - Manual libloading for function pointers
   - Complex coordinate conversion (Retina scaling)

2. **Software Animation System is Redundant:**
   - The `animation/` module implements 60fps software interpolation
   - Reinvents what Core Animation does natively on GPU
   - `AnimationRunner` manually calls CFRunLoop at 60fps
   - Manual interpolation duplicates CA's animation timing

3. **Canvas Abstraction Creates Overhead:**
   - `Canvas` trait adds indirection when CA layers are declarative
   - `Drawable` trait complicates what should be layer composition
   - Re-renders entire window each frame vs. CA's layer-based updates

4. **IconBuilder is Over-Abstracted:**
   - Builds `Vec<StyledShape>` which then gets drawn to Canvas
   - This intermediate representation isn't needed with CA layers
   - `PulsingRecordingIcon` shows the awkwardness: returns both Icon and AnimationSet

5. **API Chain is Convoluted:**
   ```rust
   OsdFlashBuilder::new()
       .build()?                    // -> SkylightOsdWindow
       .draw(icon)                  // -> AnimatedWindow<SkylightOsdWindow>
       .show_animated(animations, 10.seconds())?;
   ```

   Here `icon` is an `Icon` struct containing `Vec<StyledShape>`. The `.draw()` method requires this specific type, making it difficult to compose different content sources.

   **Proposal:** `.composition()` should accept `impl Into<LayerComposition>`. Library types like `RecordingIndicator` implement `Into<LayerComposition>`, allowing any composable content:

   ```rust
   OsdBuilder::new()
       .composition(RecordingIndicator::new())  // impl Into<LayerComposition>
       .show_for(10.seconds())?;
   ```

   Compare to `core-animation`'s fluent approach where layers and animations are inline with window configuration.

---

## Target Architecture

### Core Principle

osd-flash provides a **platform-agnostic public API** for OSD notifications. The macOS backend is a thin wrapper around `core-animation::WindowBuilder`.

```
                      Platform-Agnostic
    +---------------------------------------------+
    |  osd-flash public API                       |
    |  - OsdBuilder (window + content)            |
    |  - Position, Level, Size                    |
    |  - LayerComposition (declarative shapes)    |
    |  - Animation presets                        |
    +---------------------------------------------+
                         |
         +---------------+---------------+
         |                               |
    +----v----+                     +----v----+
    | macOS   |                     | Linux   |
    | Backend |                     | Backend |
    +---------+                     +---------+
    | Thin glue to                  | (future)|
    | core-animation                |         |
    +---------+                     +---------+
         |
    +----v--------------+
    | core-animation    |
    | WindowBuilder     |
    | CAShapeLayer      |
    | CABasicAnimation  |
    +-------------------+
```

### New Module Structure

```
crates/osd-flash/
  src/
    lib.rs                    # Public API exports
    builder.rs                # OsdBuilder (main entry point)
    position.rs               # OsdPosition enum
    level.rs                  # WindowLevel enum
    composition/
      mod.rs                  # LayerComposition trait + types
      shapes.rs               # Circle, RoundedRect, etc.
      animations.rs           # Animation presets (pulse, fade, etc.)
    library/                  # Pre-built compositions
      recording.rs            # RecordingIndicator
      camera.rs               # CameraFlash
    geometry/                 # KEEP: Point, Size, Rect (platform-agnostic)
    color.rs                  # KEEP: Color re-export + presets
    backends/
      mod.rs                  # Backend trait + platform selection
      macos/
        mod.rs                # macOS backend via core-animation
```

---

## What to Remove

### Complete Removal

| Path | Reason |
|------|--------|
| `animation/` (entire module) | Replaced by CA's native GPU animations |
| `backends/skylight/` (entire module) | Replaced by core-animation backend |
| `canvas.rs` | Not needed with declarative CA layers |
| `icon/mod.rs` (IconBuilder, StyledShape) | Replaced by LayerComposition |
| `icon/library/pulsing_recording.rs` | Rewritten as composition |
| `style/paint.rs` | Absorbed into composition builders |
| `style/text_style.rs` | Simplified or moved to composition |
| `layout/box_model.rs` | Over-abstraction for OSD use case |
| `layout/border.rs` | Window border handled by backend |

### Files with Significant Changes

| Path | Change |
|------|--------|
| `window.rs` | Simplify to just OsdBuilder |
| `flash.rs` | Rename to position.rs, simplify |
| `icon/library/camera.rs` | Rewrite as LayerComposition |
| `icon/library/recording.rs` | Rewrite as LayerComposition |

---

## What to Keep

### Keep As-Is (Platform-Agnostic Primitives)

| Path | Reason |
|------|--------|
| `geometry/point.rs` | Clean, minimal Point type |
| `geometry/size.rs` | Clean, minimal Size type |
| `geometry/rect.rs` | Clean, minimal Rect type |
| `color.rs` | Already re-exports core-animation::Color |
| `duration_ext.rs` | Useful ergonomic trait |
| `layout/margin.rs` | Window margin from screen edge |
| `layout/padding.rs` | Content padding inside window |

### Keep with Modifications

| Path | Modification |
|------|--------------|
| `lib.rs` | Simplify exports, update prelude |
| `shape.rs` | Keep Shape enum, remove Drawable |

---

## What to Simplify

### IconBuilder to LayerComposition

**Current:** IconBuilder creates a `Vec<StyledShape>` that gets drawn via Canvas

**New:** LayerComposition returns configuration that backends translate to native layers

```rust
// OLD: IconBuilder approach
let icon = IconBuilder::new(80.0)
    .circle(40.0, 40.0, 12.0, Color::RED)
    .build();  // -> Icon { shapes: Vec<StyledShape> }

// NEW: LayerComposition approach
let composition = Composition::new(80.0)
    .layer("dot", |l| {
        l.circle(12.0)
            .center()
            .fill(Color::RED)
            .animate(Animation::pulse())
    })
    .build();  // -> LayerComposition (declarative config)
```

`LayerComposition` is a **struct** containing the layer configuration data. Library types like `RecordingIndicator` implement `Into<LayerComposition>` to convert themselves into the common format that backends can process.

### PulsingRecordingIcon Simplification

**Current:** Returns separate `icon()` and `animations()` that must be combined

```rust
let pulsing = PulsingRecordingIcon::new(80.0);
window.draw(pulsing.icon())
    .show_animated(pulsing.animations(), 10.seconds())?;
```

**New:** Single composition with embedded animation config

```rust
let recording = RecordingIndicator::new()
    .size(80.0)
    .build();

OsdBuilder::new()
    .composition(recording)
    .show_for(10.seconds())?;
```

[RESOLVED: library compositions be separate types (RecordingIndicator), To allow more customization via builder methods.]

---

## New API Design

### OsdBuilder (Main Entry Point)

```rust
use osd_flash::prelude::*;

// Simple static OSD
OsdBuilder::new()
    .size(100.0)
    .position(Position::TopRight)
    .margin(20.0)
    .composition(RecordingIndicator::new())
    .show_for(5.seconds())?;

// With inline layer definition (static - no animations on layer)
OsdBuilder::new()
    .size(100.0)
    .position(Position::Center)
    .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
    .corner_radius(16.0)
    .layer("circle", |l| {
        l.circle(32.0)
            .center()
            .fill(Color::RED)
    })
    .show_for(3.seconds())?;

// With animation (animations are defined on layers, not window)
OsdBuilder::new()
    .size(100.0)
    .composition(RecordingIndicator::new())  // layers have animations defined
    .show_for(10.seconds())?;
```

**Note:** There is no `show_animated()` - animation is an attribute of each layer, not a window concern. If layers have animations, they run automatically. If not, the content is static.

### OsdBuilder API

```rust
pub struct OsdBuilder {
    size: Size,
    position: Position,
    margin: Margin,
    level: WindowLevel,
    background: Option<Color>,
    corner_radius: f64,
    layers: Vec<LayerConfig>,
}

impl OsdBuilder {
    pub fn new() -> Self;

    // Dimensions
    pub fn size(self, size: impl Into<Size>) -> Self;
    pub fn dimensions(self, width: f64, height: f64) -> Self;

    // Positioning
    pub fn position(self, pos: Position) -> Self;
    pub fn margin(self, margin: impl Into<Margin>) -> Self;
    pub fn level(self, level: WindowLevel) -> Self;

    // Styling
    pub fn background(self, color: Color) -> Self;
    pub fn corner_radius(self, radius: f64) -> Self;

    // Content (choose one approach)
    pub fn composition(self, comp: impl Into<LayerComposition>) -> Self;
    pub fn layer<F>(self, name: &str, configure: F) -> Self
    where
        F: FnOnce(LayerBuilder) -> LayerBuilder;

    // Display
    pub fn show_for(self, duration: Duration) -> Result<()>;

    // Advanced: get handle for manual control
    pub fn build(self) -> Result<OsdWindow>;
}
```

### Position Enum

```rust
/// Position for OSD window relative to screen or target window.
#[derive(Debug, Clone, Copy, Default)]
pub enum Position {
    #[default]
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    Center,
    Custom { x: f64, y: f64 },
}
```

### LayerBuilder (Declarative Shapes)

```rust
pub struct LayerBuilder {
    shape: Option<ShapeKind>,
    position: LayerPosition,
    fill: Option<Color>,
    stroke: Option<(Color, f64)>,  // color, width
    opacity: f32,
    shadow: Option<ShadowConfig>,
    animations: Vec<AnimationConfig>,
}

impl LayerBuilder {
    // Shape
    pub fn circle(self, diameter: f64) -> Self;
    pub fn ellipse(self, width: f64, height: f64) -> Self;
    pub fn rounded_rect(self, width: f64, height: f64, radius: f64) -> Self;

    // Position
    pub fn position(self, x: f64, y: f64) -> Self;
    pub fn center(self) -> Self;
    pub fn center_offset(self, dx: f64, dy: f64) -> Self;

    // Style
    pub fn fill(self, color: Color) -> Self;
    pub fn stroke(self, color: Color, width: f64) -> Self;
    pub fn opacity(self, opacity: f32) -> Self;

    // Shadow/Glow
    pub fn shadow(self, color: Color, radius: f64) -> Self;
    pub fn shadow_offset(self, dx: f64, dy: f64) -> Self;

    // Animation
    pub fn animate(self, animation: Animation) -> Self;
}
```

### Animation Presets

```rust
pub enum Animation {
    /// Scale pulse (heartbeat effect)
    Pulse {
        min_scale: f64,
        max_scale: f64,
        duration: Duration,
    },
    /// Opacity fade
    Fade {
        from: f32,
        to: f32,
        duration: Duration,
    },
    /// Shadow/glow intensity
    Glow {
        min_radius: f64,
        max_radius: f64,
        duration: Duration,
    },
    /// Composed animations (run in parallel)
    Group(Vec<Animation>),
}

impl Animation {
    // Convenient constructors
    pub fn pulse() -> Self;  // Default heartbeat pulse
    pub fn pulse_range(min: f64, max: f64) -> Self;
    pub fn fade_in() -> Self;
    pub fn fade_out() -> Self;
    pub fn glow() -> Self;
}
```

### LayerComposition (Pre-built Content)

```rust
/// A complete layer composition with optional animations.
pub struct LayerComposition {
    pub size: Size,
    pub layers: Vec<LayerConfig>,
}

/// A single layer's configuration.
pub struct LayerConfig {
    pub name: String,
    pub shape: ShapeKind,
    pub position: LayerPosition,
    pub fill: Option<Color>,
    pub stroke: Option<(Color, f64)>,
    pub opacity: f32,
    pub shadow: Option<ShadowConfig>,
    pub animations: Vec<AnimationConfig>,
}

impl LayerComposition {
    pub fn new(size: impl Into<Size>) -> CompositionBuilder;
}
```

---

## Backend Architecture

### Backend Trait

[RESOLVED: Backends use conditional compilation without a trait. Option B below.]

```rust
// Option A: Trait-based (more flexible)
pub trait OsdBackend {
    type Window: OsdWindowHandle;

    fn create_window(config: &OsdConfig) -> Result<Self::Window>;
}

pub trait OsdWindowHandle {
    fn show(&self) -> Result<()>;
    fn hide(&self) -> Result<()>;
    fn show_for(&self, duration: Duration) -> Result<()>;
}

// Option B: Direct conditional compilation (simpler)
impl OsdBuilder {
    #[cfg(target_os = "macos")]
    pub fn build(self) -> Result<MacOsWindow> {
        MacOsWindow::from_config(self.into())
    }

    #[cfg(not(target_os = "macos"))]
    pub fn build(self) -> Result<StubWindow> {
        Err(anyhow!("OSD not supported on this platform"))
    }
}
```

### macOS Backend (Thin Glue)

The macOS backend translates osd-flash's declarative config to core-animation calls:

```rust
// crates/osd-flash/src/backends/macos/mod.rs

use core_animation::prelude::*;
use crate::{LayerComposition, LayerConfig, AnimationConfig};

pub struct MacOsWindow {
    window: core_animation::Window,
}

impl MacOsWindow {
    pub fn from_config(config: OsdConfig) -> Result<Self> {
        let mut builder = WindowBuilder::new()
            .size(config.size.width, config.size.height)
            .transparent()
            .borderless()
            .level(convert_level(config.level));

        // Apply position
        match config.position {
            Position::TopRight => builder = position_top_right(builder, &config),
            Position::Center => builder = builder.centered(),
            // ...
        }

        // Apply background
        if let Some(bg) = config.background {
            builder = builder
                .background_color(bg)
                .corner_radius(config.corner_radius);
        }

        // Convert each layer to CAShapeLayer
        for layer_config in &config.layers {
            builder = builder.layer(&layer_config.name, |l| {
                convert_layer(l, layer_config)
            });
        }

        Ok(Self { window: builder.build() })
    }

    pub fn show_for(&self, duration: Duration) {
        self.window.show_for(duration);
    }
}

fn convert_layer(
    builder: CAShapeLayerBuilder,
    config: &LayerConfig,
) -> CAShapeLayerBuilder {
    let mut b = match &config.shape {
        ShapeKind::Circle { diameter } => builder.circle(*diameter),
        ShapeKind::Ellipse { width, height } => builder.ellipse(*width, *height),
        // ...
    };

    b = b.position(convert_position(&config.position));

    if let Some(fill) = config.fill {
        b = b.fill_color(fill);
    }

    for anim in &config.animations {
        b = add_animation(b, anim);
    }

    b
}

fn add_animation(
    builder: CAShapeLayerBuilder,
    config: &AnimationConfig,
) -> CAShapeLayerBuilder {
    match config {
        AnimationConfig::Pulse { min, max, duration } => {
            builder.animate("pulse", KeyPath::TransformScale, |a| {
                a.values(*min, *max)
                    .duration(*duration)
                    .easing(Easing::InOut)
                    .autoreverses()
                    .repeat(Repeat::Forever)
            })
        }
        // ...
    }
}
```

---

## Icon/Library as Layer Compositions

### RecordingIndicator

```rust
// crates/osd-flash/src/library/recording.rs

use crate::prelude::*;

/// A recording indicator with pulsing animation.
pub struct RecordingIndicator {
    size: f64,
    dot_color: Color,
    glow_color: Color,
    highlight_color: Color,
    pulse_duration: Duration,
}

impl RecordingIndicator {
    pub fn new() -> Self {
        Self {
            size: 80.0,
            dot_color: Color::rgb(0.95, 0.15, 0.15),
            glow_color: Color::rgba(1.0, 0.2, 0.2, 0.5),
            highlight_color: Color::rgba(1.0, 0.95, 0.95, 0.7),
            pulse_duration: 800u64.millis(),
        }
    }

    pub fn size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.dot_color = color;
        self.glow_color = color.with_alpha(0.5);
        self
    }

    pub fn pulse_duration(mut self, duration: Duration) -> Self {
        self.pulse_duration = duration;
        self
    }
}

impl From<RecordingIndicator> for LayerComposition {
    fn from(ri: RecordingIndicator) -> Self {
        let center = ri.size / 2.0;
        let dot_diameter = ri.size * 0.4;
        let glow_diameter = ri.size * 0.55;
        let highlight_diameter = ri.size * 0.1;
        let highlight_offset = dot_diameter * 0.18;

        CompositionBuilder::new(ri.size)
            // Outer glow ring
            .layer("glow_ring", |l| {
                l.circle(glow_diameter)
                    .center()
                    .stroke(ri.glow_color, 2.5)
                    .animate(Animation::Pulse {
                        min_scale: 0.8,
                        max_scale: 1.3,
                        duration: ri.pulse_duration * 3 / 2,
                    })
                    .animate(Animation::Fade {
                        from: 0.7,
                        to: 0.15,
                        duration: ri.pulse_duration * 3 / 2,
                    })
            })
            // Inner glow
            .layer("inner_glow", |l| {
                l.circle(glow_diameter * 0.8)
                    .center()
                    .fill(ri.glow_color)
                    .shadow(ri.dot_color, 12.0)
                    .animate(Animation::pulse_range(0.9, 1.08))
            })
            // Main dot
            .layer("main_dot", |l| {
                l.circle(dot_diameter)
                    .center()
                    .fill(ri.dot_color)
                    .shadow(ri.dot_color.darker(0.1), 6.0)
                    .animate(Animation::pulse())
            })
            // Highlight
            .layer("highlight", |l| {
                l.circle(highlight_diameter)
                    .center_offset(-highlight_offset, highlight_offset)
                    .fill(ri.highlight_color)
                    .animate(Animation::Fade {
                        from: 0.8,
                        to: 0.4,
                        duration: ri.pulse_duration,
                    })
            })
            .build()
    }
}
```

### CameraFlash

```rust
// crates/osd-flash/src/library/camera.rs

pub struct CameraFlash {
    size: f64,
    background_color: Color,
    lens_color: Color,
}

impl CameraFlash {
    pub fn new() -> Self {
        Self {
            size: 120.0,
            background_color: Color::rgba(0.15, 0.45, 0.9, 0.92),
            lens_color: Color::WHITE,
        }
    }

    // Builder methods...
}

impl From<CameraFlash> for LayerComposition {
    fn from(cf: CameraFlash) -> Self {
        let center = cf.size / 2.0;
        // ... build layers for camera icon
    }
}
```

---

## Migration Path

### Phase 1: Add New Backend (Non-Breaking)

1. Create `backends/macos/` alongside `backends/skylight/`
2. Implement `MacOsWindow` using core-animation
3. Add feature flag `backend = "core-animation"` (default)
4. Keep SkyLight as fallback under `backend = "skylight"`

### Phase 2: New API Surface

1. Add `OsdBuilder` as new entry point
2. Add `LayerComposition` and related types
3. Add `composition/` module with animation presets
4. Update library to return `LayerComposition`

### Phase 3: Deprecate Old API

1. Mark `OsdFlashBuilder`, `IconBuilder`, `AnimatedWindow` as deprecated
2. Update examples to use new API
3. Update documentation

### Phase 4: Remove Old Code

1. Remove `animation/` module
2. Remove `backends/skylight/`
3. Remove `canvas.rs`, `icon/mod.rs` (builder parts)
4. Remove deprecated types

### Phase 5: Cleanup

1. Simplify `lib.rs` exports
2. Update prelude
3. Final documentation pass

---

## Design Decisions

### Architecture

1. **Conditional compilation (no trait)**
   - Using `#[cfg(target_os = "macos")]` for platform-specific backends
   - Simpler, no runtime overhead, trait can be added later if needed

2. **LayerComposition: struct**
   - Using struct with `Into<LayerComposition>` for library types
   - Keeps things simple while allowing library types to convert to the common format

3. **Library types (not factory functions)**
   - Using types like `RecordingIndicator::new().color(Color::GREEN)`
   - Allows customization via builder methods

### API

4. **OsdBuilder::show_for() consumes self**
   - Only one display method: `show_for(duration)` - consumes self (fire-and-forget)
   - Animation is a layer attribute, not a window concern

5. **Remove Drawable trait**
   - Simplifies API, use LayerComposition only
   - No escape hatch needed

6. **Text: layer type with minimal font config**
   - Text as a layer type with minimal font config (size, weight)
   - Backend maps to CATextLayer

### Implementation

7. **Geometry: osd-flash's own types**
   - Use osd-flash's Point/Size/Rect throughout
   - Backends convert to platform types via `Into`/`From` impls

8. **Color: NewType wrapping `core-animation::Color`**
   - NewType wrapper for platform independence
   - Backend converts to CGColor

9. **Animation timing: mirror core-animation's model**
   - One-to-one copy of `core_animation::Easing` enum
   - Duration handling similar to core-animation
   - No custom timing - direct adaptation without leaking core-animation types

---

## Success Criteria

1. **API Simplicity:** A pulsing recording indicator takes <10 lines of code
2. **Performance:** Animations run at 60fps with <5% CPU only GPU-accelerated, no software rendering, no hacks no quirks
3. **Platform Independence:** No Apple types in public API
4. **Backend Thinness:** macOS backend is <300 lines of glue code
5. **Test Coverage:** Core types have unit tests, examples serve as integration tests
6. **Examples:** all osd-flash examples updated to new API and run successfully
7. **Documentation:** Every public type has rustdoc with examples

---

## Appendix: API Comparison

### Current API (Verbose)

```rust
use osd_flash::prelude::*;

let pulsing = PulsingRecordingIcon::new(80.0);

OsdFlashBuilder::new()
    .dimensions(80.0)
    .position(FlashPosition::TopLeft)
    .margin(30.0)
    .corner_radius(14.0)
    .background(Color::rgba(0.1, 0.1, 0.1, 0.88))
    .build()?
    .draw(pulsing.icon())
    .show_animated(pulsing.animations(), 10.seconds())?;
```

### Target API (Concise)

```rust
use osd_flash::prelude::*;

OsdBuilder::new()
    .size(80.0)
    .position(Position::TopLeft)
    .margin(30.0)
    .background(Color::rgba(0.1, 0.1, 0.1, 0.88))
    .corner_radius(14.0)
    .composition(RecordingIndicator::new())
    .show_for(10.seconds())?;
```

### Inline Layer Definition

```rust
use osd_flash::prelude::*;

OsdBuilder::new()
    .size(100.0)
    .position(Position::Center)
    .background(Color::rgba(0.06, 0.06, 0.08, 0.92))
    .corner_radius(16.0)
    .layer("dot", |l| {
        l.circle(32.0)
            .center()
            .fill(Color::RED)
            .animate(Animation::pulse())
    })
    .layer("glow", |l| {
        l.circle(44.0)
            .center()
            .fill(Color::rgba(1.0, 0.2, 0.2, 0.35))
            .animate(Animation::pulse_range(0.9, 1.1))
    })
    .show_for(15.seconds())?;
```

---

## Appendix: Text Layer API

### Overview

Text is supported as a layer type with minimal font configuration. The API keeps it simple while allowing the most common customizations.

### TextLayer in LayerBuilder

```rust
impl LayerBuilder {
    /// Add text content to this layer.
    pub fn text(self, content: &str) -> Self;

    /// Set font size in points.
    pub fn font_size(self, size: f64) -> Self;

    /// Set font weight.
    pub fn font_weight(self, weight: FontWeight) -> Self;

    /// Set font family (defaults to system font).
    pub fn font_family(self, family: &str) -> Self;

    /// Set text color (uses fill color if not specified).
    pub fn text_color(self, color: Color) -> Self;

    /// Set text alignment within the layer bounds.
    pub fn text_align(self, align: TextAlign) -> Self;

    /// Set text opacity (0.0 to 1.0).
    pub fn text_opacity(self, opacity: f32) -> Self;
}
```

### FontWeight Enum

```rust
/// Font weight for text layers.
#[derive(Debug, Clone, Copy, Default)]
pub enum FontWeight {
    Thin,
    Light,
    #[default]
    Regular,
    Medium,
    Semibold,
    Bold,
    Heavy,
    Black,
}
```

### TextAlign Enum

```rust
/// Text alignment within layer bounds.
#[derive(Debug, Clone, Copy, Default)]
pub enum TextAlign {
    Left,
    #[default]
    Center,
    Right,
}
```

### Usage Examples

#### Simple Text OSD

```rust
OsdBuilder::new()
    .size(200.0, 60.0)
    .position(Position::TopRight)
    .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
    .corner_radius(12.0)
    .layer("label", |l| {
        l.text("Recording")
            .font_size(18.0)
            .font_weight(FontWeight::Medium)
            .text_color(Color::WHITE)
            .center()
    })
    .show_for(3.seconds())?;
```

#### Text with Icon (Composition)

```rust
OsdBuilder::new()
    .size(120.0, 50.0)
    .position(Position::TopLeft)
    .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
    .corner_radius(10.0)
    // Recording dot on the left
    .layer("dot", |l| {
        l.circle(12.0)
            .position(20.0, 25.0)
            .fill(Color::RED)
            .animate(Animation::pulse())
    })
    // Label on the right
    .layer("label", |l| {
        l.text("REC")
            .font_size(16.0)
            .font_weight(FontWeight::Bold)
            .text_color(Color::WHITE)
            .position(45.0, 25.0)
            .text_align(TextAlign::Left)
    })
    .show_for(10.seconds())?;
```

#### Animated Text (Fade In)

```rust
OsdBuilder::new()
    .size(150.0, 50.0)
    .position(Position::Center)
    .background(Color::rgba(0.0, 0.0, 0.0, 0.8))
    .corner_radius(8.0)
    .layer("message", |l| {
        l.text("Saved!")
            .font_size(20.0)
            .font_weight(FontWeight::Semibold)
            .text_color(Color::WHITE)
            .center()
            .animate(Animation::Fade {
                from: 0.0,
                to: 1.0,
                duration: 300u64.millis()
            })
    })
    .show_for(2.seconds())?;
```

#### Library Type with Text

```rust
/// A recording indicator with label.
pub struct LabeledRecordingIndicator {
    size: Size,
    label: String,
    dot_color: Color,
}

impl LabeledRecordingIndicator {
    pub fn new() -> Self {
        Self {
            size: Size::new(100.0, 40.0),
            label: "REC".to_string(),
            dot_color: Color::RED,
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
}

impl From<LabeledRecordingIndicator> for LayerComposition {
    fn from(ri: LabeledRecordingIndicator) -> Self {
        CompositionBuilder::new(ri.size)
            .layer("dot", |l| {
                l.circle(10.0)
                    .position(15.0, ri.size.height / 2.0)
                    .fill(ri.dot_color)
                    .animate(Animation::pulse())
            })
            .layer("label", |l| {
                l.text(&ri.label)
                    .font_size(14.0)
                    .font_weight(FontWeight::Bold)
                    .text_color(Color::WHITE)
                    .position(30.0, ri.size.height / 2.0)
                    .text_align(TextAlign::Left)
            })
            .build()
    }
}

// Usage
OsdBuilder::new()
    .composition(LabeledRecordingIndicator::new().label("LIVE"))
    .show_for(10.seconds())?;
```

#### Battery Indicator with Custom Font

```rust
fn show_battery(level: f64, label: &str) -> osd_flash::Result<()> {
    let width = 120.0;
    let height = 65.0;
    let batt_width = 80.0;
    let batt_height = 40.0;

    // Color based on level
    let fill_color = if level > 0.5 {
        Color::rgba(0.2, 0.9, 0.3, 1.0) // Green
    } else if level > 0.2 {
        Color::rgba(1.0, 0.7, 0.0, 1.0) // Orange
    } else {
        Color::rgba(1.0, 0.2, 0.2, 1.0) // Red
    };

    OsdBuilder::new()
        .size(width, height)
        .position(Position::TopRight)
        .background(Color::rgba(0.1, 0.1, 0.15, 0.95))
        .corner_radius(12.0)
        // Battery outline
        .layer("outline", |l| {
            l.rounded_rect(batt_width, batt_height, 6.0)
                .position(15.0, 8.0)
                .stroke(Color::WHITE, 2.0)
        })
        // Battery fill
        .layer("fill", |l| {
            let fill_width = (batt_width - 8.0) * level;
            l.rounded_rect(fill_width, batt_height - 8.0, 3.0)
                .position(19.0, 12.0)
                .fill(fill_color)
        })
        // Percentage label with custom font
        .layer("label", |l| {
            l.text(label)
                .font_size(14.0)
                .font_family("SF Mono")  // Monospace for alignment
                .font_weight(FontWeight::Medium)
                .text_color(Color::WHITE)
                .text_opacity(0.9)       // Slightly transparent
                .position(width / 2.0, batt_height + 18.0)
                .text_align(TextAlign::Center)
        })
        .show_for(2.seconds())?;

    Ok(())
}

// Usage
show_battery(0.85, "85%")?;
```

### Backend Mapping

The macOS backend maps text layers to `CATextLayer`:

```rust
// In backends/macos/mod.rs

fn convert_text_layer(config: &LayerConfig) -> Retained<CATextLayer> {
    let layer = CATextLayer::new();

    if let Some(text) = &config.text {
        // Set text content
        let ns_string = NSString::from_str(text);
        layer.setString(Some(&ns_string));

        // Set font family and size
        let font_family = config.font_family
            .as_deref()
            .unwrap_or("Helvetica Neue");

        let font_name = resolve_font_name(font_family, config.font_weight);
        layer.setFont(font_name);
        layer.setFontSize(config.font_size);

        // Set alignment
        let alignment = match config.text_align {
            TextAlign::Left => kCAAlignmentLeft,
            TextAlign::Center => kCAAlignmentCenter,
            TextAlign::Right => kCAAlignmentRight,
        };
        layer.setAlignmentMode(alignment);

        // Set color
        if let Some(color) = &config.text_color {
            layer.setForegroundColor(Some(&color.into()));
        }

        // Set opacity
        if let Some(opacity) = config.text_opacity {
            layer.setOpacity(opacity);
        }
    }

    layer
}

/// Resolve font name with weight variant.
fn resolve_font_name(family: &str, weight: FontWeight) -> &'static str {
    // For system fonts, append weight suffix
    // For custom fonts, use CTFont weight matching
    match (family, weight) {
        ("Helvetica Neue", FontWeight::Bold) => "HelveticaNeue-Bold",
        ("Helvetica Neue", FontWeight::Medium) => "HelveticaNeue-Medium",
        ("Helvetica Neue", FontWeight::Light) => "HelveticaNeue-Light",
        ("SF Mono", FontWeight::Bold) => "SFMono-Bold",
        ("SF Mono", FontWeight::Medium) => "SFMono-Medium",
        ("SF Mono", _) => "SFMono-Regular",
        _ => family, // Use as-is for unknown fonts
    }
}
```

### Design Notes

1. **Config Options**: Size, weight, family, color, alignment, and opacity.

2. **System Font Default**: Uses system UI font (San Francisco on macOS) when `font_family` not specified.

3. **Custom Fonts**: `font_family()` allows specifying any installed font (e.g., "Menlo", "SF Mono").

4. **Opacity**: `text_opacity()` controls text transparency independent of layer opacity.

5. **No Multiline**: Single-line text only. For multiline, stack multiple text layers.

6. **Animations**: Text layers support the same animations as shape layers (opacity, scale, etc.)

7. **Future Extensions**: If needed, can add later:
   - `line_height()` for multiline
   - `truncation()` for overflow handling
