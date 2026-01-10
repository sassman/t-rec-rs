//! Text styling for rendered text.

use crate::Color;

/// Font weight specification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontWeight {
    /// Thin weight (100).
    Thin,
    /// Extra light weight (200).
    ExtraLight,
    /// Light weight (300).
    Light,
    /// Normal/regular weight (400).
    #[default]
    Regular,
    /// Medium weight (500).
    Medium,
    /// Semi-bold weight (600).
    SemiBold,
    /// Bold weight (700).
    Bold,
    /// Extra bold weight (800).
    ExtraBold,
    /// Black/heavy weight (900).
    Black,
}

impl FontWeight {
    /// Get the numeric weight value (100-900).
    pub const fn value(&self) -> u16 {
        match self {
            Self::Thin => 100,
            Self::ExtraLight => 200,
            Self::Light => 300,
            Self::Regular => 400,
            Self::Medium => 500,
            Self::SemiBold => 600,
            Self::Bold => 700,
            Self::ExtraBold => 800,
            Self::Black => 900,
        }
    }

    /// Create a font weight from a numeric value.
    /// Values are clamped and rounded to the nearest standard weight.
    pub fn from_value(value: u16) -> Self {
        match value {
            0..=149 => Self::Thin,
            150..=249 => Self::ExtraLight,
            250..=349 => Self::Light,
            350..=449 => Self::Regular,
            450..=549 => Self::Medium,
            550..=649 => Self::SemiBold,
            650..=749 => Self::Bold,
            750..=849 => Self::ExtraBold,
            _ => Self::Black,
        }
    }
}

/// Text alignment within its bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlignment {
    /// Align text to the left (start).
    #[default]
    Left,
    /// Center the text.
    Center,
    /// Align text to the right (end).
    Right,
}

/// Style specification for rendering text.
#[derive(Debug, Clone, PartialEq)]
pub struct TextStyle {
    /// Font family name.
    pub font_family: String,
    /// Font size in points.
    pub size: f64,
    /// Text color.
    pub color: Color,
    /// Font weight.
    pub weight: FontWeight,
    /// Text alignment.
    pub alignment: TextAlignment,
    /// Overall opacity (0.0 to 1.0).
    pub opacity: f64,
}

impl TextStyle {
    /// Default font family used when none is specified.
    pub const DEFAULT_FONT: &'static str = "Helvetica Neue";

    /// Create a new text style with the given size and color.
    pub fn new(size: f64, color: Color) -> Self {
        Self {
            font_family: Self::DEFAULT_FONT.to_string(),
            size,
            color,
            weight: FontWeight::Regular,
            alignment: TextAlignment::Left,
            opacity: 1.0,
        }
    }

    /// Create a text style with a specific font family.
    pub fn with_font(size: f64, color: Color, font_family: impl Into<String>) -> Self {
        Self {
            font_family: font_family.into(),
            size,
            color,
            weight: FontWeight::Regular,
            alignment: TextAlignment::Left,
            opacity: 1.0,
        }
    }

    /// Set the font family.
    pub fn font(mut self, family: impl Into<String>) -> Self {
        self.font_family = family.into();
        self
    }

    /// Set the font size.
    pub fn size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }

    /// Set the text color.
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Set the font weight.
    pub fn weight(mut self, weight: FontWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Set bold weight.
    pub fn bold(mut self) -> Self {
        self.weight = FontWeight::Bold;
        self
    }

    /// Set light weight.
    pub fn light(mut self) -> Self {
        self.weight = FontWeight::Light;
        self
    }

    /// Set the text alignment.
    pub fn alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Center align the text.
    pub fn centered(mut self) -> Self {
        self.alignment = TextAlignment::Center;
        self
    }

    /// Right align the text.
    pub fn right_aligned(mut self) -> Self {
        self.alignment = TextAlignment::Right;
        self
    }

    /// Set the opacity.
    pub fn opacity(mut self, opacity: f64) -> Self {
        self.opacity = opacity;
        self
    }

    /// Get the effective color with opacity applied.
    pub fn effective_color(&self) -> Color {
        self.color.with_alpha(self.color.a * self.opacity)
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self::new(14.0, Color::BLACK)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_weight_values() {
        assert_eq!(FontWeight::Thin.value(), 100);
        assert_eq!(FontWeight::Regular.value(), 400);
        assert_eq!(FontWeight::Bold.value(), 700);
        assert_eq!(FontWeight::Black.value(), 900);
    }

    #[test]
    fn test_font_weight_from_value() {
        assert_eq!(FontWeight::from_value(100), FontWeight::Thin);
        assert_eq!(FontWeight::from_value(400), FontWeight::Regular);
        assert_eq!(FontWeight::from_value(750), FontWeight::ExtraBold);
        assert_eq!(FontWeight::from_value(1000), FontWeight::Black);
    }

    #[test]
    fn test_text_style_new() {
        let ts = TextStyle::new(16.0, Color::WHITE);
        assert_eq!(ts.size, 16.0);
        assert_eq!(ts.color, Color::WHITE);
        assert_eq!(ts.weight, FontWeight::Regular);
        assert_eq!(ts.alignment, TextAlignment::Left);
        assert_eq!(ts.font_family, "Helvetica Neue");
    }

    #[test]
    fn test_text_style_builder() {
        let ts = TextStyle::new(12.0, Color::BLACK)
            .font("Arial")
            .bold()
            .centered()
            .opacity(0.8);

        assert_eq!(ts.font_family, "Arial");
        assert_eq!(ts.weight, FontWeight::Bold);
        assert_eq!(ts.alignment, TextAlignment::Center);
        assert_eq!(ts.opacity, 0.8);
    }

    #[test]
    fn test_effective_color() {
        let ts = TextStyle::new(14.0, Color::rgba(1.0, 0.0, 0.0, 0.5)).opacity(0.5);

        let c = ts.effective_color();
        assert!((c.a - 0.25).abs() < f64::EPSILON); // 0.5 * 0.5
    }
}
