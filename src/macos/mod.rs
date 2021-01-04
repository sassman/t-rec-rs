mod core_foundation_sys_patches;
mod screenshot;
mod window_id;

use crate::PlatformApi;
use crate::{ImageOnHeap, Margin, Result, WindowList};

use crate::common::identify_transparency::identify_transparency;
use crate::common::image::crop;
use anyhow::Context;
use screenshot::capture_window_screenshot;
use std::env;
use window_id::window_list;

pub const DEFAULT_SHELL: &str = "/bin/sh";

pub fn setup() -> Result<Box<dyn PlatformApi>> {
    Ok(Box::new(QuartzApi { margin: None }))
}

struct QuartzApi {
    margin: Option<Margin>,
}

impl PlatformApi for QuartzApi {
    fn calibrate(&mut self, window_id: u64) -> Result<()> {
        let image = capture_window_screenshot(window_id)?;
        self.margin = identify_transparency(*image)?;

        Ok(())
    }

    fn window_list(&self) -> Result<WindowList> {
        window_list()
    }

    fn capture_window_screenshot(&self, window_id: u64) -> Result<ImageOnHeap> {
        let img = capture_window_screenshot(window_id)?;
        if let Some(margin) = self.margin.as_ref() {
            if !margin.is_zero() {
                // in this case we want to crop away the transparent margins
                return crop(*img, margin);
            }
        }
        Ok(img)
    }

    fn get_active_window(&self) -> Result<u64> {
        env::var("WINDOWID")
            .context("Env variable 'WINDOWID' was not set.")
            .unwrap()
            .parse::<u64>()
            .context("Cannot parse env variable 'WINDOWID' as number")
    }
}

#[cfg(feature = "test_against_real_display")]
#[cfg(test)]
mod test {
    use super::*;
    use image::flat::View;
    use image::{save_buffer, Bgra, GenericImageView};

    ///
    /// for terminals with odd dimensions, like: 93x17
    #[test]
    fn calibrate() -> Result<()> {
        let mut api = setup()?;
        let win = api.get_active_window()?;
        let image_raw = api.capture_window_screenshot(win)?;
        let image: View<_, Bgra<u8>> = image_raw.as_view().unwrap();
        let (width, height) = image.dimensions();

        api.calibrate(win)?;
        let image_calibrated_raw = api.capture_window_screenshot(win)?;
        let image_calibrated: View<_, Bgra<u8>> = image_calibrated_raw.as_view().unwrap();
        let (width_new, height_new) = image_calibrated.dimensions();
        dbg!(width, width_new, height, height_new);

        let Bgra([_, _, _, alpha]) = image.get_pixel(width / 2, 0);
        dbg!(alpha);
        if alpha == 0 {
            // if that pixel was full transparent, for example on ubuntu / GNOME, caused by the drop shadow
            // then we expect the calibrated image to be smaller and cropped by this area
            assert!(height > height_new);
            assert!(width > width_new);
        } else {
            assert!(height >= height_new);
            assert!(width >= width_new);
        }

        let pixel = image.get_pixel(width - 1, height / 2);
        dbg!(pixel);

        // Note: visual validation is sometimes helpful:
        save_buffer(
            format!("frame-raw-{}.tga", win),
            &image_raw.samples,
            image_raw.layout.width,
            image_raw.layout.height,
            image_raw.color_hint.unwrap(),
        )
        .context("Cannot save a frame.")?;
        //
        // save_buffer(
        //     format!("frame-calibrated-{}.tga", win),
        //     &image_calibrated_raw.samples,
        //     image_calibrated_raw.layout.width,
        //     image_calibrated_raw.layout.height,
        //     image_calibrated_raw.color_hint.unwrap(),
        // )
        // .context("Cannot save a frame.")?;

        Ok(())
    }
}
