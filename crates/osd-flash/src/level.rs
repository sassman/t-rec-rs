//! Window level types for OSD z-ordering.

/// Window level determining the z-order of the overlay window.
///
/// Controls where the window appears in the window stack relative to other windows.
/// Higher levels appear above lower levels.
///
/// # Common levels
///
/// - `Normal` (0): Standard application windows
/// - `Floating` (3): Floating palettes, tool windows
/// - `ModalPanel` (8): Modal panels that block interaction
/// - `ScreenSaver` (1000): Screen saver level
/// - `AboveAll` (1001): Above all windows including fullscreen apps
///
/// # Example
///
/// ```ignore
/// use osd_flash::prelude::*;
///
/// OsdBuilder::new()
///     .level(WindowLevel::AboveAll)
///     .show_for(3.seconds())?;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowLevel {
    /// Normal window level (0), appears with regular application windows.
    Normal,
    /// Floating window level (3), appears above normal windows.
    Floating,
    /// Modal panel level (8), appears above floating windows.
    ModalPanel,
    /// Screen saver level (1000).
    ScreenSaver,
    /// Above all other windows (1001), including fullscreen apps and the Dock.
    #[default]
    AboveAll,
    /// Custom window level value (platform-specific).
    Custom(isize),
}

impl WindowLevel {
    /// Returns the raw window level value.
    ///
    /// This is the platform-specific integer value used by the underlying window system.
    pub fn raw_level(&self) -> isize {
        match self {
            WindowLevel::Normal => 0,
            WindowLevel::Floating => 3,
            WindowLevel::ModalPanel => 8,
            WindowLevel::ScreenSaver => 1000,
            WindowLevel::AboveAll => 1001,
            WindowLevel::Custom(level) => *level,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        assert_eq!(WindowLevel::default(), WindowLevel::AboveAll);
    }

    #[test]
    fn test_raw_levels() {
        assert_eq!(WindowLevel::Normal.raw_level(), 0);
        assert_eq!(WindowLevel::Floating.raw_level(), 3);
        assert_eq!(WindowLevel::ModalPanel.raw_level(), 8);
        assert_eq!(WindowLevel::ScreenSaver.raw_level(), 1000);
        assert_eq!(WindowLevel::AboveAll.raw_level(), 1001);
        assert_eq!(WindowLevel::Custom(500).raw_level(), 500);
    }
}
