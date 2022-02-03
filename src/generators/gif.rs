use crate::utils::file_name_for;

use anyhow::{Context, Result};
use std::ops::Div;
use std::process::{Command, Output};
use std::time::Duration;
use tempfile::TempDir;

const PROGRAM: &str = "convert";
#[cfg(target_os = "macos")]
const INST_CMD: &str = "brew install imagemagick";
#[cfg(not(target_os = "macos"))]
const INST_CMD: &str = "apt-get install imagemagick";

///
/// checks for imagemagick
/// and suggests the installation command if there are issues
pub fn check_for_imagemagick() -> Result<Output> {
    Command::new(PROGRAM)
        .arg("--version")
        .output()
        .with_context(|| {
            format!(
                "There is an issue with '{}', please install: `{}`",
                PROGRAM, INST_CMD,
            )
        })
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
    println!("🎉 🚀 Generating {}", target);
    let mut cmd = Command::new(PROGRAM);
    cmd.arg("-loop").arg("0");
    let mut delay = 0;
    let last_frame_i = time_codes.len() - 1;
    for (i, tc) in time_codes.iter().enumerate() {
        delay = *tc - delay;
        let frame = tempdir.path().join(file_name_for(tc, "tga"));
        let mut frame_delay = (delay as f64 * 0.1) as u64;
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
