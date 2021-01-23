use std::time::Duration;

const ONE_MIN: Duration = Duration::from_secs(60);
const ONE_SEC: Duration = Duration::from_secs(1);

pub trait HumanReadable {
    fn as_human_readable(&self) -> String;
}

impl HumanReadable for Duration {
    fn as_human_readable(&self) -> String {
        if self >= &ONE_MIN {
            let time = (self.as_secs() / 60) as u128;
            let seconds = self.as_secs() - (time * 60) as u64;
            return format!("{}m {}s", time, seconds);
        } else if self >= &ONE_SEC {
            let unit = "s";
            return format!("~{}{}", self.as_secs_f32().round(), unit);
        }
        let time = self.as_millis();
        let unit = "ms";

        format!("{}{}", time, unit)
    }
}

/// escape sequences that clears the screen
pub fn clear_screen() {
    print!("{esc}[2J", esc = 27 as char);
    print!("{esc}[H", esc = 27 as char);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_format_time() {
        assert_eq!(Duration::from_secs(100).as_human_readable(), "1m 40s");
        assert_eq!(Duration::from_millis(1200).as_human_readable(), "~1s");
        assert_eq!(Duration::from_millis(1800).as_human_readable(), "~2s");
        assert_eq!(Duration::from_millis(100).as_human_readable(), "100ms");
    }
}
