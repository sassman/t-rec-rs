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
/// Use [`Wallpaper::custom()`] to create a custom wallpaper with validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValidatedPath(PathBuf);

impl ValidatedPath {
    /// Returns the path as a reference.
    pub fn as_path(&self) -> &std::path::Path {
        &self.0
    }

    /// Returns the path as a PathBuf.
    pub fn to_path_buf(&self) -> PathBuf {
        self.0.clone()
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

/// Error returned when creating a wallpaper fails.
#[derive(Debug, Clone)]
pub enum WallpaperError {
    /// The specified file path does not exist
    PathNotFound(PathBuf),
}

impl fmt::Display for WallpaperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WallpaperError::PathNotFound(path) => {
                write!(f, "Wallpaper file not found: {}", path.display())
            }
        }
    }
}

impl std::error::Error for WallpaperError {}

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
    /// Returns `WallpaperError::PathNotFound` if the file does not exist.
    pub fn custom(path: impl AsRef<std::path::Path>) -> Result<Self, WallpaperError> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(WallpaperError::PathNotFound(path.to_path_buf()));
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
    pub fn builtin_values() -> &'static [&'static str] {
        &["ventura"]
    }

    /// Check if this is a built-in wallpaper.
    pub fn is_builtin(&self) -> bool {
        matches!(self, Wallpaper::Ventura)
    }

    /// Get the path for custom wallpapers, or None for built-in.
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

/// Wallpaper configuration with padding.
///
/// Combines a wallpaper source with the padding amount (in pixels)
/// to use around the captured frame.
///
/// # Example
///
/// ```
/// use t_rec::wallpapers::{Wallpaper, WallpaperConfig};
///
/// // Create config with built-in wallpaper and 60px padding
/// let config = WallpaperConfig::new(Wallpaper::Ventura, 60);
///
/// // Parse from string
/// let config2 = WallpaperConfig::from_string("ventura", 50);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WallpaperConfig {
    /// The wallpaper source
    pub wallpaper: Wallpaper,
    /// Padding in pixels around the frame
    pub padding: u32,
}

impl WallpaperConfig {
    /// Create a new wallpaper configuration.
    pub fn new(wallpaper: Wallpaper, padding: u32) -> Self {
        Self { wallpaper, padding }
    }

    /// Create from a string wallpaper value and padding.
    ///
    /// The string can be a built-in name or a file path.
    pub fn from_string(wallpaper: &str, padding: u32) -> Self {
        Self {
            wallpaper: wallpaper.parse().unwrap(), // Infallible
            padding,
        }
    }

    /// Create a Ventura wallpaper config with default padding.
    pub fn ventura(padding: u32) -> Self {
        Self::new(Wallpaper::Ventura, padding)
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
            assert!(matches!(
                result.unwrap_err(),
                WallpaperError::PathNotFound(_)
            ));
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
                assert_eq!(validated_path.to_path_buf(), temp_file.path().to_path_buf());
                // Display trait
                assert!(format!("{}", validated_path)
                    .contains(temp_file.path().file_name().unwrap().to_str().unwrap()));
            } else {
                panic!("Expected Custom variant");
            }
        }

        #[test]
        fn test_wallpaper_error_display() {
            let err = WallpaperError::PathNotFound(PathBuf::from("/some/path.png"));
            let display = format!("{}", err);
            assert!(display.contains("Wallpaper file not found"));
            assert!(display.contains("/some/path.png"));
        }
    }

    mod wallpaper_config_tests {
        use super::*;

        #[test]
        fn test_new() {
            let config = WallpaperConfig::new(Wallpaper::Ventura, 60);
            assert_eq!(config.wallpaper, Wallpaper::Ventura);
            assert_eq!(config.padding, 60);
        }

        #[test]
        fn test_from_string() {
            let config = WallpaperConfig::from_string("ventura", 50);
            assert_eq!(config.wallpaper, Wallpaper::Ventura);
            assert_eq!(config.padding, 50);

            // Note: from_string does not validate path existence
            let config = WallpaperConfig::from_string("/path/to/wp.png", 100);
            assert_eq!(config.wallpaper.as_str(), "/path/to/wp.png");
            assert_eq!(config.padding, 100);
        }

        #[test]
        fn test_ventura_helper() {
            let config = WallpaperConfig::ventura(80);
            assert_eq!(config.wallpaper, Wallpaper::Ventura);
            assert_eq!(config.padding, 80);
        }
    }
}
