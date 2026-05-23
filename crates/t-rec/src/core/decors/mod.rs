mod big_sur_corner;
mod shadow;

pub use big_sur_corner::apply_corner_to_file;
pub use shadow::apply_shadow_to_file;

/// ImageMagick CLI binary name. Windows ships `magick`; the legacy `convert`
/// binary name collides with a built-in Windows utility.
#[cfg(target_os = "windows")]
pub(crate) const IMAGEMAGICK_CMD: &str = "magick";
#[cfg(not(target_os = "windows"))]
pub(crate) const IMAGEMAGICK_CMD: &str = "convert";
