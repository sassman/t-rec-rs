use std::path::Path;
use std::process::Command;

use anyhow::Context;

use crate::Result;
use super::IMAGEMAGICK_CMD;

/// apply a border decor effect via a chain of convert commands
///
/// ```sh
/// convert t-rec-frame-000000251.bmp \
///     \( +clone -background black -shadow 140x10+0+0 \) \
///     +swap -background white \
///     -layers merge \
///     t-rec-frame-000000251.bmp
/// ```
pub fn apply_shadow_to_file(file: &Path, bg_color: &str) -> Result<()> {
    let e = Command::new(IMAGEMAGICK_CMD)
        .arg(file.to_str().unwrap())
        .arg("(")
        .args(["+clone", "-background", "black", "-shadow", "100x20+0+0"])
        .arg(")")
        .args(["+swap", "-background", bg_color])
        .args(["-layers", "merge"])
        .arg(file.to_str().unwrap())
        .output()
        .context("Cannot apply shadow decor effect")?;

    if !e.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&e.stderr))
    } else {
        Ok(())
    }
}
