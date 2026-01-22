pub const DEFAULT_EXT: &str = "gif";
pub const MOVIE_EXT: &str = "mp4";
pub const IMG_EXT: &str = "bmp";

/// encapsulate the file naming convention
pub fn file_name_for(tc: &u128, ext: &str) -> String {
    format!("t-rec-frame-{:09}.{}", tc, ext)
}
