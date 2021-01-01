use crate::{file_name_for, target_file};
use anyhow::{Context, Result};
use gif::{Encoder, Repeat};
use image::gif::GifEncoder;
use image::{open, Delay, Frame};
use std::fs::File;
use std::io::{Cursor, Write};
use std::process::Command;
use tempfile::TempDir;

///
/// generating the final gif with help of convert
pub fn generate_gif_with_convert(time_codes: &[u128], tempdir: &TempDir) -> Result<()> {
    let target = target_file();
    println!("🎉 🚀 Generating {}", target);
    let mut cmd = Command::new("magick");
    cmd.arg("convert");
    cmd.arg("-loop").arg("0");
    let mut delay = 0;
    for tc in time_codes.iter() {
        delay = *tc - delay;
        cmd.arg("-delay")
            .arg(format!("{}", (delay as f64 * 0.1) as u64))
            .arg(tempdir.path().join(file_name_for(tc, "tga")));
        delay = *tc;
    }
    cmd.arg("-layers")
        .arg("Optimize")
        .arg(target)
        .output()
        .context("Cannot start 'convert' to generate the final gif")?;

    Ok(())
}

/// TODO implement a image native gif creation
pub fn _generate_gif(time_codes: &[u128], tempdir: &TempDir) -> Result<()> {
    let target = target_file();
    println!(
        "\n🎉 🚀 Generating {:?} out of {} frames!",
        target,
        time_codes.len()
    );
    let target = std::path::Path::new(target.as_str());
    let mut target = File::create(target)?;
    let mut buf = Vec::new();

    {
        let mut curs = Cursor::new(&mut buf);
        let mut gif_encoder = GifEncoder::new(&mut curs);
        let mut delay = 0;
        for tc in time_codes.iter() {
            delay = *tc - delay;
            let frame = {
                let delay = delay as u32;
                let filename = file_name_for(tc, "tga");
                let frame = tempdir.path().join(&filename);
                let frame = open(frame).context(format!("Cannot load frame {}", &filename))?;
                Frame::from_parts(
                    frame.into_rgba8(),
                    0,
                    0,
                    Delay::from_numer_denom_ms(delay, 1),
                )
            };
            print!(".");
            gif_encoder.encode_frame(frame)?;

            delay = *tc;
        }
    }

    target.write_all(&buf)?;
    println!();

    Ok(())
}

pub fn _generate_gif_2(time_codes: &[u128], tempdir: &TempDir) -> Result<()> {
    let target = target_file();
    println!(
        "\n🎉 🚀 Generating {:?} out of {} frames!",
        target,
        time_codes.len()
    );
    let target = std::path::Path::new(target.as_str());
    let image = File::create(target)?;
    let mut encoder = None;
    // Encoder::new(&mut image, width, height, color_map).unwrap();

    let mut delay = 0;
    for tc in time_codes.iter() {
        delay = *tc - delay;
        let filename = file_name_for(tc, "tga");
        let frame = tempdir.path().join(&filename);
        let mut frame = open(frame)
            .context(format!("Cannot load frame {}", &filename))?
            .into_rgba8();
        let (h, w) = (frame.height(), frame.width());
        let pixel = frame.as_flat_samples_mut().samples;
        // let mut pixel = frame.into_rgba().as_flat_samples().samples.to_vec();
        let mut frame = gif::Frame::from_rgba(w as u16, h as u16, pixel);
        frame.delay = delay as u16;

        if encoder.is_none() {
            encoder =
                Some(Encoder::new(image.try_clone().unwrap(), w as u16, h as u16, &[]).unwrap());
            encoder
                .as_mut()
                .unwrap()
                .set_repeat(Repeat::Infinite)
                .unwrap();
        }
        print!(".");
        encoder.as_mut().unwrap().write_frame(&frame).unwrap();

        delay = *tc;
    }
    println!();

    Ok(())
}

// pub fn generate_gif_3(time_codes: &[u128], tempdir: &TempDir) -> Result<()> {
//     let target = target_file();
//     println!(
//         "\n🎉 🚀 Generating {:?} out of {} frames!",
//         target,
//         time_codes.len()
//     );
//     let target = std::path::Path::new(target.as_str());
//     let mut image = File::create(target)?;
//     let mut config = GIFConfig::new();
//
//     for tc in time_codes.iter() {
//         let frame = tempdir.path().join(&file_name_for(tc, "tga"));
//         let img = ImageResource::from_path(&frame);
//
//         // to_gif()
//     }
//
//     Ok(())
// }
