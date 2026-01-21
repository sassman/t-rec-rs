use std::io::Cursor;
use std::sync::OnceLock;

use image::{DynamicImage, ImageReader};

use super::super::assets::VENTURA_WALLPAPER;

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
