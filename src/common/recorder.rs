use crate::{ImageOnHeap, PlatformApi, WindowId};

use smol::future::FutureExt;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use smol::stream::Stream;
use smol::Timer;

struct Recorder<A: PlatformApi> {
    api: A,
    window_id: WindowId,
    timer: Timer,
    last_frame_timestamp: Option<Instant>,
}

impl<A: PlatformApi> Recorder<A> {
    pub fn new(api: A, window_id: WindowId, fps: u8) -> Self {
        let fps = Duration::from_millis(1000 / fps as u64);
        Self {
            api,
            window_id,
            timer: Timer::interval(fps),
            last_frame_timestamp: None,
        }
    }
}

impl<A: PlatformApi> Stream for Recorder<A> {
    type Item = ImageOnHeap;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        let now = Instant::now();
        match this.timer.poll(_cx) {
            Poll::Ready(_) => {
                let d = now.duration_since(this.last_frame_timestamp.unwrap_or(now));
                dbg!(d);
                this.last_frame_timestamp.replace(now);
                Poll::Ready(this.api.capture_window_screenshot(this.window_id).ok())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use super::*;
    use crate::macos::*;
    use smol::stream::block_on;

    #[test]
    fn should_record_not() {
        let api = setup().unwrap();
        let rec = Recorder::new(api, 682, 4);

        let mut i = 0;
        for _img in block_on(rec) {
            i += 1;
            if i >= 4 {
                println!("Done with testing..");
                break;
            }
        }
    }
}
