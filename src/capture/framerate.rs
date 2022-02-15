use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub struct Framerate(u32);

impl Framerate {
    pub fn new(f: u32) -> Self {
        Self(f)
    }
}

impl Display for Framerate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let framerate = self.0;
        write!(f, "framerate {framerate} [fps]")
    }
}

impl From<u32> for Framerate {
    fn from(fr: u32) -> Self {
        Self(fr)
    }
}

impl AsRef<u32> for Framerate {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}
