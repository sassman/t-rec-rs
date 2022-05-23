use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use smol::future::FutureExt;
use smol::stream::Stream;
use smol::Timer;

use crate::common::Frame;
use crate::{PlatformApi, WindowId};

pub struct Recorder<A: PlatformApi> {
    api: A,
    window_id: WindowId,
    timer: Timer,
}

impl<A: PlatformApi> Recorder<A> {
    pub fn new(api: A, window_id: WindowId, fps: u8) -> Self {
        let fps = Duration::from_millis(1000 / fps as u64);
        Self {
            api,
            window_id,
            timer: Timer::interval(fps),
        }
    }
}

impl<A: PlatformApi> Stream for Recorder<A> {
    type Item = Frame;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.timer.poll(_cx) {
            Poll::Ready(_) => Poll::Ready(this.api.capture_window_screenshot(this.window_id).ok()),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
#[cfg(target_os = "macos")]
#[cfg(feature = "e2e_tests")]
mod tests {
    use smol::stream::block_on;

    use crate::macos::*;

    use super::*;

    #[test]
    fn should_record_not() {
        let win = 9123;
        let mut api = setup().unwrap();
        api.calibrate(win).unwrap();
        let rec = Recorder::new(api, win, 10);

        let mut i = 0;
        for _img in block_on(rec) {
            i += 1;
            if i >= 5 {
                println!("Done with testing..");
                break;
            }
        }
    }

    // #[test]
    // fn should_queue_frames_for_saving() {
    //     let win = 9123;
    //     let mut api = setup().unwrap();
    //     api.calibrate(win).unwrap();
    //     let mut rec = Recorder::new(api, win, 10);
    //
    //     let future = rec.next();
    //     {
    //         let (sender, receiver) = flume::unbounded();
    //
    //         // A function that schedules the task when it gets woken up.
    //         let schedule = move |runnable| sender.send(runnable).unwrap();
    //
    //         // Construct a task.
    //         let (runnable, task) = async_task::spawn(future, schedule);
    //
    //         // Push the task into the queue by invoking its schedule function.
    //         runnable.schedule();
    //
    //         for runnable in receiver {
    //             runnable.run();
    //         }
    //     }
    // }
}
