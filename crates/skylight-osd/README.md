# skylight-osd

On-screen display overlays for macOS using the private SkyLight framework.

This crate provides a simple API for creating overlay windows that appear above all other content, similar to macOS system notifications or the screenshot flash indicator.

## Features

- **Overlay windows** that appear above all apps, including fullscreen
- **Built-in camera icon** for screenshot feedback
- **IconBuilder API** for creating custom indicators
- **Flexible positioning** (corners, center, or custom coordinates)
- **Hex color support** for branded icons

## Requirements

- macOS only (uses private SkyLight framework)
- Works on macOS 10.14+

## Quick Start

```rust
use skylight_osd::prelude::*;

// Show the built-in camera flash
let config = FlashConfig::new()
    .icon_size(120.0)
    .position(FlashPosition::TopRight)
    .duration(1.5);

skylight_osd::flash_screenshot(&config, 0);
```

## Examples

### Camera Flash
Shows the built-in camera icon, similar to macOS screenshot feedback.

```bash
cargo run -p skylight-osd --example camera_flash
```

### Custom Icon
Demonstrates building custom icons using the `IconBuilder` API.

```bash
cargo run -p skylight-osd --example custom_icon
```

### Notification Badge
Creates notification-style badges at different screen corners.

```bash
cargo run -p skylight-osd --example notification_badge
```

### Recording Indicator
Shows a red recording dot, useful for screen recording apps.

```bash
cargo run -p skylight-osd --example recording_indicator
```

### Hex Colors
Demonstrates using hex color codes for branded icons.

```bash
cargo run -p skylight-osd --example hex_colors
```

## API Overview

### FlashConfig

Configure the flash indicator:

```rust
use skylight_osd::prelude::*;

let config = FlashConfig::new()
    .icon_size(100.0)           // Size in points
    .position(FlashPosition::Center)
    .duration(2.0)              // Duration in seconds
    .margin(20.0);              // Margin from screen edge
```

### IconBuilder

Create custom icons from shapes:

```rust
use skylight_osd::prelude::*;

let icon = IconBuilder::new(120.0)
    .padding(12.0)
    .background(Color::rgba(0.2, 0.8, 0.3, 0.92), 16.0)
    .circle(60.0, 60.0, 30.0, Color::WHITE)
    .build();
```

### SkylightWindowBuilder

Low-level window creation:

```rust
use skylight_osd::prelude::*;

let mut window = SkylightWindowBuilder::new()
    .frame(Rect::from_xywh(100.0, 100.0, 120.0, 120.0))
    .level(WindowLevel::AboveAll)
    .sticky(true)  // Appear on all spaces
    .build()?;

window.draw(&icon)?;
window.show(1.5)?;
```

### Colors

Multiple ways to define colors:

```rust
use skylight_osd::prelude::*;

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

## Window Levels

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
