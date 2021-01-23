use crate::file_name_for;
use anyhow::{Context, Result};
use std::process::{Command, Output};
use tempfile::TempDir;

const PROGRAM: &str = "convert";

///
/// checks for imagemagick
/// and suggests the installation command if there are issues
pub fn check_for_imagemagick() -> Result<Output> {
    Command::new(PROGRAM)
        .arg("--version")
        .output()
        .with_context(|| {
            format!(
                "There is an issue with '{}', please install: `brew install imagemagick`",
                PROGRAM
            )
        })
}

///
/// generating the final gif with help of convert
pub fn generate_gif_with_convert(
    time_codes: &[u128],
    tempdir: &TempDir,
    target: &str,
) -> Result<()> {
    println!("🎉 🚀 Generating {}", target);
    let mut cmd = Command::new(PROGRAM);
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
