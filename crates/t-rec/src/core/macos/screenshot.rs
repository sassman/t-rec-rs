use super::cg_window_constants::{
    K_CG_WINDOW_IMAGE_BOUNDS_IGNORE_FRAMING, K_CG_WINDOW_IMAGE_NOMINAL_RESOLUTION,
    K_CG_WINDOW_IMAGE_SHOULD_BE_OPAQUE, K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS,
    K_CG_WINDOW_LIST_OPTION_INCLUDING_WINDOW,
};
use crate::core::common::image::convert_bgra_to_rgba;
use crate::ImageOnHeap;

use anyhow::{ensure, Context, Result};
use image::flat::SampleLayout;
use image::{ColorType, FlatSamples};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_core_graphics::{CGDataProvider, CGImage};

#[allow(deprecated)] // CGWindowListCreateImage is deprecated but we still need it
pub fn capture_window_screenshot(win_id: u64) -> Result<ImageOnHeap> {
    use objc2_core_graphics::CGWindowListCreateImage;
    let (w, h, channels, mut raw_data) = {
        let null_rect = CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(0.0, 0.0));

        let image = CGWindowListCreateImage(
            null_rect,
            K_CG_WINDOW_LIST_OPTION_INCLUDING_WINDOW | K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS,
            win_id as u32,
            K_CG_WINDOW_IMAGE_NOMINAL_RESOLUTION
                | K_CG_WINDOW_IMAGE_BOUNDS_IGNORE_FRAMING
                | K_CG_WINDOW_IMAGE_SHOULD_BE_OPAQUE,
        );

        let image = image.context(format!(
            "Cannot grab screenshot from CGWindowListCreateImage of window id {}",
            win_id
        ))?;

        // CAUTION: the width is not trust worthy, only the buffer dimensions are real
        let (_wrong_width, h) = (
            CGImage::width(Some(&image)) as u32,
            CGImage::height(Some(&image)) as u32,
        );

        let data_provider = CGImage::data_provider(Some(&image))
            .context("Cannot get data provider from CGImage")?;
        let cf_data = CGDataProvider::data(Some(&data_provider))
            .context("Cannot copy data from data provider")?;
        let raw_data: Vec<u8> = unsafe {
            std::slice::from_raw_parts(cf_data.byte_ptr(), cf_data.length() as usize).to_vec()
        };

        let byte_per_row = CGImage::bytes_per_row(Some(&image)) as u32;
        // the buffer must be as long as the row length x height
        ensure!(
            byte_per_row * h == raw_data.len() as u32,
            format!(
                "Cannot grab screenshot from CGWindowListCreateImage of window id {}",
                win_id
            )
        );
        let byte_per_pixel = (CGImage::bits_per_pixel(Some(&image)) / 8) as u8;
        // the actual width based on the buffer dimensions
        let w = byte_per_row / byte_per_pixel as u32;

        (w, h, byte_per_pixel, raw_data)
    };

    convert_bgra_to_rgba(&mut raw_data);

    let color = ColorType::Rgba8;
    let buffer = FlatSamples {
        samples: raw_data,
        layout: SampleLayout::row_major_packed(channels, w, h),
        color_hint: Some(color),
    };

    Ok(ImageOnHeap::new(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(
        expected = "Cannot grab screenshot from CGWindowListCreateImage of window id 999999"
    )]
    fn should_throw_on_invalid_window_id() {
        capture_window_screenshot(9999999).unwrap();
    }

    #[test]
    #[cfg(feature = "e2e_tests")]
    fn should_capture_with_cropped_transparent_area() -> Result<()> {
        use crate::core::common::{Platform, PlatformApi, PlatformApiFactory};
        use crate::core::utils::IMG_EXT;
        use image::save_buffer;

        let mut api = Platform::setup()?;
        let win = 23421;
        let image = api.capture_window_screenshot(win)?;
        let (width, height) = (image.layout.width, image.layout.height);
        dbg!(width, height);

        // Note: visual validation is sometimes helpful:
        save_buffer(
            format!("frame-org-{win}.{IMG_EXT}"),
            &image.samples,
            image.layout.width,
            image.layout.height,
            image.color_hint.unwrap(),
        )
        .context("Cannot save a frame.")?;

        api.calibrate(win)?;
        let image_cropped = api.capture_window_screenshot(win)?;

        assert!(width > image_cropped.layout.width);
        // Note: visual validation is sometimes helpful:
        save_buffer(
            format!("frame-cropped-{win}.{IMG_EXT}"),
            &image_cropped.samples,
            image_cropped.layout.width,
            image_cropped.layout.height,
            image_cropped.color_hint.unwrap(),
        )
        .context("Cannot save a frame.")?;

        Ok(())
    }
}
