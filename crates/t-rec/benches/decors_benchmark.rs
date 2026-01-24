use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use image::{ColorType, Rgba, RgbaImage};
use std::path::Path;
use tempfile::TempDir;

/// Create a test image of given size and save it to the specified path
fn create_test_image(path: &Path, width: u32, height: u32) {
    let img = RgbaImage::from_pixel(width, height, Rgba([255, 0, 0, 255]));
    image::save_buffer(path, img.as_raw(), width, height, ColorType::Rgba8).unwrap();
}

fn bench_corner_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("corner_effects");

    // Test different image sizes
    for size in [100, 400, 800].iter() {
        let temp_dir = TempDir::new().unwrap();

        // Benchmark native implementation
        group.bench_with_input(BenchmarkId::new("native", size), size, |b, &size| {
            let path = temp_dir.path().join(format!("native_{}.bmp", size));
            b.iter(|| {
                create_test_image(&path, size, size);
                t_rec::core::decors::apply_corner_to_file_native(black_box(&path)).unwrap();
            });
        });

        // Benchmark ImageMagick implementation
        group.bench_with_input(BenchmarkId::new("imagemagick", size), size, |b, &size| {
            let path = temp_dir.path().join(format!("magick_{}.bmp", size));
            b.iter(|| {
                create_test_image(&path, size, size);
                t_rec::core::decors::apply_corner_to_file_imagemagick(black_box(&path)).unwrap();
            });
        });
    }

    group.finish();
}

fn bench_shadow_effects(c: &mut Criterion) {
    let mut group = c.benchmark_group("shadow_effects");

    // Test different image sizes
    for size in [100, 400, 800].iter() {
        let temp_dir = TempDir::new().unwrap();

        // Benchmark native implementation
        group.bench_with_input(BenchmarkId::new("native", size), size, |b, &size| {
            let path = temp_dir.path().join(format!("native_{}.bmp", size));
            b.iter(|| {
                create_test_image(&path, size, size);
                t_rec::core::decors::apply_shadow_to_file_native(black_box(&path), "white")
                    .unwrap();
            });
        });

        // Benchmark ImageMagick implementation
        group.bench_with_input(BenchmarkId::new("imagemagick", size), size, |b, &size| {
            let path = temp_dir.path().join(format!("magick_{}.bmp", size));
            b.iter(|| {
                create_test_image(&path, size, size);
                t_rec::core::decors::apply_shadow_to_file_imagemagick(black_box(&path), "white")
                    .unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_corner_effects, bench_shadow_effects);
criterion_main!(benches);
