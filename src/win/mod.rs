use crate::{ImageOnHeap, WindowList};

pub fn window_list() -> anyhow::Result<WindowList> {
    unimplemented!("there is only an impl for MacOS")
}

pub fn capture_window_screenshot(_win_id: u64) -> anyhow::Result<ImageOnHeap> {
    unimplemented!("there is only an impl for MacOS")
}

// references for winRT
// https://github.com/robmikh/wgc-rust-demo/blob/master/src/main.rs
