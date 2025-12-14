//! Post-processing effects for frames and screenshots.
//!
//! This module provides a unified pipeline for applying visual effects
//! to captured frames and screenshots.

use image::{DynamicImage, GenericImageView};
use log::warn;
use rayon::prelude::*;
use std::path::{Path, PathBuf};

use crate::decors::{apply_corner_to_file, apply_shadow_to_file};
use crate::screenshot::ScreenshotInfo;
use crate::wallpapers::composite_frame;
use crate::Result;

/// Options for post-processing effects.
///
/// These options control which effects are applied to frames/screenshots.
#[derive(Clone)]
pub struct PostProcessingOptions<'a> {
    /// Decoration style ("none", "shadow", etc.)
    pub decor: &'a str,
    /// Background color for shadow effect
    pub bg_color: &'a str,
    /// Optional wallpaper configuration (image, padding)
    pub wallpaper: Option<(&'a DynamicImage, u32)>,
}

impl<'a> PostProcessingOptions<'a> {
    /// Create new post-processing options.
    pub fn new(decor: &'a str, bg_color: &'a str) -> Self {
        Self {
            decor,
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

pub fn post_process_screenshots(
    screenshots: &[ScreenshotInfo],
    target: &str,
    opts: &PostProcessingOptions,
) -> Vec<String> {
    let mut saved_screenshots = Vec::new();

    let name_file = |target: &str, timecode_ms: u128| {
        crate::screenshot::screenshot_output_name(target, timecode_ms, "png")
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
/// 2. Shadow effect (if decor == "shadow")
/// 3. Wallpaper compositing (if wallpaper is configured)
pub fn post_process_file(file: &Path, opts: &PostProcessingOptions) -> Result<()> {
    // Apply corner effect
    apply_corner_to_file(file)?;

    // Apply shadow effect if enabled
    if opts.decor == "shadow" {
        apply_shadow_to_file(file, opts.bg_color)?;
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
        let opts = PostProcessingOptions::new("shadow", "#ffffff");
        assert_eq!(opts.decor, "shadow");
        assert_eq!(opts.bg_color, "#ffffff");
        assert!(opts.wallpaper.is_none());
    }

    #[test]
    fn test_post_processing_options_default_decor() {
        let opts = PostProcessingOptions::new("none", "#000000");
        assert_eq!(opts.decor, "none");
    }
}
