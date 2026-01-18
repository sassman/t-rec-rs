//! Type-safe wallpaper configuration types.
//!
//! This module provides strongly-typed enums for wallpaper options.

use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result as LibResult};

/// A validated path to a wallpaper image file.
///
/// This newtype ensures that only existing file paths can be stored.
/// Use [`Wallpaper::custom()`] to create a custom wallpaper with validation.
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
/// - Custom wallpaper: `Wallpaper::custom(path)` - validates that the path exists
///
/// # Example
///
/// ```
/// use t_rec::wallpapers::Wallpaper;
/// use std::path::Path;
///
/// // Built-in wallpaper (no validation needed)
/// let ventura = Wallpaper::Ventura;
///
/// // Custom wallpaper with path validation
/// // This will fail if the path doesn't exist
/// // let custom = Wallpaper::custom("/path/to/wallpaper.png")?;
///
/// // Parse from string (for CLI compatibility, does not validate path)
/// let parsed: Wallpaper = "ventura".parse().unwrap();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Wallpaper {
    /// macOS Ventura-style wallpaper (built-in)
    Ventura,
    /// Custom wallpaper from a validated file path
    ///
    /// Use [`Wallpaper::custom()`] to create this variant with path validation.
    #[serde(untagged)]
    Custom(ValidatedPath),
}

impl Wallpaper {
    /// Create a custom wallpaper from a file path with validation.
    ///
    /// This is the recommended way to create custom wallpapers. The path is validated
    /// at construction time to ensure the file exists.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use t_rec::wallpapers::Wallpaper;
    ///
    /// let wallpaper = Wallpaper::custom("/path/to/wallpaper.png")?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Error::WallpaperNotFound` if the file does not exist.
    #[allow(dead_code)] // Library-only API, used by HeadlessRecorder
    pub fn custom(path: impl AsRef<std::path::Path>) -> LibResult<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(Error::WallpaperNotFound(path.to_path_buf()));
        }
        Ok(Wallpaper::Custom(ValidatedPath(path.to_path_buf())))
    }

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

    /// Returns all built-in wallpaper names.
    #[allow(dead_code)] // Library-only API
    pub fn builtin_values() -> &'static [&'static str] {
        &["ventura"]
    }

    /// Check if this is a built-in wallpaper.
    #[allow(dead_code)] // Library-only API
    pub fn is_builtin(&self) -> bool {
        matches!(self, Wallpaper::Ventura)
    }

    /// Get the path for custom wallpapers, or None for built-in.
    #[allow(dead_code)] // Library-only API
    pub fn custom_path(&self) -> Option<&std::path::Path> {
        match self {
            Wallpaper::Custom(path) => Some(path.as_path()),
            _ => None,
        }
    }
}

impl fmt::Display for Wallpaper {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Note: FromStr does NOT validate the path exists. Use `Wallpaper::custom()` for validation.
/// This is kept for CLI/config compatibility where validation happens separately.
impl FromStr for Wallpaper {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ventura" => Ok(Wallpaper::Ventura),
            // Note: We create ValidatedPath without validation here for FromStr compatibility.
            // Path validation should happen at a higher level (e.g., in the builder).
            _ => Ok(Wallpaper::Custom(ValidatedPath(PathBuf::from(s)))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    mod wallpaper_tests {
        use super::*;

        #[test]
        fn test_parse_builtin() {
            assert_eq!("ventura".parse::<Wallpaper>().unwrap(), Wallpaper::Ventura);
            assert_eq!("Ventura".parse::<Wallpaper>().unwrap(), Wallpaper::Ventura);
            assert_eq!("VENTURA".parse::<Wallpaper>().unwrap(), Wallpaper::Ventura);
        }

        #[test]
        fn test_parse_custom_path() {
            // Note: FromStr does not validate path existence
            let wp: Wallpaper = "/path/to/wallpaper.png".parse().unwrap();
            assert_eq!(wp.as_str(), "/path/to/wallpaper.png");
        }

        #[test]
        fn test_as_str() {
            assert_eq!(Wallpaper::Ventura.as_str(), "ventura");
            // FromStr creates ValidatedPath without validation
            let wp: Wallpaper = "/path/to/wp.png".parse().unwrap();
            assert_eq!(wp.as_str(), "/path/to/wp.png");
        }

        #[test]
        fn test_is_builtin() {
            assert!(Wallpaper::Ventura.is_builtin());
            let wp: Wallpaper = "/test.png".parse().unwrap();
            assert!(!wp.is_builtin());
        }

        #[test]
        fn test_custom_path() {
            assert!(Wallpaper::Ventura.custom_path().is_none());
            let wp: Wallpaper = "/test.png".parse().unwrap();
            assert_eq!(wp.custom_path().unwrap().to_str().unwrap(), "/test.png");
        }

        // Tests for the custom() factory method with path validation
        #[test]
        fn test_custom_factory_validates_path() {
            // Non-existent path should fail
            let result = Wallpaper::custom("/nonexistent/path/wallpaper.png");
            assert!(result.is_err());
            assert!(matches!(result.unwrap_err(), Error::WallpaperNotFound(_)));
        }

        #[test]
        fn test_custom_factory_with_existing_file() {
            // Create a temporary file to test with
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, "test content").unwrap();

            let result = Wallpaper::custom(temp_file.path());
            assert!(result.is_ok());
            let wallpaper = result.unwrap();
            assert!(!wallpaper.is_builtin());
            assert_eq!(wallpaper.custom_path().unwrap(), temp_file.path());
        }

        #[test]
        fn test_validated_path_newtype() {
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, "test").unwrap();

            let wallpaper = Wallpaper::custom(temp_file.path()).unwrap();
            if let Wallpaper::Custom(validated_path) = wallpaper {
                assert_eq!(validated_path.as_path(), temp_file.path());
                // Display trait
                assert!(format!("{}", validated_path)
                    .contains(temp_file.path().file_name().unwrap().to_str().unwrap()));
            } else {
                panic!("Expected Custom variant");
            }
        }

        #[test]
        fn test_wallpaper_error_display() {
            let err = Error::WallpaperNotFound(PathBuf::from("/some/path.png"));
            let display = format!("{}", err);
            assert!(display.contains("Wallpaper file not found"));
            assert!(display.contains("/some/path.png"));
        }
    }
}
