use crate::{Image, Margin, Result};
use image::flat::View;
use image::{Bgra, GenericImageView};

///
/// this helps to identify outer transparent regions
/// since some backends provides transparency either from a compositor effect like drop shadow on ubuntu / GNOME
/// or some strange right side strip on MacOS
pub fn identify_transparency(image: Image) -> Result<Option<Margin>> {
    let image: View<_, Bgra<u8>> = image.as_view()?;
    let (width, height) = image.dimensions();
    let half_width = width / 2;
    let half_height = height / 2;
    // > 3/4 transparency is good enough to declare the end of transparent regions
    let transparency_end: u8 = 0xff - (0xff / 4);

    let mut margin = Margin::zero();
    // identify top margin
    for y in 0..half_height {
        let Bgra([_, _, _, a]) = image.get_pixel(half_width, y);
        if a > transparency_end {
            // the end of the transparent area
            margin.top = y as u16;
            dbg!(margin.top);
            break;
        }
    }
    // identify bottom margin
    for y in (half_height..height).rev() {
        let Bgra([_, _, _, a]) = image.get_pixel(half_width, y);
        if a > transparency_end {
            // the end of the transparent area
            margin.bottom = (height - y - 1) as u16;
            dbg!(margin.bottom);
            break;
        }
    }
    // identify left margin
    for x in 0..half_width {
        let Bgra([_, _, _, a]) = image.get_pixel(x, half_height);
        if a > transparency_end {
            // the end of the transparent area
            margin.left = x as u16;
            dbg!(margin.left);
            break;
        }
    }
    // identify right margin
    for x in (half_width..width).rev() {
        let Bgra([_, _, _, a]) = image.get_pixel(x, half_height);
        if a > transparency_end {
            // the end of the transparent area
            margin.right = (width - x - 1) as u16;
            dbg!(margin.right);
            break;
        }
    }

    Ok(Some(margin))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_identify_macos_right_side_issue() -> Result<()> {
        // given an screen capture with transparency on the right side
        let image_org = image::open("tests/frames/frame-macos-right-side-issue.tga")?;
        let image = image_org.into_bgra8();
        let (width, height) = image.dimensions();
        let Bgra([blue, green, red, alpha]) = image.get_pixel(width - 1, height / 2);
        assert_eq!(alpha, &0, "the test image was not transparent");
        assert_eq!(red, &0, "the test image is not as expected");
        assert_eq!(green, &0, "the test image is not as expected");
        assert_eq!(blue, &0, "the test image is not as expected");

        // when
        let image_raw = image.into_flat_samples();
        let margin = identify_transparency(image_raw)?;

        // then
        assert_eq!(margin, Some(Margin::new(0, 14, 0, 0)));

        Ok(())
    }
}
