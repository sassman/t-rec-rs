# Implementation Plan: Custom Wallpaper Support

## Overview

Extend the existing `--wallpaper` feature to support user-provided custom wallpaper images. Users can specify their own wallpaper file path, with strict validation to ensure the wallpaper meets resolution requirements before recording starts.

**Reference**: [GitHub Issue #225](https://github.com/sassman/t-rec-rs/issues/225)

## Requirements from Issue #225

1. **Default wallpaper**: Provided by t-rec, available offline (already implemented with `ventura`)
2. **Custom wallpaper**: Users can specify their own wallpaper via CLI argument
3. **Supported formats**: PNG, JPEG, and TGA
4. **Resolution validation**: Wallpaper must be at least n-pixel wider and higher than the recorded terminal, where n must match the 2x the padding value (default 60px), so for left side one padding, and righ side another padding, that makes 2x60 = 120px total extra width and height for the default case
5. **Pre-recording validation**: Resolution must be validated BEFORE recording starts; if insufficient, inform user and stop
6. **Offline operation**: No internet download required

## Current State (Already Implemented)

- `--wallpaper ventura` / `-p ventura`: Built-in Ventura wallpaper
- `--wallpaper-padding <1-500>`: Configurable padding (default: 60px)
- `apply_wallpaper_effect()`: Generic function accepting any `&DynamicImage`

---

## Implementation Steps

### Step 1: Update CLI Arguments

**File**: `src/cli.rs`

**Current**:
```rust
.arg(
    Arg::new("wallpaper")
        .value_parser(["ventura"])
        .required(false)
        .short('p')
        .long("wallpaper")
        .help("...")
)
```

**New**: Change `--wallpaper` to accept either a preset name OR a file path:

```rust
.arg(
    Arg::new("wallpaper")
        .value_parser(NonEmptyStringValueParser::new())
        .required(false)
        .short('p')
        .long("wallpaper")
        .help("Wallpaper background. Use 'ventura' for built-in, or provide a path to a custom image (PNG, JPEG, TGA)")
)
```

**Validation logic** (in main.rs, not cli.rs):
- If value is `"ventura"` → use built-in
- Otherwise → treat as file path and validate

### Step 2: Create Wallpaper Validation Module

**File**: `src/wallpapers/validation.rs` (new)

```rust
use std::path::Path;
use image::{DynamicImage, GenericImageView, ImageReader};
use anyhow::{Context, Result, bail};

/// Supported wallpaper formats
const SUPPORTED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "tga"];

/// Validate and load a custom wallpaper from a file path
///
/// # Validation checks:
/// 1. File exists and is readable
/// 2. File extension is supported (png, jpg, jpeg, tga)
/// 3. File is a valid image that can be decoded
/// 4. Resolution is sufficient for the terminal size + padding
///    (wallpaper must be at least terminal_size + 2*padding in each dimension)
///
/// # Security considerations:
/// - Path traversal: We only read the file, no writes
/// - File size: image crate handles memory allocation
/// - Format validation: Only decode known safe formats
pub fn load_and_validate_wallpaper(
    path: &Path,
    terminal_width: u32,
    terminal_height: u32,
    padding: u32,
) -> Result<DynamicImage> {
    // 1. Check file exists
    if !path.exists() {
        bail!("Wallpaper file not found: {}", path.display());
    }

    if !path.is_file() {
        bail!("Wallpaper path is not a file: {}", path.display());
    }

    // 2. Validate file extension
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    if !SUPPORTED_EXTENSIONS.contains(&extension.as_str()) {
        bail!(
            "Unsupported wallpaper format '{}'. Supported formats: {}",
            extension,
            SUPPORTED_EXTENSIONS.join(", ")
        );
    }

    // 3. Load and decode the image
    let wallpaper = ImageReader::open(path)
        .with_context(|| format!("Failed to open wallpaper file: {}", path.display()))?
        .with_guessed_format()
        .with_context(|| "Failed to detect wallpaper image format")?
        .decode()
        .with_context(|| format!("Failed to decode wallpaper image: {}", path.display()))?;

    // 4. Validate resolution
    // Wallpaper must be at least terminal_size + 2*padding (padding on each side)
    let (wp_width, wp_height) = wallpaper.dimensions();
    let min_width = terminal_width + (padding * 2);
    let min_height = terminal_height + (padding * 2);

    if wp_width < min_width || wp_height < min_height {
        bail!(
            "Wallpaper resolution {}x{} is too small.\n\
             Required: at least {}x{} (terminal {}x{} + {}px padding on each side).\n\
             Please use a larger wallpaper image.",
            wp_width, wp_height,
            min_width, min_height,
            terminal_width, terminal_height,
            padding
        );
    }

    Ok(wallpaper)
}

/// Check if a wallpaper value is a built-in preset or a custom path
pub fn is_builtin_wallpaper(value: &str) -> bool {
    matches!(value.to_lowercase().as_str(), "ventura")
}
```

### Step 3: Update Main Recording Flow

**File**: `src/main.rs`

**Key change**: Validate wallpaper resolution BEFORE starting the recording.

```rust
fn main() -> Result<()> {
    // ... existing setup ...

    // Parse wallpaper settings early
    let wallpaper_config = if let Some(wp_value) = args.get_one::<String>("wallpaper") {
        let padding = *args.get_one::<u32>("wallpaper-padding").unwrap();
        Some(parse_wallpaper_config(wp_value, padding, win_id, &api)?)
    } else {
        None
    };

    // ... rest of recording logic ...

    // Apply wallpaper effect (after recording completes)
    if let Some(config) = &wallpaper_config {
        apply_wallpaper_effect(
            &time_codes.lock().unwrap(),
            tempdir.lock().unwrap().borrow(),
            &config.wallpaper,
            config.padding,
        );
    }
}

struct WallpaperConfig {
    wallpaper: DynamicImage,
    padding: u32,
}

fn parse_wallpaper_config(
    value: &str,
    padding: u32,
    win_id: WindowId,
    api: &impl PlatformApi,
) -> Result<WallpaperConfig> {
    // Get terminal window dimensions for validation
    let (terminal_width, terminal_height) = api.get_window_dimensions(win_id)?;

    let wallpaper = if is_builtin_wallpaper(value) {
        match value.to_lowercase().as_str() {
            "ventura" => get_ventura_wallpaper().clone(),
            _ => bail!("Unknown built-in wallpaper: {}", value),
        }
    } else {
        // Custom wallpaper path - validate before recording
        let path = Path::new(value);
        load_and_validate_wallpaper(path, terminal_width, terminal_height, padding)?
    };

    Ok(WallpaperConfig { wallpaper, padding })
}
```

### Step 4: Add Window Dimensions to Platform API

**File**: `src/common/platform_api.rs`

Add method to get window dimensions for pre-validation:

```rust
pub trait PlatformApi {
    // ... existing methods ...

    /// Get the dimensions of a window (width, height)
    fn get_window_dimensions(&self, win_id: WindowId) -> Result<(u32, u32)>;
}
```

**macOS implementation** (`src/macos/mod.rs`):
```rust
fn get_window_dimensions(&self, win_id: WindowId) -> Result<(u32, u32)> {
    // Use CGWindowListCopyWindowInfo or similar to get window bounds
    // Extract width and height from the bounds
}
```

**Linux implementation** (`src/linux/mod.rs`):
```rust
fn get_window_dimensions(&self, win_id: WindowId) -> Result<(u32, u32)> {
    // Use X11 XGetWindowAttributes or similar
}
```

### Step 5: Update Cargo.toml

**File**: `Cargo.toml`

Ensure PNG support is enabled (already have jpeg, tga, bmp):

```toml
[dependencies.image]
version = "0.25"
default-features = false
features = ["bmp", "tga", "jpeg", "png"]
```

### Step 6: Update Wallpapers Module

**File**: `src/wallpapers/mod.rs`

```rust
mod validation;
mod ventura;

pub use validation::{is_builtin_wallpaper, load_and_validate_wallpaper};
pub use ventura::{apply_ventura_wallpaper_effect, get_ventura_wallpaper};

// ... existing apply_wallpaper_effect function ...
```

---

## Security Considerations

### Input Validation

1. **Path Traversal**:
   - Risk: User provides path like `../../etc/passwd`
   - Mitigation: We only READ the file, never write. The image crate will fail to decode non-image files.
   - Additional: Consider canonicalizing the path and warning if it's outside expected directories.

2. **File Extension Validation**:
   - Risk: User provides malicious file with fake extension
   - Mitigation: We validate extension first, then let the image crate validate the actual format during decode. If format doesn't match, decode fails safely.

3. **Memory Exhaustion**:
   - Risk: User provides extremely large image
   - Mitigation: The image crate handles allocation. There's no upper bound check, but users loading huge images would impact their own system.

4. **Symlink Following**:
   - Risk: Symlink to sensitive file
   - Mitigation: We only read as image, non-image files fail to decode. Consider using `fs::metadata()` to check for symlinks if paranoid.

5. **Format-Specific Vulnerabilities**:
   - Risk: Malformed PNG/JPEG could exploit decoder bugs
   - Mitigation: Use well-maintained `image` crate from Rust ecosystem. Keep dependencies updated.

### Error Messages

- Never include full system paths in error messages if they could reveal sensitive information
- Provide actionable error messages (what went wrong, what to do)

---

## File Changes Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `src/cli.rs` | Modify | Change `--wallpaper` to accept string (preset or path) |
| `src/wallpapers/validation.rs` | **New** | Wallpaper loading and validation logic |
| `src/wallpapers/mod.rs` | Modify | Export validation functions |
| `src/main.rs` | Modify | Add pre-recording wallpaper validation |
| `src/common/platform_api.rs` | Modify | Add `get_window_dimensions()` trait method |
| `src/macos/mod.rs` | Modify | Implement `get_window_dimensions()` |
| `src/linux/mod.rs` | Modify | Implement `get_window_dimensions()` |
| `Cargo.toml` | Modify | Add `png` feature to image crate |

---

## Testing Plan

### Unit Tests

1. **Extension validation**: Test all supported extensions (png, jpg, jpeg, tga)
2. **Extension rejection**: Test unsupported extensions (gif, webp, bmp, svg)
3. **Case insensitivity**: Test `PNG`, `Png`, `png`
4. **Built-in detection**: Test `is_builtin_wallpaper("ventura")` returns true

### Integration Tests

1. **Valid custom wallpaper**: Provide valid PNG, verify it loads
2. **Non-existent file**: Verify clear error message
3. **Too small wallpaper**: Verify resolution error with helpful message
4. **Wrong format**: Provide text file with .png extension, verify decode error

### Manual Tests

1. **Happy path**: `t-rec --wallpaper ~/Pictures/my-wallpaper.png`
2. **Built-in still works**: `t-rec --wallpaper ventura`
3. **Custom with padding**: `t-rec --wallpaper ~/wp.jpg --wallpaper-padding 100`
4. **Error case**: Try with wallpaper smaller than terminal

---

## CLI Usage Examples

```bash
# Built-in wallpaper (existing behavior)
t-rec --wallpaper ventura
t-rec -p ventura

# Custom wallpaper from file path
t-rec --wallpaper ~/Pictures/my-wallpaper.png
t-rec -p /path/to/wallpaper.jpg

# Custom wallpaper with adjusted padding
t-rec --wallpaper ~/wp.png --wallpaper-padding 100

# Relative path
t-rec --wallpaper ./backgrounds/custom.tga
```

---

## Error Messages

### File not found
```
Error: Wallpaper file not found: /path/to/wallpaper.png
```

### Unsupported format
```
Error: Unsupported wallpaper format 'gif'. Supported formats: png, jpg, jpeg, tga
```

### Resolution too small
```
Error: Wallpaper resolution 800x600 is too small.
Required: at least 1144x888 (terminal 1024x768 + 60px padding on each side).
Please use a larger wallpaper image.
```

### Decode failure
```
Error: Failed to decode wallpaper image: /path/to/file.png
Caused by: Invalid PNG signature
```

---

## Future Enhancements (Out of Scope)

- **Wallpaper scaling**: Automatically scale wallpaper if slightly too small
- **Aspect ratio handling**: Crop or letterbox wallpapers with different aspect ratios
- **URL support**: Download wallpaper from URL (explicitly not wanted per issue - offline operation)
- **Wallpaper caching**: Cache decoded custom wallpapers for repeated use
