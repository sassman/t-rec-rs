use std::process::Command;

use anyhow::{Context, Result};
use tempfile::TempDir;

const PROGRAM: &str = "ffmpeg";

/// checks if ffmpeg is available
pub fn check_for_ffmpeg() -> Result<()> {
    let out = Command::new(PROGRAM)
        .arg("-version")
        .output()
        .with_context(|| {
            format!(
                "There is an issue with '{}', please install: `brew install {}`",
                PROGRAM, PROGRAM
            )
        })?;

    if !String::from_utf8(out.stdout.to_vec())
        .with_context(|| format!("Unable to parse the `{} -version`", PROGRAM))
        .unwrap()
        .contains("--enable-libx264")
    {
        anyhow::bail!("ffmpeg does not support codec 'libx264', please reinstall with the option '--enable-libx264'")
    }

    Ok(())
}

/// a nice resource that illustrates the power of ffmpeg
/// https://hamelot.io/visualization/using-ffmpeg-to-convert-a-set-of-images-into-a-video/
///
/// generating the final mp4 with help of ffmpeg
pub fn generate_mp4_with_ffmpeg(
    _time_codes: &[u128],
    tempdir: &TempDir,
    target: &str,
) -> Result<()> {
    println!("🎉 🎬 Generating {}", &target);
    Command::new(PROGRAM)
        .arg("-y")
        .arg("-r")
        // framerate
        .arg("4")
        .arg("-f")
        .arg("image2")
        .arg("-pattern_type")
        .arg("glob")
        .arg("-i")
        .arg(tempdir.path().join("*.tga"))
        .arg("-vcodec")
        .arg("libx264")
        .arg("-pix_fmt")
        .arg("yuv420p")
        // fixes: [libx264 @ 0x7fc216019000] height not divisible by 2 (650x477)
        .arg("-vf")
        .arg("pad='width=ceil(iw/2)*2:height=ceil(ih/2)*2'")
        // end of fix
        .arg(target)
        .output()
        .with_context(|| format!("Cannot start '{}' to generate the final video", PROGRAM))
        .map(|_| ())
}
