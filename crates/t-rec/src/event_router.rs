use tokio::sync::broadcast::Sender;

/// Capture commands for the Photographer actor.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum CaptureEvent {
    Start,
    Screenshot { timecode_ms: u128 },
    Stop,
}

/// Visual feedback events for the Presenter actor.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum FlashEvent {
    ScreenshotTaken,
    #[allow(dead_code)]
    KeyPressed {
        key: String,
    },
    RecordingStarted,
}

/// Lifecycle events for actor coordination.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum LifecycleEvent {
    Shutdown,
    #[allow(dead_code)]
    Error(String),
}

/// Unified event type for all actors.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Event {
    Capture(CaptureEvent),
    Flash(FlashEvent),
    Lifecycle(LifecycleEvent),
}

/// Broadcasts events to all actors.
#[derive(Clone)]
pub struct EventRouter {
    tx: Sender<Event>,
}

impl EventRouter {
    pub fn new() -> Self {
        let (tx, _rx) = tokio::sync::broadcast::channel::<Event>(100);
        Self { tx }
    }

    /// Broadcast an event to all subscribed actors.
    pub fn send(&self, event: Event) {
        let _ = self.tx.send(event);
    }

    /// like `send` but returns a `Result`
    #[allow(dead_code)]
    pub fn try_send(
        &self,
        event: Event,
    ) -> Result<usize, tokio::sync::broadcast::error::SendError<Event>> {
        self.tx.send(event)
    }

    /// Sends a shutdown event to all actors.
    #[allow(dead_code)]
    pub fn shutdown(&self) {
        let _ = self.tx.send(Event::Lifecycle(LifecycleEvent::Shutdown));
    }

    /// Subscribes to events from the router.
    /// Returns a receiver that listens for broadcasted events.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_router() {
        let router = EventRouter::new();
        let mut receiver = router.subscribe();

        router.send(Event::Capture(CaptureEvent::Start));
        router.send(Event::Flash(FlashEvent::ScreenshotTaken));

        assert!(matches!(
            receiver.blocking_recv(),
            Ok(Event::Capture(CaptureEvent::Start))
        ));
        assert!(matches!(
            receiver.blocking_recv(),
            Ok(Event::Flash(FlashEvent::ScreenshotTaken))
        ));
    }

    #[test]
    fn test_event_router_shutdown() {
        let router = EventRouter::new();
        let mut receiver = router.subscribe();

        router.shutdown();

        assert!(matches!(
            receiver.blocking_recv(),
            Ok(Event::Lifecycle(LifecycleEvent::Shutdown))
        ));
    }
}
