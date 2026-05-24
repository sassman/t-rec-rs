use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use super::ProfileSettings;

/// Raw config file structure (matches TOML layout)
#[derive(Debug, Deserialize, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub default: ProfileSettings,
    #[serde(default)]
    pub profiles: HashMap<String, ProfileSettings>,
}

/// Find the config file path
pub fn find_config_file() -> Option<PathBuf> {
    // 1. Project-local
    let local = PathBuf::from("t-rec.toml");
    if local.exists() {
        return Some(local);
    }

    // 2. XDG config directory
    if let Some(config_dir) = dirs::config_dir() {
        let xdg_path = config_dir.join("t-rec").join("config.toml");
        if xdg_path.exists() {
            return Some(xdg_path);
        }
    }

    None
}

/// Load and parse the config file
pub fn load_config() -> Result<Option<ConfigFile>> {
    match find_config_file() {
        Some(path) => {
            let content = fs::read_to_string(&path)
                .with_context(|| format!("Failed to read config file: {}", path.display()))?;
            let config: ConfigFile = toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
            Ok(Some(config))
        }
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_config() {
        let config: ConfigFile = toml::from_str("").unwrap();
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn test_parse_config_with_default() {
        let toml = r#"
[default]
wallpaper = "ventura"
wallpaper-padding = 80
"#;
        let config: ConfigFile = toml::from_str(toml).unwrap();
        assert_eq!(config.default.wallpaper, Some("ventura".to_string()));
        assert_eq!(config.default.wallpaper_padding, Some(80));
    }

    #[test]
    fn test_parse_config_with_profiles() {
        let toml = r#"
[default]
quiet = true

[profiles.demo]
wallpaper = "ventura"
wallpaper-padding = 100

[profiles.quick]
idle-pause = "1s"
"#;
        let config: ConfigFile = toml::from_str(toml).unwrap();
        assert_eq!(config.default.quiet, Some(true));
        assert_eq!(config.profiles.len(), 2);
        assert!(config.profiles.contains_key("demo"));
        assert!(config.profiles.contains_key("quick"));
        assert_eq!(
            config.profiles.get("demo").unwrap().wallpaper_padding,
            Some(100)
        );
    }
}
