//! Post-processing effects for frames and screenshots.
//!
//! This module provides a unified pipeline for applying visual effects
//! to captured frames and screenshots.
//!
//! # Type-Safe Configuration
//!
//! Post-processing options use type-safe enums for configuration:
//! - [`Decor`](crate::types::Decor) for decoration style (None, Shadow)
//! - [`BackgroundColor`](crate::types::BackgroundColor) for background color

use image::{DynamicImage, GenericImageView};
use log::warn;
use rayon::prelude::*;
use std::path::Path;

use super::decors::{apply_corner_to_file, apply_shadow_to_file};
#[cfg(feature = "cli")]
use super::screenshot::ScreenshotInfo;
use super::types::{BackgroundColor, Decor};
use super::wallpapers::composite_frame;
use crate::Result;

/// Options for post-processing effects.
///
/// These options control which effects are applied to frames/screenshots.
/// Uses type-safe enums for decoration and background color configuration.
///
/// # Example
///
/// ```ignore
/// use t_rec::post_processing::PostProcessingOptions;
/// use t_rec::types::{Decor, BackgroundColor};
///
/// let opts = PostProcessingOptions::new(Decor::Shadow, &BackgroundColor::White);
/// ```
#[derive(Clone)]
pub struct PostProcessingOptions<'a> {
    /// Decoration style (None or Shadow)
    pub decor: Decor,
    /// Background color for shadow effect
    pub bg_color: &'a BackgroundColor,
    /// Optional wallpaper configuration (image, padding)
    pub wallpaper: Option<(&'a DynamicImage, u32)>,
}

impl<'a> PostProcessingOptions<'a> {
    /// Create new post-processing options with type-safe enums.
    pub fn new(decor: Decor, bg_color: &'a BackgroundColor) -> Self {
        Self {
            decor,
            bg_color,
            wallpaper: None,
        }
    }

    /// Create post-processing options from string values.
    ///
    /// This is primarily for backward compatibility with CLI/config string values.
    /// Prefer [`new`](Self::new) for programmatic use.
    ///
    /// # Panics
    ///
    /// Panics if `decor` is not a valid decoration value.
    #[cfg(test)]
    pub fn from_strings(decor: &str, bg_color: &'a BackgroundColor) -> Self {
        Self {
            decor: decor.parse().unwrap_or_else(|_| {
                panic!(
                    "Invalid decor '{}'. Valid options: {}",
                    decor,
                    Decor::valid_values().join(", ")
                )
            }),
            bg_color,
            wallpaper: None,
        }
    }

    /// Set wallpaper configuration.
    pub fn with_wallpaper(mut self, wallpaper: &'a DynamicImage, padding: u32) -> Self {
        self.wallpaper = Some((wallpaper, padding));
        self
    }
}

/// Process screenshots with effects (CLI only).
#[cfg(feature = "cli")]
pub fn post_process_screenshots(
    screenshots: &[ScreenshotInfo],
    target: &str,
    opts: &PostProcessingOptions,
) -> Vec<String> {
    let mut saved_screenshots = Vec::new();

    let name_file = |target: &str, timecode_ms: u128| {
        super::screenshot::screenshot_output_name(target, timecode_ms, "png")
    };

    screenshots.iter().for_each(|screenshot| {
        let file = &screenshot.temp_path;
        let timecode_ms = screenshot.timecode_ms;
        if let Err(e) = post_process_file(file.as_ref(), opts) {
            warn!("Failed to apply effects to {file:?}: {e}");
        } else {
            let output_name = name_file(target, timecode_ms);
            match image::open(&screenshot.temp_path) {
                Ok(img) => {
                    if let Err(e) = img.save(&output_name) {
                        log::error!("Failed to save screenshot: {}", e);
                    } else {
                        saved_screenshots.push(output_name);
                    }
                }
                Err(e) => log::error!(
                    "Failed to read screenshot from {:?}: {}",
                    screenshot.temp_path,
                    e
                ),
            }
        }
    });

    saved_screenshots
}

/// Apply post-processing effects to a single file.
///
/// Applies the following effects in order:
/// 1. Corner radius effect (rounded corners)
/// 2. Shadow effect (if decor is Shadow)
/// 3. Wallpaper compositing (if wallpaper is configured)
pub fn post_process_file(file: &Path, opts: &PostProcessingOptions) -> Result<()> {
    // Apply corner effect
    apply_corner_to_file(file)?;

    // Apply shadow effect if enabled
    if opts.decor == Decor::Shadow {
        apply_shadow_to_file(file, opts.bg_color.to_imagemagick_color())?;
    }

    // Apply wallpaper effect if configured
    if let Some((wallpaper, padding)) = opts.wallpaper {
        let (wp_width, wp_height) = wallpaper.dimensions();
        composite_frame(file, wallpaper, wp_width, wp_height, padding)?;
    }

    Ok(())
}

/// Apply post-processing effects to multiple files in parallel.
///
/// This is the main entry point for batch processing frames.
pub fn post_process_effects<P: AsRef<Path> + Sync>(files: &[P], opts: &PostProcessingOptions) {
    files.par_iter().for_each(|file| {
        if let Err(e) = post_process_file(file.as_ref(), opts) {
            warn!("Failed to apply effects to {:?}: {}", file.as_ref(), e);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_processing_options_new() {
        let bg = BackgroundColor::White;
        let opts = PostProcessingOptions::new(Decor::Shadow, &bg);
        assert_eq!(opts.decor, Decor::Shadow);
        assert_eq!(opts.bg_color.to_imagemagick_color(), "white");
        assert!(opts.wallpaper.is_none());
    }

    #[test]
    fn test_post_processing_options_none_decor() {
        let bg = BackgroundColor::Black;
        let opts = PostProcessingOptions::new(Decor::None, &bg);
        assert_eq!(opts.decor, Decor::None);
    }

    #[test]
    fn test_post_processing_options_from_strings() {
        let bg = BackgroundColor::Transparent;
        let opts = PostProcessingOptions::from_strings("shadow", &bg);
        assert_eq!(opts.decor, Decor::Shadow);
    }

    #[test]
    fn test_post_processing_options_custom_color() {
        let bg = BackgroundColor::custom("#ff5500").unwrap();
        let opts = PostProcessingOptions::new(Decor::Shadow, &bg);
        assert_eq!(opts.bg_color.to_imagemagick_color(), "#ff5500");
    }
}
