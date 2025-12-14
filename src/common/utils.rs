use anyhow::Context;
use humantime::{format_duration, parse_duration};
use std::ops::Add;
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
            let mut less = Duration::from_millis(self.as_millis() as u64);
            let mut prefix = "";
            if less < *self {
                prefix = "~";
                less = less.add(Duration::from_millis(1))
            }
            format!("{}{}", prefix, format_duration(less))
        }
    }
}

/// escape sequences that clears the screen
pub fn clear_screen() {
    print!("{esc}[2J", esc = 27 as char);
    print!("{esc}[H", esc = 27 as char);
}

/// Print items in a tree-style format
pub fn print_tree_list<I, S>(items: I)
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let items: Vec<_> = items.into_iter().collect();
    let last_idx = items.len().saturating_sub(1);
    for (i, item) in items.iter().enumerate() {
        let prefix = if i == last_idx { "└─" } else { "├─" };
        println!("   {} {}", prefix, item.as_ref());
    }
}

/// parses a human duration string into something valid
pub fn parse_delay(
    s: Option<impl AsRef<str>>,
    t: impl AsRef<str>,
) -> crate::Result<Option<Duration>> {
    if let Some(d) = s.map(|s| parse_duration(s.as_ref())) {
        let d = d.with_context(|| {
            format!(
                "{} had an invalid format, allowed is 0ms < XXs <= 5m",
                t.as_ref()
            )
        })?;
        if d > MAX_DELAY {
            anyhow::bail!(
                "{} was out of range, allowed is 0ms < XXs <= 5m",
                t.as_ref()
            )
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
        assert_eq!(Duration::from_micros(10).as_human_readable(), "~1ms");
        assert_eq!(Duration::from_nanos(10).as_human_readable(), "~1ms");
        // this is 1.12ms so ~2ms
        assert_eq!(Duration::from_micros(1120).as_human_readable(), "~2ms");
    }

    #[test]
    fn should_parse_time() -> crate::Result<()> {
        let s = parse_delay(Some("1m"), "foo")?.unwrap();
        assert_eq!(s, Duration::from_secs(60));
        let s = parse_delay(Some("60s"), "foo")?.unwrap();
        assert_eq!(s, Duration::from_secs(60));
        let s = parse_delay(Some("500ms"), "foo")?.unwrap();
        assert_eq!(s, Duration::from_millis(500));

        Ok(())
    }

    #[test]
    #[should_panic(expected = "foo was out of range, allowed is 0ms < XXs <= 5m")]
    fn should_not_parse_time_that_is_too_long() {
        parse_delay(Some("5m 1s"), "foo").unwrap();
    }

    #[test]
    #[should_panic(expected = "time unit needed, for example 1sec or 1ms")]
    fn should_not_parse_time_that_is_invalid() {
        parse_delay(Some("1"), "foo").unwrap();
    }
}
