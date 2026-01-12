use std::process::Command;

use anyhow::{Context, Result};
use tempfile::TempDir;

use crate::utils::IMG_EXT;

const FFMPEG_BINARY: &str = "ffmpeg";

#[cfg(target_os = "macos")]
const INST_CMD: &str = "brew install ffmpeg";
#[cfg(target_os = "windows")]
const INST_CMD: &str = "winget install ffmpeg";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const INST_CMD: &str = "apt-get install ffmpeg";

/// checks if ffmpeg is available
pub fn check_for_ffmpeg() -> Result<()> {
    let out = Command::new(FFMPEG_BINARY)
        .arg("-version")
        .output()
        .with_context(|| {
            format!("There is an issue with '{FFMPEG_BINARY}', please install: `{INST_CMD}`")
        })?;

    if !String::from_utf8(out.stdout.to_vec())
        .with_context(|| format!("Unable to parse the `{FFMPEG_BINARY} -version`"))?
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
    fps: u8,
) -> Result<()> {
    println!("🎬 🎉 🚀 Generating {target}");

    // Build the glob pattern - need to use forward slashes for ffmpeg on Windows
    let temp_path = tempdir.path().to_string_lossy().replace('\\', "/");
    let pattern = format!("{temp_path}/*.{IMG_EXT}");

    let output = Command::new(FFMPEG_BINARY)
        .arg("-y")
        .arg("-r")
        // framerate
        .arg(fps.to_string())
        .arg("-f")
        .arg("image2")
        .arg("-pattern_type")
        .arg("glob")
        .arg("-i")
        .arg(&pattern)
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
        .with_context(|| format!("Cannot start '{FFMPEG_BINARY}' to generate the final video"))?;

    if !output.status.success() {
        anyhow::bail!(
            "ffmpeg failed with exit code {:?}\nStderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(())
}
