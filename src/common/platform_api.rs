use crate::{ImageOnHeap, Result, WindowId, WindowList};

pub trait PlatformApi: Send + Unpin + Sized {
    /// 1. it does check for the screenshot
    /// 2. it checks for transparent margins and configures the api
    ///     to cut them away in further screenshots
    fn calibrate(&mut self, window_id: WindowId) -> Result<()>;
    fn window_list(&self) -> Result<WindowList>;
    fn capture_window_screenshot(&self, window_id: WindowId) -> Result<ImageOnHeap>;
    fn get_active_window(&self) -> Result<WindowId>;
}
