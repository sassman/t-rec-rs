use crate::core::ImageOnHeap;

use anyhow::{Context, Result};
use image::flat::SampleLayout;
use image::{ColorType, FlatSamples};
use std::convert::TryFrom;
use win_screenshot::capture::capture_window;

/// Captures a screenshot of the window with the given HWND.
///
/// The win-screenshot crate returns RGBA pixel data, which we convert
/// to our ImageOnHeap format for further processing.
pub fn capture_window_screenshot(win_id: u64) -> Result<ImageOnHeap> {
    let hwnd = isize::try_from(win_id).with_context(|| {
        format!("Window id {win_id} does not fit in the native pointer size on this platform.")
    })?;

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
    #[should_panic(expected = "Cannot grab screenshot of window id 9999999")]
    fn should_throw_on_invalid_window_id() {
        capture_window_screenshot(9999999).unwrap();
    }
}
