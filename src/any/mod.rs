use crate::ImageOnHeap;

pub fn get_window_id_for(_terminal: String) -> Option<u32> {
    unimplemented!("there is only an impl for MacOS")
}

pub fn ls_win() {
    unimplemented!("there is only an impl for MacOS")
}

pub fn capture_window_screenshot(_win_id: u32) -> anyhow::Result<ImageOnHeap> {
    unimplemented!("there is only an impl for MacOS")
}

// references for winRT
// https://github.com/robmikh/wgc-rust-demo/blob/master/src/main.rs
