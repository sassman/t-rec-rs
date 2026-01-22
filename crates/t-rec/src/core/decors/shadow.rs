use std::path::Path;

use crate::core::Result;

/// Shadow blur sigma (matches ImageMagick -shadow 100x20)
const SHADOW_SIGMA: f32 = 20.0;

/// Padding around image for shadow spread (2 × sigma per ImageMagick docs)
const SHADOW_PADDING: u32 = 40;

/// Apply shadow effect to a single file.
pub fn apply_shadow_to_file(file: &Path, bg_color: &str) -> Result<()> {
    if cfg!(feature = "x-native-imgops") {
        native::apply_shadow_to_file(file, bg_color)
    } else {
        imagemagick::apply_shadow_to_file(file, bg_color)
    }
}

/// Native implementation (for benchmarking)
#[allow(dead_code)]
pub fn apply_shadow_to_file_native(file: &Path, bg_color: &str) -> Result<()> {
    native::apply_shadow_to_file(file, bg_color)
}

/// ImageMagick implementation (for benchmarking)
#[allow(dead_code)]
pub fn apply_shadow_to_file_imagemagick(file: &Path, bg_color: &str) -> Result<()> {
    imagemagick::apply_shadow_to_file(file, bg_color)
}

/// Native implementation using the image crate.
mod native {
    use super::*;
    use anyhow::Context;
    use image::{save_buffer, ColorType, Rgba, RgbaImage};

    pub fn apply_shadow_to_file(file: &Path, bg_color: &str) -> Result<()> {
        let img = image::open(file).with_context(|| {
            format!(
                "Cannot open image file for shadow effect: {}",
                file.display()
            )
        })?;

        let img = img.to_rgba8();
        let (width, height) = img.dimensions();

        // Parse background color
        let bg = parse_hex_color(bg_color).unwrap_or(Rgba([255, 255, 255, 255]));

        // Create larger canvas with padding for shadow
        let canvas_width = width + SHADOW_PADDING * 2;
        let canvas_height = height + SHADOW_PADDING * 2;
        let mut canvas = RgbaImage::from_pixel(canvas_width, canvas_height, bg);

        // Create shadow layer (alpha from original, blurred)
        let shadow = create_shadow(&img, canvas_width, canvas_height, SHADOW_PADDING);

        // Composite shadow onto canvas
        for y in 0..canvas_height {
            for x in 0..canvas_width {
                let shadow_pixel = shadow.get_pixel(x, y);
                if shadow_pixel[3] > 0 {
                    let canvas_pixel = canvas.get_pixel(x, y);
                    let blended = alpha_blend(shadow_pixel, canvas_pixel);
                    canvas.put_pixel(x, y, blended);
                }
            }
        }

        // Composite original image on top (centered)
        for y in 0..height {
            for x in 0..width {
                let src = img.get_pixel(x, y);
                let dst_x = x + SHADOW_PADDING;
                let dst_y = y + SHADOW_PADDING;
                let dst = canvas.get_pixel(dst_x, dst_y);
                canvas.put_pixel(dst_x, dst_y, alpha_blend(src, dst));
            }
        }

        save_buffer(
            file,
            canvas.as_raw(),
            canvas_width,
            canvas_height,
            ColorType::Rgba8,
        )
        .with_context(|| {
            format!(
                "Cannot save image file after shadow effect: {}",
                file.display()
            )
        })?;

        Ok(())
    }

    /// Creates a blurred shadow from the original image's alpha channel.
    ///
    /// The shadow effect works by:
    /// 1. Extracting the alpha channel from the source image into a float buffer
    /// 2. Centering it on a larger canvas (with padding for the blur spread)
    /// 3. Applying a Gaussian blur to create the soft shadow falloff
    /// 4. Converting the blurred alpha back to a black RGBA image
    ///
    /// The result is a semi-transparent black image where opacity decreases
    /// smoothly from the original shape's edges outward.
    fn create_shadow(img: &RgbaImage, canvas_w: u32, canvas_h: u32, padding: u32) -> RgbaImage {
        let (w, h) = img.dimensions();

        // Create alpha mask on canvas-sized buffer
        let mut alpha_buffer: Vec<f32> = vec![0.0; (canvas_w * canvas_h) as usize];

        // Copy alpha values to buffer (centered with padding)
        for y in 0..h {
            for x in 0..w {
                let alpha = img.get_pixel(x, y)[3] as f32 / 255.0;
                let idx = ((y + padding) * canvas_w + (x + padding)) as usize;
                alpha_buffer[idx] = alpha;
            }
        }

        // Apply Gaussian blur (separable: horizontal then vertical)
        let kernel = gaussian_kernel(SHADOW_SIGMA);
        let blurred = blur_separable(&alpha_buffer, canvas_w, canvas_h, &kernel);

        // Convert to shadow image (black with blurred alpha)
        let mut shadow = RgbaImage::new(canvas_w, canvas_h);
        for y in 0..canvas_h {
            for x in 0..canvas_w {
                let idx = (y * canvas_w + x) as usize;
                let alpha = (blurred[idx] * 255.0).min(255.0) as u8;
                shadow.put_pixel(x, y, Rgba([0, 0, 0, alpha]));
            }
        }

        shadow
    }

    /// Generates a 1D Gaussian kernel for convolution.
    ///
    /// The Gaussian function G(x) = e^(-x²/2σ²) creates a bell curve where:
    /// - σ (sigma) controls the spread/width of the blur
    /// - Higher σ = wider blur, softer shadow edges
    /// - Lower σ = tighter blur, sharper shadow edges
    ///
    /// The kernel radius is set to 3σ because the Gaussian function drops to
    /// ~0.01% of its peak at 3σ, making values beyond negligible (99.7% rule).
    ///
    /// The kernel is normalized (sums to 1.0) to preserve overall brightness.
    fn gaussian_kernel(sigma: f32) -> Vec<f32> {
        let radius = (sigma * 3.0).ceil() as i32;
        let size = (radius * 2 + 1) as usize;
        let mut kernel = vec![0.0; size];
        let mut sum = 0.0;

        for (i, item) in kernel.iter_mut().enumerate().take(size) {
            let x = (i as i32 - radius) as f32;
            let val = (-x * x / (2.0 * sigma * sigma)).exp();
            *item = val;
            sum += val;
        }

        // Normalize
        for v in &mut kernel {
            *v /= sum;
        }

        kernel
    }

    /// Applies Gaussian blur using the separable filter optimization.
    ///
    /// # Why Separable?
    /// A 2D Gaussian blur is "separable", meaning it can be decomposed into
    /// two 1D passes (horizontal then vertical). This reduces complexity from
    /// O(width × height × kernel_size²) to O(width × height × kernel_size × 2).
    ///
    /// For a kernel size of 121 (σ=20), this is ~60x faster than naive 2D convolution.
    ///
    /// # How It Works
    /// ```text
    /// Original → [Horizontal 1D blur] → Intermediate → [Vertical 1D blur] → Result
    /// ```
    ///
    /// Each pass slides the 1D kernel across the image:
    /// - Horizontal: for each pixel, sum weighted neighbors along the row
    /// - Vertical: for each pixel, sum weighted neighbors along the column
    ///
    /// # Parallelization
    /// Both passes are parallelized over rows using rayon's `par_chunks_mut`.
    /// Each row's computation is independent, making this embarrassingly parallel.
    /// Edge pixels use clamped sampling (repeat edge values) to avoid artifacts.
    fn blur_separable(input: &[f32], width: u32, height: u32, kernel: &[f32]) -> Vec<f32> {
        use rayon::prelude::*;

        let radius = (kernel.len() / 2) as i32;
        let w = width as usize;
        let wi = width as i32;
        let hi = height as i32;

        // Horizontal pass - parallelize over rows
        let mut temp = vec![0.0; input.len()];
        temp.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
            for (x, px) in row.iter_mut().enumerate().take(w) {
                let mut sum = 0.0;
                for (i, &k) in kernel.iter().enumerate() {
                    let sx = (x as i32 + i as i32 - radius).clamp(0, wi - 1) as usize;
                    sum += input[y * w + sx] * k;
                }
                *px = sum;
            }
        });

        // Vertical pass - parallelize over rows
        let mut output = vec![0.0; input.len()];
        output.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
            for x in 0..w {
                let mut sum = 0.0;
                for (i, &k) in kernel.iter().enumerate() {
                    let sy = (y as i32 + i as i32 - radius).clamp(0, hi - 1) as usize;
                    sum += temp[sy * w + x] * k;
                }
                row[x] = sum;
            }
        });

        output
    }

    /// Composites source over destination using Porter-Duff "over" operator.
    ///
    /// This is the standard alpha compositing formula used in image editing:
    /// ```text
    /// out_alpha = src_alpha + dst_alpha × (1 - src_alpha)
    /// out_color = (src_color × src_alpha + dst_color × dst_alpha × (1 - src_alpha)) / out_alpha
    /// ```
    ///
    /// The formula handles:
    /// - Fully opaque src (α=1): completely replaces dst
    /// - Fully transparent src (α=0): dst shows through unchanged
    /// - Semi-transparent src: blends proportionally with dst
    fn alpha_blend(src: &Rgba<u8>, dst: &Rgba<u8>) -> Rgba<u8> {
        let sa = src[3] as f32 / 255.0;
        let da = dst[3] as f32 / 255.0;

        let out_a = sa + da * (1.0 - sa);
        if out_a == 0.0 {
            return Rgba([0, 0, 0, 0]);
        }

        let blend = |s: u8, d: u8| -> u8 {
            let s = s as f32 / 255.0;
            let d = d as f32 / 255.0;
            let out = (s * sa + d * da * (1.0 - sa)) / out_a;
            (out * 255.0) as u8
        };

        Rgba([
            blend(src[0], dst[0]),
            blend(src[1], dst[1]),
            blend(src[2], dst[2]),
            (out_a * 255.0) as u8,
        ])
    }

    /// Parse hex color string (e.g., "#ffffff" or "white").
    fn parse_hex_color(color: &str) -> Option<Rgba<u8>> {
        let color = color.trim().to_lowercase();

        // Named colors
        match color.as_str() {
            "white" => return Some(Rgba([255, 255, 255, 255])),
            "black" => return Some(Rgba([0, 0, 0, 255])),
            "transparent" => return Some(Rgba([0, 0, 0, 0])),
            _ => {}
        }

        // Hex format
        let hex = color.strip_prefix('#').unwrap_or(&color);
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Rgba([r, g, b, 255]));
        }

        None
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use tempfile::NamedTempFile;

        #[test]
        fn test_shadow_increases_image_size() -> Result<()> {
            // Create small test image
            let mut img = RgbaImage::new(100, 100);
            for pixel in img.pixels_mut() {
                *pixel = Rgba([255, 0, 0, 255]);
            }

            let temp = NamedTempFile::with_suffix(".bmp")?;
            save_buffer(temp.path(), img.as_raw(), 100, 100, ColorType::Rgba8)?;

            apply_shadow_to_file(temp.path(), "white")?;

            let result = image::open(temp.path())?.to_rgba8();
            let (w, h) = result.dimensions();

            // Should be larger due to shadow padding
            assert_eq!(w, 100 + SHADOW_PADDING * 2);
            assert_eq!(h, 100 + SHADOW_PADDING * 2);

            Ok(())
        }

        #[test]
        fn test_shadow_has_gradient_alpha() -> Result<()> {
            let mut img = RgbaImage::new(50, 50);
            for pixel in img.pixels_mut() {
                *pixel = Rgba([255, 0, 0, 255]);
            }

            let temp = NamedTempFile::with_suffix(".bmp")?;
            save_buffer(temp.path(), img.as_raw(), 50, 50, ColorType::Rgba8)?;

            apply_shadow_to_file(temp.path(), "#ffffff")?;

            let result = image::open(temp.path())?.to_rgba8();

            // Check that shadow region has varying alpha (gradient from blur)
            // Sample pixels in the padding area
            let mut found_partial = false;
            for x in 0..SHADOW_PADDING {
                let pixel = result.get_pixel(x, SHADOW_PADDING + 25);
                // Shadow should blend with white background, creating gray tones
                if pixel[0] < 255 && pixel[0] > 0 {
                    found_partial = true;
                    break;
                }
            }

            assert!(
                found_partial,
                "shadow should create gradient in padding area"
            );

            Ok(())
        }

        #[test]
        fn test_parse_hex_color() {
            assert_eq!(parse_hex_color("white"), Some(Rgba([255, 255, 255, 255])));
            assert_eq!(parse_hex_color("#ff0000"), Some(Rgba([255, 0, 0, 255])));
            assert_eq!(parse_hex_color("000000"), Some(Rgba([0, 0, 0, 255])));
            assert_eq!(parse_hex_color("#ABC123"), Some(Rgba([171, 193, 35, 255])));
        }

        #[test]
        fn test_gaussian_kernel_sums_to_one() {
            let kernel = gaussian_kernel(10.0);
            let sum: f32 = kernel.iter().sum();
            assert!(
                (sum - 1.0).abs() < 0.001,
                "kernel should sum to 1.0, got {}",
                sum
            );
        }
    }
}

/// ImageMagick implementation (original).
mod imagemagick {
    use super::*;
    use anyhow::Context;
    use std::process::Command;

    pub fn apply_shadow_to_file(file: &Path, bg_color: &str) -> Result<()> {
        let e = Command::new("convert")
            .arg(file.to_str().unwrap())
            .arg("(")
            .args(["+clone", "-background", "black", "-shadow", "100x20+0+0"])
            .arg(")")
            .args(["+swap", "-background", bg_color])
            .args(["-layers", "merge"])
            .arg(file.to_str().unwrap())
            .output()
            .context("Cannot apply shadow decor effect")?;

        if !e.status.success() {
            anyhow::bail!("{}", String::from_utf8_lossy(&e.stderr))
        } else {
            Ok(())
        }
    }
}
