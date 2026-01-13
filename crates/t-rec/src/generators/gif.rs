use crate::utils::{file_name_for, IMG_EXT};

use anyhow::{Context, Result};
use std::ops::Div;
use std::process::{Command, Output};
use std::time::Duration;
use tempfile::TempDir;

// On Windows, ImageMagick 7.x uses 'magick' instead of 'convert'
// because 'convert.exe' conflicts with Windows filesystem conversion utility
#[cfg(target_os = "windows")]
const PROGRAM: &str = "magick";
#[cfg(not(target_os = "windows"))]
const PROGRAM: &str = "convert";

#[cfg(target_os = "macos")]
const INST_CMD: &str = "brew install imagemagick";
#[cfg(target_os = "windows")]
const INST_CMD: &str = "winget install ImageMagick.ImageMagick";
#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const INST_CMD: &str = "apt-get install imagemagick";

///
/// checks for imagemagick
/// and suggests the installation command if there are issues
/// On Windows, also verifies the output is actually from ImageMagick
/// (not Windows system convert.exe)
pub fn check_for_imagemagick() -> Result<Output> {
    let output = Command::new(PROGRAM)
        .arg("-version")
        .output()
        .with_context(|| {
            format!("There is an issue with '{PROGRAM}', please install: `{INST_CMD}`")
        })?;

    // Verify it's actually ImageMagick by checking the version output
    let version_str = String::from_utf8_lossy(&output.stdout);
    if !version_str.contains("ImageMagick") {
        anyhow::bail!(
            "'{PROGRAM}' does not appear to be ImageMagick. \
             Please install ImageMagick: `{INST_CMD}`"
        );
    }

    Ok(output)
}

///
/// generating the final gif with help of convert
pub fn generate_gif_with_convert(
    time_codes: &[u128],
    tempdir: &TempDir,
    target: &str,
    start_pause: Option<Duration>,
    end_pause: Option<Duration>,
) -> Result<()> {
    println!("🎉 🚀 Generating {target}\n");
    let mut cmd = Command::new(PROGRAM);
    cmd.arg("-loop").arg("0");
    let mut delay = 0;
    let temp = tempdir.path();
    let last_frame_i = time_codes.len() - 1;
    for (i, tc) in time_codes.iter().enumerate() {
        delay = *tc - delay;
        let frame = temp.join(file_name_for(tc, IMG_EXT));
        if !frame.exists() {
            continue;
        }
        let mut frame_delay = ((delay as f64 * 0.1).round() as u64).max(1);
        match (i, start_pause, end_pause) {
            (0, Some(delay), _) => {
                frame_delay += delay.as_millis().div(10) as u64;
            }
            (i, _, Some(delay)) if i == last_frame_i => {
                frame_delay += delay.as_millis().div(10) as u64;
            }
            (_, _, _) => {}
        }
        cmd.arg("-delay").arg(frame_delay.to_string()).arg(frame);
        delay = *tc;
    }
    cmd.arg("-layers")
        .arg("Optimize")
        .arg(target)
        .output()
        .context("Cannot start 'convert' to generate the final gif")?;

    Ok(())
}
