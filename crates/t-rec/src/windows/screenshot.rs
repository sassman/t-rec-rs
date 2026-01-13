use crate::ImageOnHeap;

use anyhow::{Context, Result};
use image::flat::SampleLayout;
use image::{ColorType, FlatSamples};
use win_screenshot::capture::capture_window;

/// Captures a screenshot of the window with the given HWND.
///
/// The win-screenshot crate returns RGBA pixel data, which we convert
/// to our ImageOnHeap format for further processing.
pub fn capture_window_screenshot(win_id: u64) -> Result<ImageOnHeap> {
    let hwnd = win_id as isize;

    let buf = capture_window(hwnd).with_context(|| {
        format!(
            "Cannot grab screenshot of window id {}. \
             Make sure the window is not minimized.",
            win_id
        )
    })?;

    let (w, h) = (buf.width, buf.height);
    let channels = 4_u8; // RGBA

    let color = ColorType::Rgba8;
    let buffer = FlatSamples {
        samples: buf.pixels,
        layout: SampleLayout::row_major_packed(channels, w, h),
        color_hint: Some(color),
    };

    Ok(ImageOnHeap::new(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Cannot grab screenshot of window id 999999")]
    fn should_throw_on_invalid_window_id() {
        capture_window_screenshot(9999999).unwrap();
    }

    #[test]
    #[cfg(feature = "e2e_tests")]
    fn should_capture_with_cropped_transparent_area() -> Result<()> {
        use crate::common::{Platform, PlatformApi, PlatformApiFactory};
        use crate::utils::IMG_EXT;
        use image::save_buffer;

        let mut api = Platform::setup()?;
        let win = api.get_active_window()?;
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

        // On Windows, we may or may not have transparent margins depending on
        // window style and compositor settings
        assert!(width >= image_cropped.layout.width);

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
