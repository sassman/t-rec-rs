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
    #[allow(dead_code)]
    win_id: WindowId,
}

impl SkylightPresenter {
    pub fn new(win_id: WindowId) -> Self {
        Self { win_id }
    }
}

/// Show a camera flash indicator using the library composition.
fn show_camera_flash() -> osd_flash::Result<()> {
    let camera = CameraFlash::new();
    OsdBuilder::new()
        .position(Position::TopRight)
        .margin(20.0)
        .level(WindowLevel::AboveAll)
        .background(camera.get_background_color())
        .corner_radius(camera.get_corner_radius())
        .composition(camera)
        .show_for(1500.millis())
}

impl Presenter for SkylightPresenter {
    fn handle_event(&mut self, event: FlashEvent) -> Result<()> {
        match event {
            FlashEvent::ScreenshotTaken => {
                log::debug!(
                    "Screenshot taken - showing indicator for window {}",
                    self.win_id
                );
                if let Err(e) = show_camera_flash() {
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
