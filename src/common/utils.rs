use anyhow::Context;
use humantime::{format_duration, parse_duration};
use std::time::Duration;

const ONE_MIN: Duration = Duration::from_secs(60);
const ONE_SEC: Duration = Duration::from_secs(1);
const MAX_DELAY: Duration = Duration::from_secs(5 * 60);

pub trait HumanReadable {
    fn as_human_readable(&self) -> String;
}

impl HumanReadable for Duration {
    fn as_human_readable(&self) -> String {
        if self >= &ONE_SEC && self < &ONE_MIN {
            format!("~{}s", self.as_secs_f32().round())
        } else {
            format_duration(*self).to_string()
        }
    }
}

/// escape sequences that clears the screen
pub fn clear_screen() {
    print!("{esc}[2J", esc = 27 as char);
    print!("{esc}[H", esc = 27 as char);
}

/// parses a human duration string into something valid
pub fn parse_delay(s: Option<&str>, t: &str) -> crate::Result<Option<Duration>> {
    if let Some(d) = s.map(parse_duration) {
        let d =
            d.with_context(|| format!("{} had an valid format, allowed is 0ms < XXs <= 5m", t))?;
        if d > MAX_DELAY {
            anyhow::bail!("{} was out of range, allowed is 0ms < XXs <= 5m", t)
        } else {
            Ok(Some(d))
        }
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_format_time() {
        assert_eq!(
            Duration::from_secs(60 * 60 + 1).as_human_readable(),
            "1h 1s"
        );
        assert_eq!(Duration::from_secs(100).as_human_readable(), "1m 40s");
        assert_eq!(Duration::from_millis(1200).as_human_readable(), "~1s");
        assert_eq!(Duration::from_millis(1800).as_human_readable(), "~2s");
        assert_eq!(Duration::from_millis(100).as_human_readable(), "100ms");
        assert_eq!(Duration::from_micros(10).as_human_readable(), "10us");
        assert_eq!(Duration::from_nanos(10).as_human_readable(), "10ns");
    }
}
