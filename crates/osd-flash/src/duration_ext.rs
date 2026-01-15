//! Duration extension trait for ergonomic duration creation.
//!
//! # Example
//!
//! ```
//! use osd_flash::prelude::*;
//!
//! let two_seconds = 2.seconds();
//! let half_second = 500.millis();
//! ```

use std::time::Duration;

/// Extension trait for creating `Duration` from numeric types.
///
/// This provides a more ergonomic API for specifying durations,
/// especially useful for animation timing.
pub trait DurationExt {
    /// Create a `Duration` representing this many seconds.
    fn seconds(self) -> Duration;

    /// Create a `Duration` representing this many milliseconds.
    fn millis(self) -> Duration;
}

impl DurationExt for u64 {
    fn seconds(self) -> Duration {
        Duration::from_secs(self)
    }

    fn millis(self) -> Duration {
        Duration::from_millis(self)
    }
}

impl DurationExt for u32 {
    fn seconds(self) -> Duration {
        Duration::from_secs(self as u64)
    }

    fn millis(self) -> Duration {
        Duration::from_millis(self as u64)
    }
}

impl DurationExt for usize {
    fn seconds(self) -> Duration {
        Duration::from_secs(self as u64)
    }

    fn millis(self) -> Duration {
        Duration::from_millis(self as u64)
    }
}

impl DurationExt for i32 {
    fn seconds(self) -> Duration {
        Duration::from_secs(self.max(0) as u64)
    }

    fn millis(self) -> Duration {
        Duration::from_millis(self.max(0) as u64)
    }
}

impl DurationExt for f64 {
    fn seconds(self) -> Duration {
        Duration::from_secs_f64(self.max(0.0))
    }

    fn millis(self) -> Duration {
        Duration::from_secs_f64(self.max(0.0) / 1000.0)
    }
}

impl DurationExt for f32 {
    fn seconds(self) -> Duration {
        Duration::from_secs_f32(self.max(0.0))
    }

    fn millis(self) -> Duration {
        Duration::from_secs_f32(self.max(0.0) / 1000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u64_seconds() {
        assert_eq!(2u64.seconds(), Duration::from_secs(2));
    }

    #[test]
    fn test_u64_millis() {
        assert_eq!(500u64.millis(), Duration::from_millis(500));
    }

    #[test]
    fn test_u32_seconds() {
        assert_eq!(3u32.seconds(), Duration::from_secs(3));
    }

    #[test]
    fn test_usize_seconds() {
        assert_eq!(5usize.seconds(), Duration::from_secs(5));
    }

    #[test]
    fn test_i32_seconds() {
        assert_eq!(4i32.seconds(), Duration::from_secs(4));
        // Negative values clamp to 0
        assert_eq!((-1i32).seconds(), Duration::from_secs(0));
    }
}
