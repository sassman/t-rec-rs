use crate::ImageOnHeap;
use anyhow::{ensure, Context, Result};
use core_graphics::display::*;
use core_graphics::image::CGImageRef;
use image::flat::SampleLayout;
use image::{ColorType, FlatSamples};

pub fn capture_window_screenshot(win_id: u64) -> Result<ImageOnHeap> {
    let (w, h, channels, raw_data) = {
        let image = unsafe {
            CGDisplay::screenshot(
                CGRectNull,
                kCGWindowListOptionIncludingWindow | kCGWindowListExcludeDesktopElements,
                win_id as u32,
                kCGWindowImageNominalResolution
                    | kCGWindowImageBoundsIgnoreFraming
                    | kCGWindowImageShouldBeOpaque,
            )
        }
        .context(format!(
            "Cannot grab screenshot from CGDisplay of window id {}",
            win_id
        ))?;

        let img_ref: &CGImageRef = &image;
        // CAUTION: the width is not trust worthy, only the buffer dimensions are real
        let (_wrong_width, h) = (img_ref.width() as u32, img_ref.height() as u32);
        let raw_data: Vec<_> = img_ref.data().to_vec();
        let byte_per_row = img_ref.bytes_per_row() as u32;
        // the buffer must be as long as the row length x height
        ensure!(
            byte_per_row * h == raw_data.len() as u32,
            format!(
                "Cannot grab screenshot from CGDisplay of window id {}",
                win_id
            )
        );
        let byte_per_pixel = (img_ref.bits_per_pixel() / 8) as u8;
        // the actual width based on the buffer dimensions
        let w = byte_per_row / byte_per_pixel as u32;

        (w, h, byte_per_pixel, raw_data)
    };

    let color = ColorType::Bgra8;
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
    use crate::macos::setup;
    use image::save_buffer;

    #[test]
    #[should_panic(expected = "Cannot grab screenshot from CGDisplay of window id 999999")]
    fn should_throw_on_invalid_window_id() {
        capture_window_screenshot(9999999).unwrap();
    }

    #[test]
    #[cfg(feature = "e2e_tests")]
    fn should_capture_with_cropped_transparent_area() -> Result<()> {
        let mut api = setup()?;
        let win = 5308;
        let image = api.capture_window_screenshot(win)?;
        let (width, height) = (image.layout.width, image.layout.height);
        dbg!(width, height);

        // Note: visual validation is sometimes helpful:
        save_buffer(
            format!("frame-org-{}.tga", win),
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
            format!("frame-cropped-{}.tga", win),
            &image_cropped.samples,
            image_cropped.layout.width,
            image_cropped.layout.height,
            image_cropped.color_hint.unwrap(),
        )
        .context("Cannot save a frame.")?;

        Ok(())
    }
}
