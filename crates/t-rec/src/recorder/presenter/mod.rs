//! Visual feedback during recording (screenshot indicators, keystroke overlays).
//!
//! On macOS with `osd-flash-indicator` feature enabled, uses core-animation for on-screen display.
//! Otherwise falls back to a no-op implementation.
//!
//! The Presenter must run on the main thread due to macOS requirements.

#[cfg(all(target_os = "macos", feature = "osd-flash-indicator"))]
mod macos;
mod noop;

use std::thread;
use std::time::Duration;
use tokio::sync::broadcast;

use crate::event_router::{CaptureEvent, Event, FlashEvent, LifecycleEvent};
use crate::Result;
use crate::WindowId;

pub trait Presenter {
    fn handle_event(&mut self, event: FlashEvent) -> Result<()>;

    fn run(&mut self, mut rx: broadcast::Receiver<Event>) -> Result<()> {
        loop {
            match rx.try_recv() {
                Ok(Event::Flash(event)) => {
                    if let Err(e) = self.handle_event(event) {
                        log::error!("Presenter error: {}", e);
                    }
                }
                Ok(Event::Capture(CaptureEvent::Stop)) => {
                    log::debug!("Presenter received Stop");
                    break;
                }
                Ok(Event::Lifecycle(LifecycleEvent::Shutdown)) => {
                    log::debug!("Presenter received Shutdown");
                    break;
                }
                Ok(_) => {} // Ignore other events
                Err(broadcast::error::TryRecvError::Closed) => break,
                Err(_) => {}
            }
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }
}

#[cfg(all(target_os = "macos", feature = "osd-flash-indicator"))]
pub fn create_presenter(win_id: WindowId) -> impl Presenter {
    macos::OsdPresenter::new(win_id)
}

#[cfg(not(all(target_os = "macos", feature = "osd-flash-indicator")))]
pub fn create_presenter(win_id: WindowId) -> impl Presenter {
    noop::NoopPresenter::new(win_id)
}
