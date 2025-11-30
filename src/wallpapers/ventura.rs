use std::io::Cursor;
use std::sync::OnceLock;

use image::{DynamicImage, ImageReader};
use tempfile::TempDir;

use crate::assets::VENTURA_WALLPAPER;

use super::apply_wallpaper_effect;

/// Lazily loaded and cached Ventura wallpaper image
static WALLPAPER: OnceLock<DynamicImage> = OnceLock::new();

/// Load the embedded Ventura wallpaper (cached after first load)
pub fn get_ventura_wallpaper() -> &'static DynamicImage {
    WALLPAPER.get_or_init(|| {
        ImageReader::new(Cursor::new(VENTURA_WALLPAPER))
            .with_guessed_format()
            .expect("Failed to detect wallpaper format")
            .decode()
            .expect("Failed to decode embedded Ventura wallpaper")
    })
}

///
/// Apply the Ventura wallpaper background effect to all frames.
///
/// Each frame is composited onto a centered crop of the Ventura wallpaper
/// with the specified padding on all sides.
///
pub fn apply_ventura_wallpaper_effect(time_codes: &[u128], tempdir: &TempDir, padding: u32) {
    let wallpaper = get_ventura_wallpaper();
    apply_wallpaper_effect(time_codes, tempdir, wallpaper, padding);
}
