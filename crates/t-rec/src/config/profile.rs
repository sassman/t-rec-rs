use anyhow::Result;
use serde::Deserialize;

use super::defaults;
use super::ConfigFile;
use crate::cli::CliArgs;

/// Settings that can be specified in a profile (all optional for merging)
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct ProfileSettings {
    pub verbose: Option<bool>,
    pub quiet: Option<bool>,
    pub video: Option<bool>,
    pub video_only: Option<bool>,
    pub decor: Option<String>,
    pub wallpaper: Option<String>,
    pub wallpaper_padding: Option<u32>,
    pub bg: Option<String>,
    pub natural: Option<bool>,
    pub end_pause: Option<String>,
    pub start_pause: Option<String>,
    pub idle_pause: Option<String>,
    pub output: Option<String>,
    pub fps: Option<u8>,
}

impl ProfileSettings {
    /// Merge another profile into this one (other takes precedence)
    pub fn merge(&mut self, other: &ProfileSettings) {
        macro_rules! merge_field {
            ($field:ident) => {
                if other.$field.is_some() {
                    self.$field = other.$field.clone();
                }
            };
        }
        merge_field!(verbose);
        merge_field!(quiet);
        merge_field!(video);
        merge_field!(video_only);
        merge_field!(decor);
        merge_field!(wallpaper);
        merge_field!(wallpaper_padding);
        merge_field!(bg);
        merge_field!(natural);
        merge_field!(end_pause);
        merge_field!(start_pause);
        merge_field!(idle_pause);
        merge_field!(output);
        merge_field!(fps);
    }

    /// Get final values with defaults applied
    pub fn verbose(&self) -> bool {
        self.verbose.unwrap_or(false)
    }
    pub fn quiet(&self) -> bool {
        self.quiet.unwrap_or(false)
    }
    pub fn video(&self) -> bool {
        self.video.unwrap_or(false)
    }
    pub fn video_only(&self) -> bool {
        self.video_only.unwrap_or(false)
    }
    pub fn natural(&self) -> bool {
        self.natural.unwrap_or(false)
    }
    pub fn decor(&self) -> &str {
        self.decor.as_deref().unwrap_or(defaults::DECOR)
    }
    pub fn bg(&self) -> &str {
        self.bg.as_deref().unwrap_or(defaults::BG)
    }
    pub fn wallpaper_padding(&self) -> u32 {
        self.wallpaper_padding
            .unwrap_or(defaults::WALLPAPER_PADDING)
    }
    pub fn idle_pause(&self) -> &str {
        self.idle_pause.as_deref().unwrap_or(defaults::IDLE_PAUSE)
    }
    pub fn output(&self) -> &str {
        self.output.as_deref().unwrap_or(defaults::OUTPUT)
    }
    pub fn fps(&self) -> u8 {
        self.fps.unwrap_or(defaults::FPS)
    }
}

impl From<&CliArgs> for ProfileSettings {
    fn from(args: &CliArgs) -> Self {
        ProfileSettings {
            // Flags: only set if true (otherwise None lets config win)
            verbose: if args.verbose { Some(true) } else { None },
            quiet: if args.quiet { Some(true) } else { None },
            video: if args.video { Some(true) } else { None },
            video_only: if args.video_only { Some(true) } else { None },
            natural: if args.natural { Some(true) } else { None },

            // Values: None if user didn't provide, Some if they did
            decor: args.decor.clone(),
            wallpaper: args.wallpaper.clone(),
            wallpaper_padding: args.wallpaper_padding,
            bg: args.bg.clone(),
            end_pause: args.end_pause.clone(),
            start_pause: args.start_pause.clone(),
            idle_pause: args.idle_pause.clone(),
            output: args.output.clone(),
            fps: args.fps,
        }
    }
}

/// Expand $HOME in a string value (only $HOME is supported)
pub fn expand_home(value: &str) -> String {
    if value.contains("$HOME") {
        if let Some(home) = dirs::home_dir() {
            return value.replace("$HOME", &home.to_string_lossy());
        }
    }
    value.to_string()
}

/// Resolve settings: default -> profile -> CLI args
pub fn resolve_settings(
    config: Option<&ConfigFile>,
    profile_name: Option<&str>,
    args: &CliArgs,
) -> Result<ProfileSettings> {
    let mut settings = ProfileSettings::default();

    if let Some(config) = config {
        // Apply default section
        settings.merge(&config.default);

        // Apply named profile if specified
        if let Some(name) = profile_name {
            if let Some(profile) = config.profiles.get(name) {
                settings.merge(profile);
            } else {
                let available: Vec<_> = config.profiles.keys().cloned().collect();
                if available.is_empty() {
                    anyhow::bail!(
                        "Profile '{}' not found. No profiles defined in config.",
                        name
                    );
                } else {
                    anyhow::bail!(
                        "Profile '{}' not found. Available profiles: {}",
                        name,
                        available.join(", ")
                    );
                }
            }
        }
    }

    // CLI args override everything
    settings.merge(&ProfileSettings::from(args));

    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_home() {
        let home = dirs::home_dir().unwrap();
        let home_str = home.to_string_lossy();

        assert_eq!(
            expand_home("$HOME/Pictures/bg.png"),
            format!("{}/Pictures/bg.png", home_str)
        );
        assert_eq!(expand_home("/absolute/path.png"), "/absolute/path.png");
        assert_eq!(expand_home("relative/path.png"), "relative/path.png");
    }

    #[test]
    fn test_profile_merge() {
        let mut base = ProfileSettings {
            wallpaper: Some("ventura".to_string()),
            wallpaper_padding: Some(60),
            ..Default::default()
        };

        let overlay = ProfileSettings {
            wallpaper_padding: Some(100),
            quiet: Some(true),
            ..Default::default()
        };

        base.merge(&overlay);

        assert_eq!(base.wallpaper, Some("ventura".to_string()));
        assert_eq!(base.wallpaper_padding, Some(100));
        assert_eq!(base.quiet, Some(true));
    }

    #[test]
    fn test_default_values() {
        let settings = ProfileSettings::default();

        assert!(!settings.verbose());
        assert!(!settings.quiet());
        assert_eq!(settings.decor(), defaults::DECOR);
        assert_eq!(settings.bg(), defaults::BG);
        assert_eq!(settings.wallpaper_padding(), defaults::WALLPAPER_PADDING);
        assert_eq!(settings.idle_pause(), defaults::IDLE_PAUSE);
        assert_eq!(settings.output(), defaults::OUTPUT);
        assert_eq!(settings.fps(), defaults::FPS);
    }
}
