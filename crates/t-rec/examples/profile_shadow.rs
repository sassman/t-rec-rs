//! Profiling helper for shadow effect
//! Run with: cargo flamegraph --example profile_shadow -p t-rec --features x-native-imgops

use image::{ColorType, Rgba, RgbaImage};
use tempfile::TempDir;

fn main() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().join("test.bmp");

    // Use 400x400 as representative size
    let img = RgbaImage::from_pixel(400, 400, Rgba([255, 0, 0, 255]));

    // Run multiple iterations for better sampling
    for i in 0..50 {
        image::save_buffer(&path, img.as_raw(), 400, 400, ColorType::Rgba8).unwrap();
        t_rec::core::decors::apply_shadow_to_file_native(&path, "white").unwrap();
        if i % 10 == 0 {
            eprintln!("Iteration {}/50", i);
        }
    }

    eprintln!("Done!");
}
