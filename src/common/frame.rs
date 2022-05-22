use std::time::Instant;

use image::flat::SampleLayout;
use image::{ColorType, FlatSamples};

use crate::common::image::{convert_bgra_to_rgba, crop, CropMut};
use crate::{Image, Margin, Result};

pub struct Frame {
    image: Image,
    timecode: Instant,
}

impl Frame {
    pub fn from_bgra(raw_data: Vec<u8>, channels: u8, width: u32, height: u32) -> Self {
        let timecode = Instant::now();
        let mut raw_data = raw_data;
        convert_bgra_to_rgba(&mut raw_data[..]);

        let color = ColorType::Rgba8;
        let image = FlatSamples {
            samples: raw_data,
            layout: SampleLayout::row_major_packed(channels, width, height),
            color_hint: Some(color),
        };

        Self { image, timecode }
    }
}

impl AsRef<Image> for Frame {
    fn as_ref(&self) -> &Image {
        &self.image
    }
}

impl CropMut for Frame {
    fn crop(&mut self, margin: &Margin) -> Result<()> {
        self.image = crop(&self, margin)?;

        Ok(())
    }
}

impl AsRef<Instant> for Frame {
    fn as_ref(&self) -> &Instant {
        &self.timecode
    }
}

impl From<FlatSamples<Vec<u8>>> for Frame {
    fn from(image: FlatSamples<Vec<u8>>) -> Self {
        Self {
            image,
            timecode: Instant::now(),
        }
    }
}
