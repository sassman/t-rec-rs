# osd-flash

On-screen display (OSD) flash indicators for various platforms.

This crate provides a simple API for creating overlay windows that appear above all other content, similar to macOS system notifications or the screenshot flash indicator.

## Backends

- **skylight** (macOS): Uses Apple's private SkyLight framework for overlay windows that appear above all other content, including fullscreen apps.

## Features

- **Overlay windows** that appear above all apps, including fullscreen
- **Built-in icons** (camera, recording) for common use cases
- **IconBuilder API** for creating custom indicators
- **Flexible positioning** (corners, center, or custom coordinates)
- **Hex color support** for branded icons

## Requirements

- macOS: Uses private SkyLight framework, works on macOS 10.14+

## Quick Start

```rust
use osd_flash::prelude::*;

// Show a camera flash indicator
OsdFlashBuilder::new()
    .dimensions(120.0)
    .position(FlashPosition::TopRight)
    .margin(20.0)
    .level(WindowLevel::AboveAll)
    .build()?
    .draw(CameraIcon::new(120.0).build())
    .show_for_seconds(1.5)?;
```

## Examples

### Camera Flash
Shows the built-in camera icon, similar to macOS screenshot feedback.

```bash
cargo run -p osd-flash --example camera_flash
```

### Custom Icon
Demonstrates building custom icons using the `IconBuilder` API.

```bash
cargo run -p osd-flash --example custom_icon
```

### Notification Badge
Creates notification-style badges at different screen corners.

```bash
cargo run -p osd-flash --example notification_badge
```

### Recording Indicator
Shows a red recording dot, useful for screen recording apps.

```bash
cargo run -p osd-flash --example recording_indicator
```

### Hex Colors
Demonstrates using hex color codes for branded icons.

```bash
cargo run -p osd-flash --example hex_colors
```

## API Overview

### OsdFlashBuilder

The main entry point for creating OSD flash windows:

```rust
use osd_flash::prelude::*;

OsdFlashBuilder::new()
    .dimensions(100.0)              // Size in points (square)
    .position(FlashPosition::Center)
    .margin(20.0)                   // Margin from screen edge
    .level(WindowLevel::AboveAll)   // Z-order
    .attach_to_window(window_id)    // Optional: attach to specific window
    .build()?
    .draw(CameraIcon::new(100.0).build())
    .show_for_seconds(2.0)?;
```

### IconBuilder

Create custom icons from shapes:

```rust
use osd_flash::prelude::*;

let icon = IconBuilder::new(120.0)
    .padding(12.0)
    .background(Color::rgba(0.2, 0.8, 0.3, 0.92), 16.0)
    .circle(60.0, 60.0, 30.0, Color::WHITE)
    .build();
```

### Built-in Icons

```rust
use osd_flash::prelude::*;

// Camera icon for screenshot feedback
let camera = CameraIcon::new(120.0).build();

// Recording indicator (red dot)
let recording = RecordingIcon::new(60.0).build();
```

### Colors

Multiple ways to define colors:

```rust
use osd_flash::prelude::*;

// Preset colors
let white = Color::WHITE;
let blue = Color::VIBRANT_BLUE;

// RGB/RGBA (0.0 to 1.0)
let custom = Color::rgba(0.5, 0.3, 0.8, 0.9);

// 8-bit values (0 to 255)
let orange = Color::rgb8(255, 128, 0);

// Hex codes
let github = Color::from_hex("#24292e").unwrap();
```

## Window Levels (macOS)

Control z-ordering of the overlay:

- `WindowLevel::Normal` - Regular window level
- `WindowLevel::Floating` - Above normal windows
- `WindowLevel::ModalPanel` - Above floating windows
- `WindowLevel::AboveAll` - Maximum z-index (default)

## Positions

Built-in screen positions:

- `FlashPosition::TopRight` (default)
- `FlashPosition::TopLeft`
- `FlashPosition::BottomRight`
- `FlashPosition::BottomLeft`
- `FlashPosition::Center`
- `FlashPosition::Custom { x, y }`

## License

GPL-3.0-only
