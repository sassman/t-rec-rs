use tokio::sync::broadcast::Sender;

/// Capture commands for the Photographer actor.
#[derive(Debug, Clone)]
pub enum CaptureEvent {
    Start,
    /// Manual screenshot request (CLI only).
    #[cfg(feature = "cli")]
    Screenshot {
        timecode_ms: u128,
    },
    Stop,
}

/// Visual feedback events for the Presenter actor (CLI only).
#[cfg(feature = "cli")]
#[derive(Debug, Clone)]
pub enum FlashEvent {
    ScreenshotTaken,
    RecordingStarted,
}

/// Lifecycle events for actor coordination (CLI only).
#[cfg(feature = "cli")]
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    Shutdown,
}

/// Unified event type for all actors.
#[derive(Debug, Clone)]
pub enum Event {
    Capture(CaptureEvent),
    /// Visual feedback events (CLI only).
    #[cfg(feature = "cli")]
    Flash(FlashEvent),
    /// Lifecycle events (CLI only).
    #[cfg(feature = "cli")]
    Lifecycle(LifecycleEvent),
}

/// Broadcasts events to all actors.
#[derive(Clone)]
pub struct EventRouter {
    tx: Sender<Event>,
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
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

    /// like `send` but returns a `Result` (CLI only).
    #[cfg(feature = "cli")]
    pub fn try_send(
        &self,
        event: Event,
    ) -> Result<usize, tokio::sync::broadcast::error::SendError<Event>> {
        self.tx.send(event)
    }

    /// Sends a shutdown event to all actors (CLI only).
    #[cfg(feature = "cli")]
    pub fn shutdown(&self) {
        let _ = self.tx.send(Event::Lifecycle(LifecycleEvent::Shutdown));
    }

    /// Subscribes to events from the router.
    /// Returns a receiver that listens for broadcasted events.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.tx.subscribe()
    }
}

#[cfg(all(test, feature = "cli"))]
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
    fn test_event_broadcast() {
        let router = EventRouter::new();
        let mut receiver1 = router.subscribe();
        let mut receiver2 = router.subscribe();

        router.send(Event::Capture(CaptureEvent::Start));
        assert!(matches!(
            receiver1.blocking_recv(),
            Ok(Event::Capture(CaptureEvent::Start))
        ));
        assert!(matches!(
            receiver2.blocking_recv(),
            Ok(Event::Capture(CaptureEvent::Start))
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
