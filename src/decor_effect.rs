use crate::{file_name_for, Result};

use anyhow::Context;
use std::process::Command;
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
///
pub fn apply_shadow_decor_effect(time_codes: &[u128], tempdir: &TempDir) -> Result<()> {
    let mut results = Vec::new();
    for tc in time_codes.iter() {
        let file = tempdir.path().join(file_name_for(tc, "tga"));
        results.push(
            Command::new("convert")
                .arg(file.to_str().unwrap())
                .arg("(")
                .args(&["+clone", "-background", "black", "-shadow", "140x10+0+0"])
                .arg(")")
                .args(&["+swap", "-background", "white"])
                .args(&["-layers", "merge"])
                .arg(file.to_str().unwrap())
                .spawn()
                .context("Cannot apply decor effect")?,
        );
    }

    for mut r in results {
        r.wait()?;
    }

    Ok(())
}
