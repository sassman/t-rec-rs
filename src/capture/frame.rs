use anyhow::Context;
pub use blockhash::Image;
use image::flat::View;
use image::ColorType::Rgba8;
use image::{save_buffer, FlatSamples, GenericImageView, Rgba};
use tempfile::TempDir;

use crate::capture::Timecode;
use crate::utils::IMG_EXT;
use crate::{ImageOnHeap, Result};

pub struct Frame(FlatSamples<Vec<u8>>);

impl Frame {
    /// saves a frame as a IMG_EXT file
    pub fn save(
        &self,
        tc: &Timecode,
        tempdir: &TempDir,
        file_name_for: fn(&Timecode, &str) -> String,
    ) -> Result<()> {
        let image = self.as_ref();
        save_buffer(
            tempdir.path().join(file_name_for(tc, IMG_EXT)),
            &image.samples,
            image.layout.width,
            image.layout.height,
            image.color_hint.unwrap_or(Rgba8),
        )
        .context("Cannot save frame")
    }
}

impl Image for Frame {
    fn dimensions(&self) -> (u32, u32) {
        (self.0.layout.width, self.0.layout.height)
    }

    fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let image: View<_, Rgba<u8>> = self.0.as_view().unwrap();
        image.get_pixel(x, y).0
    }
}

impl AsRef<FlatSamples<Vec<u8>>> for Frame {
    fn as_ref(&self) -> &FlatSamples<Vec<u8>> {
        &self.0
    }
}

impl From<ImageOnHeap> for Frame {
    fn from(img: ImageOnHeap) -> Self {
        Self(*img)
    }
}
