use tempfile::TempDir;

pub fn get_window_id_for(_terminal: String) -> Option<u32> {
    unimplemented!("there is only an impl for MacOS")
}

pub fn ls_win() {
    unimplemented!("there is only an impl for MacOS")
}

pub fn screenshot_and_save(
    _win_id: u32,
    _time_code: u128,
    _tempdir: &TempDir,
    _file_name_for: fn(&u128, &str) -> String,
) -> anyhow::Result<()> {
    unimplemented!("there is only an impl for MacOS")
}

// references for winRT
// https://github.com/robmikh/wgc-rust-demo/blob/master/src/main.rs
