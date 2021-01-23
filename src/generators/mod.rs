mod gif;
mod mp4;

pub use self::gif::check_for_imagemagick as check_for_gif;
pub use self::gif::generate_gif_with_convert as generate_gif;
pub use self::mp4::check_for_ffmpeg as check_for_mp4;
pub use self::mp4::generate_mp4_with_ffmpeg as generate_mp4;
