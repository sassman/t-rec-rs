#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct Timecode(u32);

impl AsRef<u32> for Timecode {
    fn as_ref(&self) -> &u32 {
        &self.0
    }
}

impl From<u128> for Timecode {
    fn from(x: u128) -> Self {
        Self(x as u32)
    }
}
