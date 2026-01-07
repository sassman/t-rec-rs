//! macOS presenter using Skylight for on-screen display.
//!
//! This module is only compiled when both:
//! - `target_os = "macos"`
//! - `feature = "osd-flash-indicator"`

use super::Presenter;
use crate::event_router::FlashEvent;
use crate::Result;
use crate::WindowId;
use osd_flash::prelude::*;

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
                log::debug!(
                    "Screenshot taken - showing indicator for window {}",
                    self.win_id
                );
                if let Err(e) = OsdFlashBuilder::new()
                    .dimensions(120.0)
                    .position(FlashPosition::TopRight)
                    .margin(20.0)
                    .level(WindowLevel::AboveAll)
                    .attach_to_window(self.win_id)
                    .build()
                    .and_then(|w| w.draw(CameraIcon::new(120.0).build()).show_for_seconds(1.5))
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
