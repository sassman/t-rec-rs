mod core_foundation_sys_patches;
mod screenshot;
mod window_id;

use crate::PlatformApi;
use crate::{ImageOnHeap, Result, WindowList};

use screenshot::capture_window_screenshot;
use window_id::window_list;

pub const DEFAULT_SHELL: &str = "/bin/sh";

pub fn setup() -> Result<Box<dyn PlatformApi>> {
    Ok(Box::new(QuartzApi))
}

struct QuartzApi;

impl PlatformApi for QuartzApi {
    fn calibrate(&mut self, window_id: u64) -> Result<()> {
        capture_window_screenshot(window_id).map(|_| ())
    }

    fn window_list(&self) -> Result<WindowList> {
        window_list()
    }

    fn capture_window_screenshot(&self, window_id: u64) -> Result<ImageOnHeap> {
        capture_window_screenshot(window_id)
    }

    fn get_active_window(&self) -> Result<u64> {
        unimplemented!("MacOS has no support for get_active_window yet.")
    }
}
