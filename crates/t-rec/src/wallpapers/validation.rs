use std::path::Path;

use anyhow::{bail, Context, Result};
use image::{DynamicImage, GenericImageView, ImageReader};

use super::types::Wallpaper;
use super::ventura::get_ventura_wallpaper;

/// Supported wallpaper formats
const SUPPORTED_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "tga"];

/// Validate that a wallpaper image has sufficient dimensions for the terminal size + padding.
///
/// This is the core validation logic used by both built-in and custom wallpapers.
pub fn validate_wallpaper_dimensions(
    image: &DynamicImage,
    terminal_width: u32,
    terminal_height: u32,
    padding: u32,
) -> Result<()> {
    let (wp_width, wp_height) = image.dimensions();
    let min_width = terminal_width + (padding * 2);
    let min_height = terminal_height + (padding * 2);

    if wp_width < min_width || wp_height < min_height {
        bail!(
            "Wallpaper resolution {}x{} is too small.\n\
             Required: at least {}x{} (terminal {}x{} + {}px padding on each side).\n\
             Please use a larger wallpaper image or reduce the terminal size/padding.",
            wp_width,
            wp_height,
            min_width,
            min_height,
            terminal_width,
            terminal_height,
            padding
        );
    }

    Ok(())
}

/// Resolve a wallpaper from a `Wallpaper` enum variant.
///
/// This is the main entry point for wallpaper loading and validation.
/// It handles both built-in wallpapers and custom file paths.
///
/// # Arguments
/// * `wallpaper` - The wallpaper variant to resolve
/// * `terminal_width` - Width of the terminal in pixels
/// * `terminal_height` - Height of the terminal in pixels
/// * `padding` - Padding in pixels to add around the terminal
///
/// # Returns
/// A validated `DynamicImage` ready for use as a wallpaper background.
pub fn resolve_wallpaper(
    wallpaper: &Wallpaper,
    terminal_width: u32,
    terminal_height: u32,
    padding: u32,
) -> Result<DynamicImage> {
    let image = match wallpaper {
        Wallpaper::Ventura => get_ventura_wallpaper().clone(),
        Wallpaper::Custom(path) => load_wallpaper_from_path(path.as_path())?,
    };

    validate_wallpaper_dimensions(&image, terminal_width, terminal_height, padding)?;
    Ok(image)
}

/// Load a wallpaper image from a file path.
///
/// # Validation checks:
/// 1. File exists and is readable
/// 2. File extension is supported (png, jpg, jpeg, tga)
/// 3. File is a valid image that can be decoded
///
/// Note: Dimension validation is NOT performed here - use `resolve_wallpaper` for full validation.
fn load_wallpaper_from_path(path: &Path) -> Result<DynamicImage> {
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
    ImageReader::open(path)
        .with_context(|| format!("Failed to open wallpaper file: {}", path.display()))?
        .with_guessed_format()
        .with_context(|| "Failed to detect wallpaper image format")?
        .decode()
        .with_context(|| format!("Failed to decode wallpaper image: {}", path.display()))
}

/// Check if a wallpaper value string is a built-in preset.
///
/// Uses `Wallpaper::builtin_values()` as the single source of truth.
pub fn is_builtin_wallpaper(value: &str) -> bool {
    Wallpaper::builtin_values()
        .iter()
        .any(|&builtin| builtin.eq_ignore_ascii_case(value))
}

/// Validate and load a custom wallpaper from a file path.
///
/// This is a convenience function that combines loading and dimension validation.
/// Prefer using `resolve_wallpaper` with a `Wallpaper` enum when possible.
///
/// # Validation checks:
/// 1. File exists and is readable
/// 2. File extension is supported (png, jpg, jpeg, tga)
/// 3. File is a valid image that can be decoded
/// 4. Resolution is sufficient for the terminal size + padding
pub fn load_and_validate_wallpaper(
    path: &Path,
    terminal_width: u32,
    terminal_height: u32,
    padding: u32,
) -> Result<DynamicImage> {
    let image = load_wallpaper_from_path(path)?;
    validate_wallpaper_dimensions(&image, terminal_width, terminal_height, padding)?;
    Ok(image)
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

    #[test]
    fn test_is_builtin_uses_wallpaper_type() {
        // Verify that is_builtin_wallpaper stays in sync with Wallpaper::builtin_values()
        for &builtin in Wallpaper::builtin_values() {
            assert!(
                is_builtin_wallpaper(builtin),
                "is_builtin_wallpaper should return true for '{}'",
                builtin
            );
        }
    }
}
