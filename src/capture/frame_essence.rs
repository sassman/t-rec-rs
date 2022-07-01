use blockhash::{blockhash256, Blockhash256};

use crate::capture::{Frame, Timecode};

#[derive(Eq, PartialEq, Clone)]
pub enum FrameDropStrategy {
    DoNotDropAny,
    DropIdenticalFrames,
}

#[derive(Eq, PartialOrd, PartialEq, Clone)]
pub struct FrameEssence {
    pub(crate) when: Timecode,
    pub(crate) what: Blockhash256,
}

impl FrameEssence {
    pub fn new(frame: &Frame, timecode: &Timecode) -> Self {
        Self {
            when: timecode.clone(),
            what: blockhash256(frame),
        }
    }
}
