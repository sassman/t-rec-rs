use crate::{file_name_for, Result};

use anyhow::Context;
use std::path::PathBuf;
use std::process::{Child, Command};
use tempfile::TempDir;

///
/// apply a border decor effect via a chain of convert commands
///
/// ```sh
/// convert t-rec-frame-000000251.tga \
///     \( +clone -background black -shadow 140x10+0+0 \) \
///     +swap -background white \
///     -layers merge \
///     t-rec-frame-000000251.tga
/// ```
pub fn apply_shadow_effect(time_codes: &[u128], tempdir: &TempDir) -> Result<()> {
    apply_effect(
        time_codes,
        tempdir,
        Box::new(move |file| {
            Command::new("convert")
                .arg(file.to_str().unwrap())
                .arg("(")
                .args(&["+clone", "-background", "black", "-shadow", "100x20+0+0"])
                .arg(")")
                .args(&["+swap", "-background", "white"])
                .args(&["-layers", "merge"])
                .arg(file.to_str().unwrap())
                .spawn()
                .context("Cannot apply shadow decor effect")
        }),
    )
}

///
/// apply a corner radius decor effect via a chain of convert commands
/// this makes sure big sur corner radius, that comes in with white color, does not mess up
///
/// ```sh
/// convert t-rec-frame-000000251.tga \
///     -trim \( +clone  -alpha extract \
///         -draw 'fill black polygon 0,0 0,15 15,0 fill white circle 15,15 15,0' \
///         \( +clone -flip \) -compose Multiply -composite \
///         \( +clone -flop \) -compose Multiply -composite \
///      \) -alpha off -compose CopyOpacity -composite \
///    t-rec-frame-000000251.tga
/// ```
pub fn apply_big_sur_corner_effect(time_codes: &[u128], tempdir: &TempDir) -> Result<()> {
    let radius = 13;
    apply_effect(
        time_codes,
        tempdir,
        Box::new(move |file| {
            Command::new("convert")
                .arg(file.to_str().unwrap())
                .arg("-trim")
                .arg("(")
                .args(&["+clone", "-alpha", "extract"])
                .args(&[
                    "-draw",
                    &format!(
                        "fill black polygon 0,0 0,{r} {r},0 fill white circle {r},{r} {r},0",
                        r = radius
                    ),
                ])
                .args(&["(", "+clone", "-flip", ")"])
                .args(&["-compose", "Multiply", "-composite"])
                .args(&["(", "+clone", "-flop", ")"])
                .args(&["-compose", "Multiply", "-composite"])
                .arg(")")
                .args(&["-alpha", "off", "-compose", "CopyOpacity", "-composite"])
                .arg(file.to_str().unwrap())
                .spawn()
                .context("Cannot apply corner decor effect")
        }),
    )
}

///
/// apply a given effect (closure) to all frames
///
fn apply_effect(
    time_codes: &[u128],
    tempdir: &TempDir,
    effect: Box<dyn Fn(PathBuf) -> Result<Child>>,
) -> Result<()> {
    let mut results = Vec::new();
    for tc in time_codes.iter() {
        let file = tempdir.path().join(file_name_for(tc, "tga"));
        results.push(effect(file)?);
    }

    for mut r in results {
        r.wait()?;
    }

    Ok(())
}
