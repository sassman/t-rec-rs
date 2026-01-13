use std::fmt;

/// A platform-independent window identifier.
///
/// This newtype wraps a `u64` to provide type safety for window identifiers
/// across different platforms (macOS, Linux/X11, Windows).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct WindowId(u64);

impl WindowId {
    /// Creates a new `WindowId` from a raw `u64` value.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the raw `u64` value of this window identifier.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns the raw `u32` value of this window identifier.
    ///
    /// This is useful for X11 which uses 32-bit window IDs.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0 as u32
    }
}

impl From<u64> for WindowId {
    #[inline]
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<u32> for WindowId {
    #[inline]
    fn from(id: u32) -> Self {
        Self(id as u64)
    }
}

impl From<WindowId> for u64 {
    #[inline]
    fn from(id: WindowId) -> Self {
        id.0
    }
}

impl From<WindowId> for u32 {
    #[inline]
    fn from(id: WindowId) -> Self {
        id.0 as u32
    }
}

impl fmt::Display for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for WindowId {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<u64>().map(WindowId::new)
    }
}
