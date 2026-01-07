//! No-op presenter for platforms without visual feedback.

use super::Presenter;
use crate::event_router::FlashEvent;
use crate::Result;
use crate::WindowId;

pub struct NoopPresenter;

impl NoopPresenter {
    pub fn new(_win_id: WindowId) -> Self {
        Self
    }
}

impl Presenter for NoopPresenter {
    fn handle_event(&mut self, _event: FlashEvent) -> Result<()> {
        Ok(())
    }
}
