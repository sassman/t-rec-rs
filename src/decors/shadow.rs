use std::process::Command;

use anyhow::Context;
use tempfile::TempDir;

use super::apply_effect;

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
pub fn apply_shadow_effect(time_codes: &[u128], tempdir: &TempDir, bg_color: String) {
    apply_effect(
        time_codes,
        tempdir,
        Box::new(move |file| {
            let e = Command::new("convert")
                .arg(file.to_str().unwrap())
                .arg("(")
                .args(["+clone", "-background", "black", "-shadow", "100x20+0+0"])
                .arg(")")
                .args(["+swap", "-background", bg_color.as_str()])
                .args(["-layers", "merge"])
                .arg(file.to_str().unwrap())
                .output()
                .context("Cannot apply shadow decor effect")?;

            if !e.status.success() {
                anyhow::bail!("{}", String::from_utf8_lossy(&e.stderr))
            } else {
                Ok(())
            }
        }),
    )
}
