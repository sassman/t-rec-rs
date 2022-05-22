use crate::{Image, Margin, Result};
use image::flat::View;
use image::{GenericImageView, Rgba};

///
/// this helps to identify outer transparent regions
/// since some backends provides transparency either from a compositor effect like drop shadow on ubuntu / GNOME
/// or some strange right side strip on MacOS
pub fn identify_transparency(image: impl AsRef<Image>) -> Result<Option<Margin>> {
    let image: View<_, Rgba<u8>> = image.as_ref().as_view()?;
    let (width, height) = image.dimensions();
    let half_width = width / 2;
    let half_height = height / 2;
    // > 3/4 transparency is good enough to declare the end of transparent regions
    let transparency_end: u8 = 0xff - (0xff / 4);

    let mut margin = Margin::zero();
    // identify top margin
    for y in 0..half_height {
        let Rgba([_, _, _, a]) = image.get_pixel(half_width, y);
        if a > transparency_end {
            // the end of the transparent area
            margin.top = y as u16;
            break;
        }
    }
    // identify bottom margin
    for y in (half_height..height).rev() {
        let Rgba([_, _, _, a]) = image.get_pixel(half_width, y);
        if a > transparency_end {
            // the end of the transparent area
            margin.bottom = (height - y - 1) as u16;
            break;
        }
    }
    // identify left margin
    for x in 0..half_width {
        let Rgba([_, _, _, a]) = image.get_pixel(x, half_height);
        if a > transparency_end {
            // the end of the transparent area
            margin.left = x as u16;
            break;
        }
    }
    // identify right margin
    for x in (half_width..width).rev() {
        let Rgba([_, _, _, a]) = image.get_pixel(x, half_height);
        if a > transparency_end {
            // the end of the transparent area
            margin.right = (width - x - 1) as u16;
            break;
        }
    }

    Ok(Some(margin))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::common::Frame;

    #[test]
    fn should_identify_macos_right_side_issue() -> Result<()> {
        // given an screen capture with transparency on the right side
        let image_org = image::open("tests/frames/frame-macos-right-side-issue.tga")?;
        let image = image_org.into_rgba8();
        let (width, height) = image.dimensions();
        let Rgba([blue, green, red, alpha]) = image.get_pixel(width - 1, height / 2);
        assert_eq!(alpha, &0, "the test image was not transparent");
        assert_eq!(red, &0, "the test image is not as expected");
        assert_eq!(green, &0, "the test image is not as expected");
        assert_eq!(blue, &0, "the test image is not as expected");

        // when
        let image_raw: Frame = image.into_flat_samples().into();
        let margin = identify_transparency(&image_raw)?;

        // then
        assert_eq!(margin, Some(Margin::new(0, 14, 0, 0)));

        Ok(())
    }
}
