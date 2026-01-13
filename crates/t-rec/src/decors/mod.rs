mod big_sur_corner;
mod shadow;

pub use big_sur_corner::apply_corner_to_file;
pub use shadow::apply_shadow_to_file;

// On Windows, ImageMagick 7.x uses 'magick' instead of 'convert'
// because 'convert.exe' conflicts with Windows filesystem conversion utility
#[cfg(target_os = "windows")]
pub(crate) const IMAGEMAGICK_CMD: &str = "magick";
#[cfg(not(target_os = "windows"))]
pub(crate) const IMAGEMAGICK_CMD: &str = "convert";
