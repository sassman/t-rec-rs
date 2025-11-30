use anyhow::Result;
use clap::ArgMatches;
use serde::Deserialize;

use super::ConfigFile;

/// Settings that can be specified in a profile
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
        if other.verbose.is_some() {
            self.verbose = other.verbose;
        }
        if other.quiet.is_some() {
            self.quiet = other.quiet;
        }
        if other.video.is_some() {
            self.video = other.video;
        }
        if other.video_only.is_some() {
            self.video_only = other.video_only;
        }
        if other.decor.is_some() {
            self.decor = other.decor.clone();
        }
        if other.wallpaper.is_some() {
            self.wallpaper = other.wallpaper.clone();
        }
        if other.wallpaper_padding.is_some() {
            self.wallpaper_padding = other.wallpaper_padding;
        }
        if other.bg.is_some() {
            self.bg = other.bg.clone();
        }
        if other.natural.is_some() {
            self.natural = other.natural;
        }
        if other.end_pause.is_some() {
            self.end_pause = other.end_pause.clone();
        }
        if other.start_pause.is_some() {
            self.start_pause = other.start_pause.clone();
        }
        if other.idle_pause.is_some() {
            self.idle_pause = other.idle_pause.clone();
        }
        if other.output.is_some() {
            self.output = other.output.clone();
        }
        if other.fps.is_some() {
            self.fps = other.fps;
        }
    }

    /// Apply CLI arguments on top of config settings
    /// CLI args always win over config values
    pub fn apply_cli_args(&mut self, args: &ArgMatches) {
        // Flags - only override if explicitly set on CLI
        if args.get_flag("verbose") {
            self.verbose = Some(true);
        }
        if args.get_flag("quiet") {
            self.quiet = Some(true);
        }
        if args.get_flag("video") {
            self.video = Some(true);
        }
        if args.get_flag("video-only") {
            self.video_only = Some(true);
        }
        if args.get_flag("natural-mode") {
            self.natural = Some(true);
        }

        // Values - only override if provided on CLI (not default values)
        if args.value_source("decor") == Some(clap::parser::ValueSource::CommandLine) {
            if let Some(v) = args.get_one::<String>("decor") {
                self.decor = Some(v.clone());
            }
        }
        if let Some(v) = args.get_one::<String>("wallpaper") {
            self.wallpaper = Some(v.clone());
        }
        if args.value_source("wallpaper-padding") == Some(clap::parser::ValueSource::CommandLine) {
            if let Some(v) = args.get_one::<u32>("wallpaper-padding") {
                self.wallpaper_padding = Some(*v);
            }
        }
        if args.value_source("bg") == Some(clap::parser::ValueSource::CommandLine) {
            if let Some(v) = args.get_one::<String>("bg") {
                self.bg = Some(v.clone());
            }
        }
        if let Some(v) = args.get_one::<String>("end-pause") {
            self.end_pause = Some(v.clone());
        }
        if let Some(v) = args.get_one::<String>("start-pause") {
            self.start_pause = Some(v.clone());
        }
        if args.value_source("idle-pause") == Some(clap::parser::ValueSource::CommandLine) {
            if let Some(v) = args.get_one::<String>("idle-pause") {
                self.idle_pause = Some(v.clone());
            }
        }
        if args.value_source("file") == Some(clap::parser::ValueSource::CommandLine) {
            if let Some(v) = args.get_one::<String>("file") {
                self.output = Some(v.clone());
            }
        }
        if args.value_source("fps") == Some(clap::parser::ValueSource::CommandLine) {
            if let Some(v) = args.get_one::<u8>("fps") {
                self.fps = Some(*v);
            }
        }
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
        self.decor.as_deref().unwrap_or("none")
    }
    pub fn bg(&self) -> &str {
        self.bg.as_deref().unwrap_or("transparent")
    }
    pub fn wallpaper_padding(&self) -> u32 {
        self.wallpaper_padding.unwrap_or(60)
    }
    pub fn idle_pause(&self) -> &str {
        self.idle_pause.as_deref().unwrap_or("3s")
    }
    pub fn output(&self) -> &str {
        self.output.as_deref().unwrap_or("t-rec")
    }
    /// Get fps value (default: 4, must be kept in sync with CLI default)
    pub fn fps(&self) -> u8 {
        self.fps.unwrap_or(4)
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
        assert_eq!(settings.decor(), "none");
        assert_eq!(settings.bg(), "transparent");
        assert_eq!(settings.wallpaper_padding(), 60);
        assert_eq!(settings.idle_pause(), "3s");
        assert_eq!(settings.output(), "t-rec");
        assert_eq!(settings.fps(), 4);
    }
}
