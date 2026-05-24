mod screenshot;
mod window_id;

use super::common::identify_transparency::identify_transparency;
use super::common::image::crop;
use super::common::{Platform, PlatformApiFactory};
use super::PlatformApi;
use super::{ImageOnHeap, Margin, Result, WindowList};

use screenshot::capture_window_screenshot;
use window_id::{get_foreground_window, window_list};

/// Default shell for Windows (CLI only).
#[cfg(feature = "cli")]
pub const DEFAULT_SHELL: &str = "cmd.exe";

impl PlatformApiFactory for Platform {
    fn setup() -> Result<Box<dyn PlatformApi>> {
        Ok(Box::new(WindowsApi { margin: None }))
    }
}

struct WindowsApi {
    margin: Option<Margin>,
}

impl PlatformApi for WindowsApi {
    fn calibrate(&mut self, window_id: u64) -> Result<()> {
        let image = capture_window_screenshot(window_id)?;
        self.margin = identify_transparency(*image)?;

        Ok(())
    }

    fn window_list(&self) -> Result<WindowList> {
        window_list()
    }

    fn capture_window_screenshot(&self, window_id: u64) -> Result<ImageOnHeap> {
        let img = capture_window_screenshot(window_id)?;
        if let Some(margin) = self.margin.as_ref() {
            if !margin.is_zero() {
                return crop(*img, margin);
            }
        }
        Ok(img)
    }

    fn get_active_window(&self) -> Result<u64> {
        get_foreground_window()
    }
}
