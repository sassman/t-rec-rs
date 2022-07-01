use std::collections::LinkedList;

use crate::capture::{FrameDropStrategy, FrameEssence};

pub struct FrameComparator {
    last_frames: LinkedList<FrameEssence>,
    strategy: FrameDropStrategy,
}

impl FrameComparator {
    pub fn new(strategy: FrameDropStrategy) -> Self {
        Self {
            last_frames: LinkedList::new(),
            strategy,
        }
    }
}

impl FrameComparator {
    pub fn should_drop_frame(&mut self, frame_essence: FrameEssence) -> bool {
        match self.strategy {
            FrameDropStrategy::DoNotDropAny => false,
            FrameDropStrategy::DropIdenticalFrames => {
                if let Some(FrameEssence { when, what }) = self.last_frames.pop_back() {
                    if frame_essence.when > when && what == frame_essence.what {
                        // so the last frame and this one is the same... so let's drop it..
                        // but add the current frame
                        self.last_frames.push_back(frame_essence);
                        true
                    } else {
                        let previous_should_drop_frame = self.should_drop_frame(frame_essence);
                        // restore the popped frame..
                        self.last_frames.push_back(FrameEssence { when, what });

                        previous_should_drop_frame
                    }
                } else {
                    self.last_frames.push_back(frame_essence);

                    false
                }
            }
        }
    }
}
