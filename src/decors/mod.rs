mod big_sur_corner;
mod shadow;

pub use big_sur_corner::{apply_big_sur_corner_effect, apply_corner_to_file};
pub use shadow::{apply_shadow_effect, apply_shadow_to_file};

use std::path::PathBuf;

use rayon::prelude::*;
use tempfile::TempDir;

use crate::utils::IMG_EXT;
use crate::Result;

///
/// apply a given effect (closure) to all frames
///
fn apply_effect(
    time_codes: &[u128],
    tempdir: &TempDir,
    effect: Box<dyn Fn(PathBuf) -> Result<()> + Send + Sync>,
) {
    time_codes.into_par_iter().for_each(|tc| {
        let file = tempdir
            .path()
            .join(crate::utils::file_name_for(tc, IMG_EXT));
        if let Err(e) = effect(file) {
            eprintln!("{}", e);
        }
    });
}
