//! macOS presenter using Skylight for on-screen display.

use super::Presenter;
use crate::event_router::FlashEvent;
use crate::macos::screen_flash;
use crate::Result;
use crate::WindowId;

pub struct SkylightPresenter {
    win_id: WindowId,
}

impl SkylightPresenter {
    pub fn new(win_id: WindowId) -> Self {
        Self { win_id }
    }
}

impl Presenter for SkylightPresenter {
    fn handle_event(&mut self, event: FlashEvent) -> Result<()> {
        match event {
            FlashEvent::ScreenshotTaken => {
                log::debug!("Screenshot taken - showing indicator");
                let flash_config = screen_flash::FlashConfig::default()
                    .position(screen_flash::FlashPosition::TopRight)
                    .duration(1.5);
                if let Err(e) =
                    screen_flash::show_indicator_screenshot_indicator(&flash_config, self.win_id)
                {
                    log::error!("Cannot show the screenshot indicator: {}", e);
                }
            }
            FlashEvent::KeyPressed { key: _ } => {
                // Future: keystroke overlay
                log::debug!("Key press overlay not yet implemented");
            }
        }
        Ok(())
    }
}
