use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::process::{Command, ExitStatus};

pub const DEFAULT_EXT: &str = "gif";
pub const MOVIE_EXT: &str = "mp4";
pub const IMG_EXT: &str = "bmp";

/// encapsulate the file naming convention
pub fn file_name_for<T: AsRef<u32>>(tc: &T, ext: &str) -> String {
    format!("t-rec-frame-{:09}.{}", tc.as_ref(), ext)
}

/// starts the main program and keeps interacting with the user
/// blocks until termination
pub fn sub_shell_thread<T: AsRef<OsStr> + Clone>(program: T) -> Result<ExitStatus> {
    Command::new(program.clone())
        .spawn()
        .context(format!("failed to start {:?}", program.as_ref()))?
        .wait()
        .context("Something went wrong waiting for the sub shell.")
}

/// returns a new filename that does not yet exists.
/// Note: returns without extension, but checks with extension
/// like `t-rec` or `t-rec_1`
pub fn target_file(basename: impl AsRef<str>) -> String {
    let basename = basename.as_ref();
    let mut suffix = "".to_string();
    let mut i = 0;
    while std::path::Path::new(format!("{basename}{suffix}.{DEFAULT_EXT}").as_str()).exists()
        || std::path::Path::new(format!("{basename}{suffix}.{MOVIE_EXT}").as_str()).exists()
    {
        i += 1;
        suffix = format!("_{}", i).to_string();
    }

    format!("{basename}{suffix}")
}
