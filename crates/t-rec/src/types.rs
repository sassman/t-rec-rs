//! Type-safe configuration types for t-rec.
//!
//! This module provides strongly-typed enums for configuration options that were
//! previously represented as strings. These types provide:
//!
//! - Compile-time validation of valid options
//! - Clear error messages when parsing from strings
//! - Exhaustive pattern matching in code
//! - Type safety throughout the codebase
//!
//! # Design Philosophy
//!
//! - **Enums are the canonical internal representation**
//! - **Validation happens at construction time** via factory methods
//! - **Only valid invariants are possible** - no escape hatches
//! - **Strings are accepted at API boundaries** (CLI args, config files) via `FromStr`
//!
//! # Example
//!
//! ```
//! use t_rec::types::{Decor, BackgroundColor};
//!
//! // Use type-safe variants directly
//! let decor = Decor::Shadow;
//! let bg = BackgroundColor::White;
//!
//! // For wallpapers, see the wallpapers module:
//! // use t_rec::wallpapers::Wallpaper;
//!
//! // Use factory methods for custom values (with validation)
//! let custom_bg = BackgroundColor::custom("#ff0000").unwrap();
//!
//! // Parse from strings (for CLI/config compatibility)
//! let decor: Decor = "shadow".parse().unwrap();
//! let bg: BackgroundColor = "#ff0000".parse().unwrap();
//!
//! // Convert back to strings for ImageMagick commands
//! assert_eq!(decor.as_str(), "shadow");
//! assert_eq!(bg.to_imagemagick_color(), "#ff0000");
//! ```

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Decoration style for frames.
///
/// Controls visual effects applied to captured frames during post-processing.
///
/// # Available Options
///
/// - `None` - No decoration, raw frame output
/// - `Shadow` - Adds a drop shadow effect around the frame
///
/// # Example
///
/// ```
/// use t_rec::types::Decor;
///
/// let decor: Decor = "shadow".parse().unwrap();
/// assert_eq!(decor, Decor::Shadow);
///
/// // Default is no decoration
/// assert_eq!(Decor::default(), Decor::None);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Decor {
    /// No decoration - raw frame output
    #[default]
    None,
    /// Drop shadow effect around the frame
    Shadow,
}

impl Decor {
    /// Returns the string representation for CLI/config usage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Decor::None => "none",
            Decor::Shadow => "shadow",
        }
    }

    /// Returns all available decor options.
    pub fn all() -> &'static [Decor] {
        &[Decor::None, Decor::Shadow]
    }

    /// Returns all valid string values for help text.
    pub fn valid_values() -> &'static [&'static str] {
        &["none", "shadow"]
    }
}

impl fmt::Display for Decor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Decor {
    type Err = DecorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Decor::None),
            "shadow" => Ok(Decor::Shadow),
            _ => Err(DecorParseError(s.to_string())),
        }
    }
}

/// Error returned when parsing an invalid decor value.
#[derive(Debug, Clone)]
pub struct DecorParseError(pub String);

impl fmt::Display for DecorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid decor '{}'. Valid options: {}",
            self.0,
            Decor::valid_values().join(", ")
        )
    }
}

impl std::error::Error for DecorParseError {}

/// A validated hex color string.
///
/// This newtype ensures that only valid hex color formats can be stored.
/// Valid formats: #RGB, #RGBA, #RRGGBB, #RRGGBBAA
///
/// Use [`BackgroundColor::custom()`] to create a custom color with validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HexColor(String);

impl HexColor {
    /// Returns the hex color string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HexColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Background color for post-processing effects.
///
/// Used primarily for the shadow effect background. Supports:
/// - Transparent (default)
/// - Predefined colors (white, black)
/// - Custom hex colors (#RRGGBB or #RRGGBBAA format)
///
/// # Example
///
/// ```
/// use t_rec::types::BackgroundColor;
///
/// // Use predefined colors directly
/// let bg = BackgroundColor::Transparent;
/// let white = BackgroundColor::White;
///
/// // Use factory method for custom hex colors (with validation)
/// let custom = BackgroundColor::custom("#ff5500").unwrap();
/// let with_alpha = BackgroundColor::custom("#ff550080").unwrap();
///
/// // Parse from strings (for CLI/config compatibility)
/// let parsed: BackgroundColor = "#ff5500".parse().unwrap();
///
/// // Default is transparent
/// assert_eq!(BackgroundColor::default(), BackgroundColor::Transparent);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackgroundColor {
    /// Fully transparent background (default)
    #[default]
    Transparent,
    /// White background (#ffffff)
    White,
    /// Black background (#000000)
    Black,
    /// Custom hex color (#RRGGBB or #RRGGBBAA)
    ///
    /// Use [`BackgroundColor::custom()`] to create this variant with validation.
    #[serde(untagged)]
    Custom(HexColor),
}

impl BackgroundColor {
    /// Create a background color from a string value.
    ///
    /// This is the recommended factory method for creating background colors from
    /// user input (CLI, config files). It accepts both named colors and hex values,
    /// validating at construction time.
    ///
    /// # Accepted Values
    ///
    /// - Named colors: "transparent", "none", "white", "black"
    /// - Hex colors: "#RGB", "#RGBA", "#RRGGBB", "#RRGGBBAA"
    ///
    /// # Example
    ///
    /// ```
    /// use t_rec::types::BackgroundColor;
    ///
    /// // Named colors
    /// let transparent = BackgroundColor::custom("transparent")?;
    /// let white = BackgroundColor::custom("white")?;
    /// let black = BackgroundColor::custom("black")?;
    ///
    /// // Hex colors
    /// let red = BackgroundColor::custom("#ff0000")?;
    /// let semi_transparent = BackgroundColor::custom("#ff000080")?;
    ///
    /// // Invalid formats return an error
    /// assert!(BackgroundColor::custom("notacolor").is_err());
    /// assert!(BackgroundColor::custom("#gg0000").is_err());
    /// # Ok::<(), t_rec::types::BackgroundColorParseError>(())
    /// ```
    pub fn custom(value: &str) -> Result<Self, BackgroundColorParseError> {
        // Try named colors first (case-insensitive)
        match value.to_lowercase().as_str() {
            "transparent" | "none" => return Ok(BackgroundColor::Transparent),
            "white" => return Ok(BackgroundColor::White),
            "black" => return Ok(BackgroundColor::Black),
            _ => {}
        }

        // Try hex color
        if value.starts_with('#') {
            Self::validate_hex(value)?;
            return Ok(BackgroundColor::Custom(HexColor(value.to_string())));
        }

        // Unknown color
        Err(BackgroundColorParseError::Unknown(value.to_string()))
    }

    /// Validates a hex color string format.
    fn validate_hex(hex: &str) -> Result<(), BackgroundColorParseError> {
        // Validate hex format: #RGB, #RGBA, #RRGGBB, or #RRGGBBAA
        if !hex.starts_with('#') {
            return Err(BackgroundColorParseError::InvalidFormat(hex.to_string()));
        }

        let hex_digits = &hex[1..];
        let valid_lengths = [3, 4, 6, 8]; // #RGB, #RGBA, #RRGGBB, #RRGGBBAA

        if !valid_lengths.contains(&hex_digits.len()) {
            return Err(BackgroundColorParseError::InvalidFormat(hex.to_string()));
        }

        // Check all characters are valid hex digits
        if !hex_digits.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(BackgroundColorParseError::InvalidFormat(hex.to_string()));
        }

        Ok(())
    }

    /// Returns the color string suitable for ImageMagick commands.
    ///
    /// ImageMagick accepts various color formats; this returns a format
    /// that works reliably across ImageMagick versions.
    pub fn to_imagemagick_color(&self) -> &str {
        match self {
            BackgroundColor::Transparent => "transparent",
            BackgroundColor::White => "white",
            BackgroundColor::Black => "black",
            BackgroundColor::Custom(hex) => hex.as_str(),
        }
    }

    /// Returns the canonical string representation.
    ///
    /// For predefined colors, returns the name (e.g., "transparent").
    /// For custom colors, returns the hex value.
    pub fn as_str(&self) -> &str {
        self.to_imagemagick_color()
    }

    /// Returns all predefined color options (excludes custom).
    pub fn predefined_values() -> &'static [&'static str] {
        &["transparent", "white", "black"]
    }

    /// Check if this is a transparent background.
    pub fn is_transparent(&self) -> bool {
        matches!(self, BackgroundColor::Transparent)
    }
}

impl fmt::Display for BackgroundColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for BackgroundColor {
    type Err = BackgroundColorParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BackgroundColor::custom(s)
    }
}

/// Error returned when parsing an invalid background color.
#[derive(Debug, Clone)]
pub enum BackgroundColorParseError {
    /// Unknown color name
    Unknown(String),
    /// Invalid hex format
    InvalidFormat(String),
}

impl fmt::Display for BackgroundColorParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackgroundColorParseError::Unknown(s) => {
                write!(
                    f,
                    "Unknown background color '{}'. Use a predefined color ({}) or a hex value (#RRGGBB or #RRGGBBAA)",
                    s,
                    BackgroundColor::predefined_values().join(", ")
                )
            }
            BackgroundColorParseError::InvalidFormat(s) => {
                write!(
                    f,
                    "Invalid hex color format '{}'. Expected #RGB, #RGBA, #RRGGBB, or #RRGGBBAA",
                    s
                )
            }
        }
    }
}

impl std::error::Error for BackgroundColorParseError {}

#[cfg(test)]
mod tests {
    use super::*;

    mod decor_tests {
        use super::*;

        #[test]
        fn test_parse_valid_decor() {
            assert_eq!("none".parse::<Decor>().unwrap(), Decor::None);
            assert_eq!("shadow".parse::<Decor>().unwrap(), Decor::Shadow);
        }

        #[test]
        fn test_parse_case_insensitive() {
            assert_eq!("None".parse::<Decor>().unwrap(), Decor::None);
            assert_eq!("SHADOW".parse::<Decor>().unwrap(), Decor::Shadow);
            assert_eq!("Shadow".parse::<Decor>().unwrap(), Decor::Shadow);
        }

        #[test]
        fn test_parse_invalid_decor() {
            let err = "invalid".parse::<Decor>().unwrap_err();
            assert!(err.to_string().contains("Invalid decor"));
            assert!(err.to_string().contains("none"));
            assert!(err.to_string().contains("shadow"));
        }

        #[test]
        fn test_as_str() {
            assert_eq!(Decor::None.as_str(), "none");
            assert_eq!(Decor::Shadow.as_str(), "shadow");
        }

        #[test]
        fn test_display() {
            assert_eq!(format!("{}", Decor::None), "none");
            assert_eq!(format!("{}", Decor::Shadow), "shadow");
        }

        #[test]
        fn test_default() {
            assert_eq!(Decor::default(), Decor::None);
        }
    }

    mod background_color_tests {
        use super::*;

        #[test]
        fn test_parse_predefined_colors() {
            assert_eq!(
                "transparent".parse::<BackgroundColor>().unwrap(),
                BackgroundColor::Transparent
            );
            assert_eq!(
                "white".parse::<BackgroundColor>().unwrap(),
                BackgroundColor::White
            );
            assert_eq!(
                "black".parse::<BackgroundColor>().unwrap(),
                BackgroundColor::Black
            );
        }

        #[test]
        fn test_parse_none_as_transparent() {
            assert_eq!(
                "none".parse::<BackgroundColor>().unwrap(),
                BackgroundColor::Transparent
            );
        }

        #[test]
        fn test_parse_case_insensitive() {
            assert_eq!(
                "TRANSPARENT".parse::<BackgroundColor>().unwrap(),
                BackgroundColor::Transparent
            );
            assert_eq!(
                "White".parse::<BackgroundColor>().unwrap(),
                BackgroundColor::White
            );
        }

        #[test]
        fn test_parse_hex_colors() {
            // 6-digit hex
            let color = "#ff5500".parse::<BackgroundColor>().unwrap();
            assert_eq!(color.as_str(), "#ff5500");

            // 8-digit hex with alpha
            let color = "#ff550080".parse::<BackgroundColor>().unwrap();
            assert_eq!(color.as_str(), "#ff550080");

            // 3-digit shorthand
            let color = "#f50".parse::<BackgroundColor>().unwrap();
            assert_eq!(color.as_str(), "#f50");

            // 4-digit shorthand with alpha
            let color = "#f508".parse::<BackgroundColor>().unwrap();
            assert_eq!(color.as_str(), "#f508");
        }

        #[test]
        fn test_parse_invalid_hex() {
            // Missing #
            let err = "ff5500".parse::<BackgroundColor>().unwrap_err();
            assert!(matches!(err, BackgroundColorParseError::Unknown(_)));

            // Invalid length
            let err = "#ff".parse::<BackgroundColor>().unwrap_err();
            assert!(matches!(err, BackgroundColorParseError::InvalidFormat(_)));

            // Invalid characters
            let err = "#gggggg".parse::<BackgroundColor>().unwrap_err();
            assert!(matches!(err, BackgroundColorParseError::InvalidFormat(_)));
        }

        #[test]
        fn test_to_imagemagick_color() {
            assert_eq!(
                BackgroundColor::Transparent.to_imagemagick_color(),
                "transparent"
            );
            assert_eq!(BackgroundColor::White.to_imagemagick_color(), "white");
            assert_eq!(BackgroundColor::Black.to_imagemagick_color(), "black");
            assert_eq!(
                BackgroundColor::custom("#ff5500")
                    .unwrap()
                    .to_imagemagick_color(),
                "#ff5500"
            );
        }

        #[test]
        fn test_is_transparent() {
            assert!(BackgroundColor::Transparent.is_transparent());
            assert!(!BackgroundColor::White.is_transparent());
            assert!(!BackgroundColor::custom("#000").unwrap().is_transparent());
        }

        #[test]
        fn test_default() {
            assert_eq!(BackgroundColor::default(), BackgroundColor::Transparent);
        }

        // Tests for the custom() factory method
        #[test]
        fn test_custom_factory_with_named_colors() {
            // Test that custom() accepts named colors
            assert_eq!(
                BackgroundColor::custom("transparent").unwrap(),
                BackgroundColor::Transparent
            );
            assert_eq!(
                BackgroundColor::custom("white").unwrap(),
                BackgroundColor::White
            );
            assert_eq!(
                BackgroundColor::custom("black").unwrap(),
                BackgroundColor::Black
            );
            // "none" is an alias for transparent
            assert_eq!(
                BackgroundColor::custom("none").unwrap(),
                BackgroundColor::Transparent
            );
        }

        #[test]
        fn test_custom_factory_case_insensitive() {
            assert_eq!(
                BackgroundColor::custom("TRANSPARENT").unwrap(),
                BackgroundColor::Transparent
            );
            assert_eq!(
                BackgroundColor::custom("White").unwrap(),
                BackgroundColor::White
            );
            assert_eq!(
                BackgroundColor::custom("BLACK").unwrap(),
                BackgroundColor::Black
            );
        }

        #[test]
        fn test_custom_factory_with_hex_colors() {
            let red = BackgroundColor::custom("#ff0000").unwrap();
            assert_eq!(red.as_str(), "#ff0000");

            let with_alpha = BackgroundColor::custom("#ff000080").unwrap();
            assert_eq!(with_alpha.as_str(), "#ff000080");

            let shorthand = BackgroundColor::custom("#f00").unwrap();
            assert_eq!(shorthand.as_str(), "#f00");
        }

        #[test]
        fn test_custom_factory_invalid_colors() {
            // Unknown named color
            assert!(BackgroundColor::custom("notacolor").is_err());

            // Missing # prefix for hex
            assert!(BackgroundColor::custom("ff0000").is_err());

            // Invalid hex characters
            assert!(BackgroundColor::custom("#gg0000").is_err());

            // Invalid hex length
            assert!(BackgroundColor::custom("#ff").is_err());
        }

        #[test]
        fn test_hex_color_newtype() {
            // Ensure HexColor wraps and displays correctly
            let color = BackgroundColor::custom("#abcdef").unwrap();
            if let BackgroundColor::Custom(hex) = color {
                assert_eq!(hex.as_str(), "#abcdef");
                assert_eq!(format!("{}", hex), "#abcdef");
            } else {
                panic!("Expected Custom variant");
            }
        }
    }
}
