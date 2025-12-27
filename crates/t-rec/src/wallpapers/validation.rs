use std::path::Path;

use anyhow::{bail, Context, Result};
use image::{DynamicImage, GenericImageView, ImageReader};

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
            wp_width,
            wp_height,
            min_width,
            min_height,
            terminal_width,
            terminal_height,
            padding
        );
    }

    Ok(wallpaper)
}

/// Check if a wallpaper value is a built-in preset or a custom path
pub fn is_builtin_wallpaper(value: &str) -> bool {
    matches!(value.to_lowercase().as_str(), "ventura")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_builtin_wallpaper() {
        assert!(is_builtin_wallpaper("ventura"));
        assert!(is_builtin_wallpaper("Ventura"));
        assert!(is_builtin_wallpaper("VENTURA"));
        assert!(!is_builtin_wallpaper("/path/to/file.png"));
        assert!(!is_builtin_wallpaper("./wallpaper.jpg"));
        assert!(!is_builtin_wallpaper("custom"));
    }
}
