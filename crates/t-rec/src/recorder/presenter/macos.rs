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

/// Show a camera flash indicator using the new OSD API.
fn show_camera_flash() -> osd_flash::Result<()> {
    OsdBuilder::new()
        .size(120.0)
        .position(Position::TopRight)
        .margin(20.0)
        .level(WindowLevel::AboveAll)
        .background(Color::rgba(0.15, 0.45, 0.9, 0.92))
        .corner_radius(20.0)
        // Camera body
        .layer("body", |l| {
            l.rounded_rect(70.0, 45.0, 8.0).center().fill(Color::WHITE)
        })
        // Viewfinder bump
        .layer("viewfinder", |l| {
            l.rounded_rect(20.0, 10.0, 3.0)
                .center_offset(0.0, 22.0)
                .fill(Color::WHITE)
        })
        // Lens outer ring
        .layer("lens_outer", |l| {
            l.circle(32.0)
                .center()
                .fill(Color::rgba(0.2, 0.3, 0.5, 1.0))
        })
        // Lens inner
        .layer("lens_inner", |l| {
            l.circle(22.0)
                .center()
                .fill(Color::rgba(0.1, 0.15, 0.3, 1.0))
        })
        // Lens highlight
        .layer("lens_highlight", |l| {
            l.circle(8.0)
                .center_offset(-4.0, 4.0)
                .fill(Color::rgba(1.0, 1.0, 1.0, 0.4))
        })
        // Flash indicator
        .layer("flash", |l| {
            l.circle(10.0)
                .center_offset(22.0, 12.0)
                .fill(Color::rgba(1.0, 0.85, 0.2, 1.0))
        })
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
