use std::path::Path;
use std::process::Command;

use anyhow::Context;

use crate::Result;

/// Apply corner radius effect to a single file.
///
/// Apply a corner radius decor effect via a chain of convert commands
/// this makes sure big sur corner radius, that comes in with white color, does not mess up
///
/// ```sh
/// convert t-rec-frame-000000251.bmp \
///     \( +clone  -alpha extract \
///         -draw 'fill black polygon 0,0 0,15 15,0 fill white circle 15,15 15,0' \
///         \( +clone -flip \) -compose Multiply -composite \
///         \( +clone -flop \) -compose Multiply -composite \
///      \) -alpha off -compose CopyOpacity -composite \
///    t-rec-frame-000000251.bmp
/// ```
pub fn apply_corner_to_file(file: &Path) -> Result<()> {
    let radius = 13;
    let e = Command::new("convert")
        .arg(file.to_str().unwrap())
        .arg("(")
        .args(["+clone", "-alpha", "extract"])
        .args([
            "-draw",
            &format!(
                "fill black polygon 0,0 0,{r} {r},0 fill white circle {r},{r} {r},0",
                r = radius
            ),
        ])
        .args(["(", "+clone", "-flip", ")"])
        .args(["-compose", "Multiply", "-composite"])
        .args(["(", "+clone", "-flop", ")"])
        .args(["-compose", "Multiply", "-composite"])
        .arg(")")
        .args(["-alpha", "off", "-compose", "CopyOpacity", "-composite"])
        .arg(file.to_str().unwrap())
        .output()
        .context("Cannot apply corner decor effect")?;

    if !e.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&e.stderr))
    } else {
        Ok(())
    }
}
