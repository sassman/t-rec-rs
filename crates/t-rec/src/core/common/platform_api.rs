use crate::core::{ImageOnHeap, Result, WindowId, WindowList};

pub struct Platform;

pub trait PlatformApiFactory {
    fn setup() -> Result<Box<dyn PlatformApi>>;
}

pub trait PlatformApi: Send {
    /// 1. it does check for the screenshot
    /// 2. it checks for transparent margins and configures the api
    ///    to cut them away in further screenshots
    fn calibrate(&mut self, window_id: WindowId) -> Result<()>;
    fn window_list(&self) -> Result<WindowList>;
    fn capture_window_screenshot(&self, window_id: WindowId) -> Result<ImageOnHeap>;
    fn get_active_window(&self) -> Result<WindowId>;
}

/// Blanket implementation for boxed trait objects.
impl<T: PlatformApi + ?Sized> PlatformApi for Box<T> {
    fn calibrate(&mut self, window_id: WindowId) -> Result<()> {
        (**self).calibrate(window_id)
    }

    fn window_list(&self) -> Result<WindowList> {
        (**self).window_list()
    }

    fn capture_window_screenshot(&self, window_id: WindowId) -> Result<ImageOnHeap> {
        (**self).capture_window_screenshot(window_id)
    }

    fn get_active_window(&self) -> Result<WindowId> {
        (**self).get_active_window()
    }
}
