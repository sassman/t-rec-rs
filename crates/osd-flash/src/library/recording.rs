//! Recording indicator composition.
//!
//! A pulsing red dot indicator commonly used to show recording status.

use std::time::Duration;

use crate::color::Color;
use crate::composition::{Animation, CompositionBuilder, LayerComposition};
use crate::DurationExt;

/// A recording indicator with pulsing animation.
///
/// Displays a pulsing red dot with glow effect, commonly used to indicate
/// that recording is in progress.
///
/// # Examples
///
/// ```ignore
/// use osd_flash::prelude::*;
///
/// // Default recording indicator
/// OsdBuilder::new()
///     .composition(RecordingIndicator::new())
///     .show_for(10.seconds())?;
///
/// // Customized indicator
/// OsdBuilder::new()
///     .composition(
///         RecordingIndicator::new()
///             .size(100.0)
///             .color(Color::GREEN)
///             .pulse_duration(1200u64.millis())
///     )
///     .show_for(10.seconds())?;
/// ```
#[derive(Debug, Clone)]
pub struct RecordingIndicator {
    size: f64,
    dot_color: Color,
    glow_color: Color,
    highlight_color: Color,
    pulse_duration: Duration,
}

impl RecordingIndicator {
    /// Create a new recording indicator with default settings.
    ///
    /// Defaults:
    /// - Size: 80x80
    /// - Color: Red (#F22626)
    /// - Pulse duration: 800ms
    pub fn new() -> Self {
        let dot_color = Color::rgb(0.95, 0.15, 0.15);
        Self {
            size: 80.0,
            dot_color,
            glow_color: dot_color.with_alpha(0.35),
            highlight_color: Color::rgba(1.0, 0.5, 0.5, 0.5),
            pulse_duration: 800u64.millis(),
        }
    }

    /// Set the size of the indicator (width and height).
    ///
    /// The dot and glow sizes scale proportionally.
    pub fn size(mut self, size: f64) -> Self {
        self.size = size;
        self
    }

    /// Set the main dot color.
    ///
    /// The glow color is automatically derived with reduced opacity.
    pub fn color(mut self, color: Color) -> Self {
        self.dot_color = color;
        self.glow_color = color.with_alpha(0.35);
        self
    }

    /// Set the glow color explicitly.
    ///
    /// Use this to override the automatic glow color derivation.
    pub fn glow_color(mut self, color: Color) -> Self {
        self.glow_color = color;
        self
    }

    /// Set the highlight color (the small bright spot).
    pub fn highlight_color(mut self, color: Color) -> Self {
        self.highlight_color = color;
        self
    }

    /// Set the pulse animation duration.
    ///
    /// This is the time for one complete pulse cycle.
    pub fn pulse_duration(mut self, duration: Duration) -> Self {
        self.pulse_duration = duration;
        self
    }
}

impl Default for RecordingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl From<RecordingIndicator> for LayerComposition {
    fn from(ri: RecordingIndicator) -> Self {
        // Calculate proportional sizes
        let glow_diameter = ri.size * 0.55;
        let dot_diameter = ri.size * 0.35;
        let highlight_diameter = ri.size * 0.1;
        let highlight_offset = dot_diameter * 0.18;

        CompositionBuilder::new(ri.size)
            // Glow layer (pulses larger)
            .layer("glow", |l| {
                l.circle(glow_diameter)
                    .center()
                    .fill(ri.glow_color)
                    .animate(Animation::pulse_range(0.9, 1.15))
            })
            // Main recording dot
            .layer("dot", |l| {
                l.circle(dot_diameter)
                    .center()
                    .fill(ri.dot_color)
                    .animate(Animation::Pulse {
                        min_scale: 0.9,
                        max_scale: 1.1,
                        duration: ri.pulse_duration,
                        easing: crate::composition::Easing::InOut,
                    })
            })
            // Highlight (small bright spot)
            .layer("highlight", |l| {
                l.circle(highlight_diameter)
                    .center_offset(-highlight_offset, highlight_offset)
                    .fill(ri.highlight_color)
            })
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let ri = RecordingIndicator::new();
        assert_eq!(ri.size, 80.0);
    }

    #[test]
    fn test_size() {
        let ri = RecordingIndicator::new().size(100.0);
        assert_eq!(ri.size, 100.0);
    }

    #[test]
    fn test_color() {
        let ri = RecordingIndicator::new().color(Color::GREEN);
        assert_eq!(ri.dot_color, Color::GREEN);
        assert_eq!(ri.glow_color.a, 0.35);
    }

    #[test]
    fn test_into_composition() {
        let comp: LayerComposition = RecordingIndicator::new().into();
        assert_eq!(comp.layers.len(), 3);
        assert_eq!(comp.layers[0].name, "glow");
        assert_eq!(comp.layers[1].name, "dot");
        assert_eq!(comp.layers[2].name, "highlight");
    }
}
