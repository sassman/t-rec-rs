pub mod identify_transparency;
pub mod image;
pub mod utils;

use crate::{ImageOnHeap, Result, WindowId, WindowList};

pub trait PlatformApi: Send {
    /// 1. it does check for the screenshot
    /// 2. it checks for transparent margins and configures the api
    ///     to cut them away in further screenshots
    fn calibrate(&mut self, window_id: WindowId) -> Result<()>;
    fn window_list(&self) -> Result<WindowList>;
    fn capture_window_screenshot(&self, window_id: WindowId) -> Result<ImageOnHeap>;
    fn get_active_window(&self) -> Result<WindowId>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Margin {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl Margin {
    pub fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn new_equal(margin: u16) -> Self {
        Self::new(margin, margin, margin, margin)
    }

    pub fn zero() -> Self {
        Self::new_equal(0)
    }

    pub fn is_zero(&self) -> bool {
        self.top == 0
            && self.right == self.left
            && self.left == self.bottom
            && self.bottom == self.top
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn margin_new() {
        let m = Margin::new(1, 2, 3, 4);
        assert_eq!(m.top, 1);
        assert_eq!(m.right, 2);
        assert_eq!(m.bottom, 3);
        assert_eq!(m.left, 4);
    }

    #[test]
    fn margin_new_equal() {
        let m = Margin::new_equal(1);
        assert_eq!(m.top, 1);
        assert_eq!(m.right, 1);
        assert_eq!(m.bottom, 1);
        assert_eq!(m.left, 1);
    }

    #[test]
    fn margin_zero() {
        let m = Margin::zero();
        assert_eq!(m.top, 0);
        assert_eq!(m.right, 0);
        assert_eq!(m.bottom, 0);
        assert_eq!(m.left, 0);
        assert!(m.is_zero());
    }
}
