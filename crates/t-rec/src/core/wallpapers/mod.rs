// Submodules are pub(crate) to allow headless module to import internal items
pub(crate) mod types;
pub(crate) mod validation;
pub(crate) mod ventura;

// Public API
pub use types::Wallpaper;
pub use validation::resolve_wallpaper;
// Library-only exports (used by api/headless.rs)
#[cfg(not(feature = "cli"))]
pub use validation::load_and_validate_wallpaper;

use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

/// Composite a single frame onto the wallpaper background
pub fn composite_frame(
    frame_path: &std::path::Path,
    wallpaper: &DynamicImage,
    wallpaper_width: u32,
    wallpaper_height: u32,
    padding: u32,
) -> anyhow::Result<()> {
    // Load the frame
    let frame = image::open(frame_path)?;
    let (frame_width, frame_height) = frame.dimensions();

    // Calculate output dimensions (frame + padding on all sides)
    let output_width = frame_width + (padding * 2);
    let output_height = frame_height + (padding * 2);

    // Ensure the wallpaper is large enough
    if output_width > wallpaper_width || output_height > wallpaper_height {
        anyhow::bail!(
            "Frame size {}x{} with padding exceeds wallpaper size {}x{}",
            frame_width,
            frame_height,
            wallpaper_width,
            wallpaper_height
        );
    }

    // Calculate crop region to center the output on the wallpaper
    let crop_x = (wallpaper_width - output_width) / 2;
    let crop_y = (wallpaper_height - output_height) / 2;

    // Crop the wallpaper to output size (centered)
    let mut output = wallpaper
        .crop_imm(crop_x, crop_y, output_width, output_height)
        .to_rgba8();

    // Overlay the frame at the padding offset
    let frame_rgba = frame.to_rgba8();
    overlay_image(&mut output, &frame_rgba, padding, padding);

    // Save back to the same path (as BMP to match the expected format)
    output.save(frame_path)?;

    Ok(())
}

/// Overlay source image onto destination at the given offset
fn overlay_image(dest: &mut RgbaImage, src: &RgbaImage, offset_x: u32, offset_y: u32) {
    for (x, y, pixel) in src.enumerate_pixels() {
        let dest_x = x + offset_x;
        let dest_y = y + offset_y;

        if dest_x < dest.width() && dest_y < dest.height() {
            // Alpha blending
            let src_alpha = pixel[3] as f32 / 255.0;
            if src_alpha > 0.0 {
                let dest_pixel = dest.get_pixel_mut(dest_x, dest_y);
                if src_alpha >= 1.0 {
                    // Fully opaque - just copy
                    *dest_pixel = *pixel;
                } else {
                    // Alpha blend
                    let dest_alpha = dest_pixel[3] as f32 / 255.0;
                    let out_alpha = src_alpha + dest_alpha * (1.0 - src_alpha);

                    if out_alpha > 0.0 {
                        let blend = |s: u8, d: u8| -> u8 {
                            let s = s as f32;
                            let d = d as f32;
                            ((s * src_alpha + d * dest_alpha * (1.0 - src_alpha)) / out_alpha) as u8
                        };

                        *dest_pixel = Rgba([
                            blend(pixel[0], dest_pixel[0]),
                            blend(pixel[1], dest_pixel[1]),
                            blend(pixel[2], dest_pixel[2]),
                            (out_alpha * 255.0) as u8,
                        ]);
                    }
                }
            }
        }
    }
}
