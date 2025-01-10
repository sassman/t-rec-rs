use crate::common::Margin;
use crate::{Image, ImageOnHeap, Result};
use image::flat::View;
use image::{imageops, GenericImageView, ImageBuffer, Rgba};

/// specialized version of crop for [`ImageOnHeap`] and [`Margin`]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn crop(image: Image, margin: &Margin) -> Result<ImageOnHeap> {
    let mut img2: View<_, Rgba<u8>> = image.as_view()?;
    let (width, height) = (
        img2.width() - (margin.left + margin.right) as u32,
        img2.height() - (margin.top + margin.bottom) as u32,
    );
    let image_cropped = imageops::crop(
        &mut img2,
        margin.left as u32,
        margin.top as u32,
        width,
        height,
    );
    let mut buf = ImageBuffer::new(image_cropped.width(), image_cropped.height());

    for y in 0..height {
        for x in 0..width {
            buf.put_pixel(x, y, image_cropped.get_pixel(x, y));
        }
    }

    Ok(Box::new(buf.into_flat_samples()))
}

pub fn convert_bgra_to_rgba(buffer: &mut [u8]) {
    for chunk in buffer.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::open;

    #[test]
    fn should_crop() -> Result<()> {
        // given
        let image_org = open("tests/frames/frame-macos-right-side-issue.tga")?;
        let image = image_org.into_rgba8();
        let image_raw = ImageOnHeap::new(image.into_flat_samples());
        let (width, height) = (image_raw.layout.width, image_raw.layout.height);

        // when
        let cropped = crop(*image_raw, &Margin::new(1, 1, 1, 1))?;

        // then
        assert_eq!(cropped.layout.width, width - 2);
        assert_eq!(cropped.layout.height, height - 2);

        Ok(())
    }
}
