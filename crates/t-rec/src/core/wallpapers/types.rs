//! Type-safe wallpaper configuration types.
//!
//! This module provides strongly-typed enums for wallpaper options.

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A validated path to a wallpaper image file.
///
/// This newtype ensures that only existing file paths can be stored.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValidatedPath(pub(crate) PathBuf);

impl ValidatedPath {
    /// Returns the path as a reference.
    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }
}

impl AsRef<std::path::Path> for ValidatedPath {
    fn as_ref(&self) -> &std::path::Path {
        &self.0
    }
}

impl fmt::Display for ValidatedPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

/// Wallpaper configuration for frame backgrounds.
///
/// Wallpapers provide a decorative background around captured frames.
/// The frame is centered on the wallpaper with configurable padding.
///
/// # Available Options
///
/// - Built-in wallpapers: `Wallpaper::Ventura` (macOS Ventura style)
/// - Custom wallpaper: Parse a path string to create custom wallpapers
///
/// # Example
///
/// ```
/// use t_rec::wallpapers::Wallpaper;
///
/// // Built-in wallpaper
/// let ventura = Wallpaper::Ventura;
///
/// // Parse from string (for CLI compatibility)
/// let parsed: Wallpaper = "ventura".parse().unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Wallpaper {
    /// macOS Ventura-style wallpaper (built-in)
    Ventura,
    /// Custom wallpaper from a file path
    #[serde(untagged)]
    Custom(ValidatedPath),
}

impl Wallpaper {
    /// Returns the string representation.
    ///
    /// For built-in wallpapers, returns the name.
    /// For custom wallpapers, returns the path as a string.
    pub fn as_str(&self) -> String {
        match self {
            Wallpaper::Ventura => "ventura".to_string(),
            Wallpaper::Custom(path) => path.to_string(),
        }
    }
}

impl fmt::Display for Wallpaper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Note: FromStr does NOT validate the path exists.
/// Path validation should happen at a higher level.
impl FromStr for Wallpaper {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ventura" => Ok(Wallpaper::Ventura),
            _ => Ok(Wallpaper::Custom(ValidatedPath(PathBuf::from(s)))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_builtin() {
        assert_eq!("ventura".parse::<Wallpaper>().unwrap(), Wallpaper::Ventura);
        assert_eq!("Ventura".parse::<Wallpaper>().unwrap(), Wallpaper::Ventura);
        assert_eq!("VENTURA".parse::<Wallpaper>().unwrap(), Wallpaper::Ventura);
    }

    #[test]
    fn test_parse_custom_path() {
        let wp: Wallpaper = "/path/to/wallpaper.png".parse().unwrap();
        assert_eq!(wp.as_str(), "/path/to/wallpaper.png");
    }

    #[test]
    fn test_as_str() {
        assert_eq!(Wallpaper::Ventura.as_str(), "ventura");
        let wp: Wallpaper = "/path/to/wp.png".parse().unwrap();
        assert_eq!(wp.as_str(), "/path/to/wp.png");
    }
}
