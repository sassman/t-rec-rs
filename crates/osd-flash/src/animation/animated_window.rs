//! Animated window wrapper for OSD windows.
//!
//! This module provides the bridge between static OSD windows and animations.
//! After drawing content with `.draw()`, you can either display it statically
//! or add animations.

use std::time::Duration;

use crate::icon::Icon;
use crate::window::OsdWindow;

use super::builder::AnimationBuilder;

/// Wrapper that holds a window and its drawable content.
///
/// This is returned by `OsdWindow::draw()` and provides methods for
/// both static display and animation.
///
/// # Example
///
/// ```ignore
/// // Static display
/// window.draw(icon).show_for_seconds(5.0)?;
///
/// // With animation
/// window.draw(icon)
///     .animate("pulse", 2.seconds())
///     .keyframe(0.0, |k| k.scale(0.95))
///     .keyframe(1.0, |k| k.scale(1.0))
///     .show_for_seconds(10.0)?;
/// ```
pub struct AnimatedWindow<W: OsdWindow> {
    pub(crate) window: W,
    pub(crate) content: Icon,
}

impl<W: OsdWindow> AnimatedWindow<W> {
    /// Create a new animated window wrapper.
    pub fn new(window: W, content: Icon) -> Self {
        Self { window, content }
    }

    /// Get a reference to the underlying window.
    pub fn window(&self) -> &W {
        &self.window
    }

    /// Get a reference to the content.
    pub fn content(&self) -> &Icon {
        &self.content
    }

    /// Add more content to the icon.
    ///
    /// This allows chaining multiple draw calls:
    /// ```ignore
    /// window.draw(shape1).draw(shape2).show_for_seconds(3.0)?;
    /// ```
    pub fn draw(mut self, content: impl Into<Icon>) -> Self {
        let additional: Icon = content.into();
        self.content.shapes.extend(additional.shapes);
        self
    }

    /// Display the icon statically (no animation) for the specified duration.
    ///
    /// This draws the icon once and keeps the window visible for the given time.
    ///
    /// # Arguments
    ///
    /// * `seconds` - Duration to display in seconds
    ///
    /// # Example
    ///
    /// ```ignore
    /// window.draw(icon).show_for_seconds(3.0)?;
    /// ```
    pub fn show_for_seconds(self, seconds: f64) -> crate::Result<()> {
        // Draw the content and show for duration
        self.window
            .draw_and_show(self.content, seconds)
    }

    /// Start building an animation.
    ///
    /// The animation will cycle with the specified duration and repeat
    /// until the display duration ends.
    ///
    /// # Arguments
    ///
    /// * `name` - A name for the animation (for debugging/identification)
    /// * `duration` - The duration of one animation cycle
    ///
    /// # Example
    ///
    /// ```ignore
    /// window.draw(icon)
    ///     .animate("pulse", 2.seconds())
    ///     .keyframe(0.0, |k| k.scale(0.95))
    ///     .keyframe(0.5, |k| k.scale(1.05))
    ///     .keyframe(1.0, |k| k.scale(0.95))
    ///     .show_for_seconds(10.0)?;
    /// ```
    pub fn animate(self, name: &str, duration: Duration) -> AnimationBuilder<W> {
        AnimationBuilder::new(self, name.to_string(), duration)
    }
}

#[cfg(test)]
mod tests {
    // Tests require a mock OsdWindow implementation
    // Will be tested via integration tests
}
