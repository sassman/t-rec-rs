use crate::ImageOnHeap;
use anyhow::{Context, Result};
use core_graphics::display::*;
use core_graphics::image::CGImageRef;
use image::flat::SampleLayout;
use image::{ColorType, FlatSamples};

pub fn capture_window_screenshot(win_id: u32) -> Result<ImageOnHeap> {
    let (w, h, channels, raw_data) = {
        let image = unsafe {
            CGDisplay::screenshot(
                CGRectNull,
                kCGWindowListOptionIncludingWindow | kCGWindowListExcludeDesktopElements,
                win_id,
                kCGWindowImageNominalResolution
                    | kCGWindowImageBoundsIgnoreFraming
                    | kCGWindowImageShouldBeOpaque,
            )
        }
        .context("Cannot gather screenshot")?;

        let img_ref: &CGImageRef = &image;
        let (_wrong_width, h) = (img_ref.width() as u32, img_ref.height() as u32);
        let raw_data: Vec<_> = img_ref.data().to_vec();
        let byte_per_row = img_ref.bytes_per_row() as u32;
        let bit_per_pixel = img_ref.bits_per_pixel() as u32;
        let channels = img_ref.bits_per_component() as u32 / 8;
        // the buffer must be as long as the row length x height
        assert_eq!(byte_per_row * h, raw_data.len() as u32);
        // CAUTION this took me hours of my life time to figure out,
        // the width is not trust worthy, only the buffer dimensions are real
        // actual width, based on the buffer dimensions
        let w = byte_per_row / ((bit_per_pixel / 8) * channels);
        assert_eq!(bit_per_pixel / 8 * w * channels, byte_per_row);

        (w, h, channels as u8, raw_data)
    };

    let color = ColorType::Bgra8;
    let buffer = FlatSamples {
        samples: raw_data,
        layout: SampleLayout::row_major_packed(channels, w, h),
        color_hint: Some(color),
    };

    Ok(ImageOnHeap::new(buffer))
}
