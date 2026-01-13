use std::path::Path;

use crate::core::Result;

/// Corner radius in pixels (matches macOS Big Sur terminal style)
const RADIUS: u32 = 13;

/// Apply corner radius effect to a single file.
pub fn apply_corner_to_file(file: &Path) -> Result<()> {
    if cfg!(feature = "x-native-imgops") {
        native::apply_corner_to_file(file)
    } else {
        imagemagick::apply_corner_to_file(file)
    }
}

/// Native implementation (for benchmarking)
pub fn apply_corner_to_file_native(file: &Path) -> Result<()> {
    native::apply_corner_to_file(file)
}

/// ImageMagick implementation (for benchmarking)
pub fn apply_corner_to_file_imagemagick(file: &Path) -> Result<()> {
    imagemagick::apply_corner_to_file(file)
}

/// Native implementation using the image crate with anti-aliased corners.
mod native {
    use super::*;
    use anyhow::Context;
    use image::{save_buffer, ColorType, Rgba};

    /// Applies rounded corners with anti-aliasing by smoothly blending
    /// the alpha channel for pixels near the corner radius edge.
    pub fn apply_corner_to_file(file: &Path) -> Result<()> {
        let img = image::open(file).with_context(|| {
            format!(
                "Cannot open image file for corner effect: {}",
                file.display()
            )
        })?;

        let mut img = img.to_rgba8();
        let (width, height) = img.dimensions();
        let radius = RADIUS as f32;

        // Apply rounded corners with anti-aliasing
        for y in 0..height {
            for x in 0..width {
                let coverage = corner_coverage(x, y, width, height, radius);
                if coverage < 1.0 {
                    let pixel = img.get_pixel(x, y);
                    let new_alpha = (pixel[3] as f32 * coverage) as u8;
                    img.put_pixel(x, y, Rgba([pixel[0], pixel[1], pixel[2], new_alpha]));
                }
            }
        }

        save_buffer(file, img.as_raw(), width, height, ColorType::Rgba8).with_context(|| {
            format!(
                "Cannot save image file after corner effect: {}",
                file.display()
            )
        })?;

        Ok(())
    }

    /// Calculate how much of a pixel is covered by the rounded corner area.
    ///
    /// Returns:
    /// - 1.0 = fully inside (pixel unchanged)
    /// - 0.0 = fully outside (pixel transparent)
    /// - 0.0-1.0 = on the edge (smooth anti-aliased blend)
    fn corner_coverage(x: u32, y: u32, width: u32, height: u32, radius: f32) -> f32 {
        let x = x as f32; // use pixel coordinate (matches ImageMagick)
        let y = y as f32;
        let w = width as f32;
        let h = height as f32;

        // Top-left corner: circle center at (radius, radius)
        if x < radius && y < radius {
            return circle_coverage(x, y, radius, radius, radius);
        }

        // Top-right corner: circle center at (width - radius, radius)
        if x > w - radius && y < radius {
            return circle_coverage(x, y, w - radius, radius, radius);
        }

        // Bottom-left corner: circle center at (radius, height - radius)
        if x < radius && y > h - radius {
            return circle_coverage(x, y, radius, h - radius, radius);
        }

        // Bottom-right corner: circle center at (width - radius, height - radius)
        if x > w - radius && y > h - radius {
            return circle_coverage(x, y, w - radius, h - radius, radius);
        }

        1.0 // Not in a corner region
    }

    /// Calculate coverage for a pixel relative to a circle using signed distance.
    fn circle_coverage(px: f32, py: f32, cx: f32, cy: f32, radius: f32) -> f32 {
        let dx = px - cx;
        let dy = py - cy;
        let distance = (dx * dx + dy * dy).sqrt();

        // Signed distance from circle edge (negative = inside, positive = outside)
        let signed_distance = distance - radius;

        // Smooth falloff over ~1 pixel for anti-aliasing
        // Bias inward so pixels on the edge stay opaque (matches ImageMagick rasterization)
        (1.0 - signed_distance).clamp(0.0, 1.0)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use image::{save_buffer, ColorType, Rgba, RgbaImage};
        use tempfile::NamedTempFile;

        #[test]
        fn test_corners_are_transparent() -> Result<()> {
            let mut img = RgbaImage::new(30, 30);
            for pixel in img.pixels_mut() {
                *pixel = Rgba([255, 0, 0, 255]);
            }

            let temp = NamedTempFile::with_suffix(".bmp")?;
            let (w, h) = img.dimensions();
            save_buffer(temp.path(), img.as_raw(), w, h, ColorType::Rgba8)?;

            apply_corner_to_file(temp.path())?;

            let result = image::open(temp.path())?.to_rgba8();

            // Corners should be fully transparent
            assert_eq!(result.get_pixel(0, 0)[3], 0, "top-left corner");
            assert_eq!(result.get_pixel(29, 0)[3], 0, "top-right corner");
            assert_eq!(result.get_pixel(0, 29)[3], 0, "bottom-left corner");
            assert_eq!(result.get_pixel(29, 29)[3], 0, "bottom-right corner");

            // Center should remain opaque
            assert_eq!(result.get_pixel(15, 15)[3], 255, "center");

            Ok(())
        }

        #[test]
        fn test_antialiasing_produces_partial_alpha() -> Result<()> {
            let mut img = RgbaImage::new(50, 50);
            for pixel in img.pixels_mut() {
                *pixel = Rgba([255, 0, 0, 255]);
            }

            let temp = NamedTempFile::with_suffix(".bmp")?;
            let (w, h) = img.dimensions();
            save_buffer(temp.path(), img.as_raw(), w, h, ColorType::Rgba8)?;

            apply_corner_to_file(temp.path())?;

            let result = image::open(temp.path())?.to_rgba8();

            // Find a pixel on the edge that should have partial alpha
            // At radius=13, corner center is at (13, 13)
            // For y=1 (center 1.5), distance to circle edge is at x ≈ 7
            // Check multiple pixels along the edge to find one with partial alpha
            let mut found_partial = false;
            for x in 0..13u32 {
                for y in 0..13u32 {
                    let alpha = result.get_pixel(x, y)[3];
                    if alpha > 0 && alpha < 255 {
                        found_partial = true;
                        break;
                    }
                }
                if found_partial {
                    break;
                }
            }

            assert!(
                found_partial,
                "should find at least one pixel with partial alpha in corner region"
            );

            Ok(())
        }

        #[test]
        fn test_circle_coverage() {
            // Pixel well inside the circle
            let inside = circle_coverage(5.5, 5.5, 13.0, 13.0, 13.0);
            assert!((inside - 1.0).abs() < 0.01, "inside should be ~1.0");

            // Pixel well outside the circle
            let outside = circle_coverage(0.5, 0.5, 13.0, 13.0, 13.0);
            assert!(outside < 0.01, "outside should be ~0.0");

            // Pixel exactly on the edge should be ~1.0 (biased inward)
            let on_edge = circle_coverage(13.0, 0.0, 13.0, 13.0, 13.0);
            assert!(
                (on_edge - 1.0).abs() < 0.01,
                "edge should be ~1.0, got {}",
                on_edge
            );

            // Pixel 0.5 outside the edge should be ~0.5
            let half_out = circle_coverage(13.0, -0.5, 13.0, 13.0, 13.0);
            assert!(
                half_out > 0.4 && half_out < 0.6,
                "half pixel outside should be ~0.5, got {}",
                half_out
            );
        }

    }
}

/// ImageMagick implementation (original).
mod imagemagick {
    use super::*;
    use anyhow::Context;
    use std::process::Command;

    pub fn apply_corner_to_file(file: &Path) -> Result<()> {
        let e = Command::new("convert")
            .arg(file.to_str().unwrap())
            .arg("(")
            .args(["+clone", "-alpha", "extract"])
            .args([
                "-draw",
                &format!(
                    "fill black polygon 0,0 0,{r} {r},0 fill white circle {r},{r} {r},0",
                    r = RADIUS
                ),
            ])
            .args(["(", "+clone", "-flip", ")"])
            .args(["-compose", "Multiply", "-composite"])
            .args(["(", "+clone", "-flop", ")"])
            .args(["-compose", "Multiply", "-composite"])
            .arg(")")
            .args(["-alpha", "off", "-compose", "CopyOpacity", "-composite"])
            .arg(file.to_str().unwrap())
            .output()
            .context("Cannot apply corner decor effect")?;

        if !e.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&e.stderr))
        } else {
            Ok(())
        }
    }
}
