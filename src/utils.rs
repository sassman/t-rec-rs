use anyhow::{Context, Result};
use std::ffi::OsStr;
use std::process::{Command, ExitStatus};

/// encapsulate the file naming convention
pub fn file_name_for(tc: &u128, ext: &str) -> String {
    format!("t-rec-frame-{:09}.{}", tc, ext)
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
pub fn target_file() -> String {
    let mut suffix = "".to_string();
    let default_ext = "gif";
    let movie_ext = "mp4";
    let mut i = 0;
    while std::path::Path::new(format!("t-rec{}.{}", suffix, default_ext).as_str()).exists()
        || std::path::Path::new(format!("t-rec{}.{}", suffix, movie_ext).as_str()).exists()
    {
        i += 1;
        suffix = format!("_{}", i).to_string();
    }

    format!("t-rec{}", suffix)
}
