//! Keyframe-based animation system for OSD indicators.
//!
//! Provides CSS-inspired declarative animations with interpolated transitions.
//!
//! # Example
//!
//! ```ignore
//! use osd_flash::prelude::*;
//!
//! let icon = IconBuilder::new(80.0)
//!     .circle(40.0, 40.0, 12.0, Color::RED)
//!     .build();
//!
//! OsdFlashBuilder::new()
//!     .dimensions(80.0)
//!     .background(Color::rgba(0.1, 0.1, 0.1, 0.85))
//!     .corner_radius(14.0)
//!     .build()?
//!     .draw(icon)
//!     .animate("pulse", 2.seconds())
//!     .keyframe(0.0, |k| k.scale(0.95))
//!     .keyframe(0.5, |k| k.scale(1.05))
//!     .keyframe(1.0, |k| k.scale(0.95))
//!     .show(10.seconds())?;
//! ```
//!
//! # API Chain
//!
//! ```text
//! OsdFlashBuilder::new().build()?   -> impl OsdWindow
//!     .draw(icon)                   -> AnimatedWindow
//!     .animate(name, duration)      -> AnimationBuilder
//!     .keyframe(progress, |k| ...)  -> AnimationBuilder
//!     .show(duration)?              -> Result<()>
//! ```
//!
//! # Keyframes
//!
//! Define animation state at progress points (0.0 to 1.0):
//!
//! ```ignore
//! .keyframe(0.0, |k| k.scale(0.95))  // start
//! .keyframe(0.5, |k| k.scale(1.05))  // midpoint
//! .keyframe(1.0, |k| k.scale(0.95))  // end (loops to start)
//! ```
//!
//! Values between keyframes are linearly interpolated with easing applied.
//!
//! # Transforms
//!
//! Applied to icon content (not window background):
//!
//! - `scale(f64)` - 1.0 = original, 0.5 = half, 2.0 = double
//!
//! # Overlay Shapes
//!
//! Keyframes can add animated shapes:
//!
//! ```ignore
//! .keyframe(0.0, |k| k.circle(40.0, 40.0, 20.0, Color::rgba(1.0, 0.0, 0.0, 0.3)))
//! .keyframe(1.0, |k| k.circle(40.0, 40.0, 30.0, Color::rgba(1.0, 0.0, 0.0, 0.0)))
//! ```
//!
//! Shape position, size, and color are interpolated.
//!
//! # Easing
//!
//! Controls transition timing:
//!
//! - `Linear` - constant speed
//! - `EaseIn` - slow start
//! - `EaseOut` - slow end
//! - `EaseInOut` - slow start and end (default)
//! - `CubicBezier(x1, y1, x2, y2)` - custom curve
//!
//! Set per-animation or per-keyframe:
//!
//! ```ignore
//! .animate("pulse", 2.seconds()).easing(Easing::Linear)
//! .keyframe(0.5, |k| k.scale(1.0).easing(Easing::EaseOut))
//! ```
//!
//! # Interpolation
//!
//! Shapes matched by index and type:
//!
//! ```text
//! Keyframe A: [circle₁, circle₂]
//! Keyframe B: [circle₃, circle₄, rect₅]
//!
//! Index 0: circle₁ <-> circle₃  (interpolate)
//! Index 1: circle₂ <-> circle₄  (interpolate)
//! Index 2: none <-> rect₅       (skip)
//! ```
//!
//! # Architecture
//!
//! ```text
//! AnimationBuilder -> Animation config
//!                          |
//!                          v
//! AnimationRunner (60fps) -> interpolate keyframes -> render frame
//!                                                          |
//!                                                          v
//!                                            clear -> background -> transform
//!                                                -> icon -> overlays -> flush
//! ```
//!
//! # Limitations
//!
//! - No double-buffering (may flicker)
//! - macOS only (SkyLight + CFRunLoop)
//! - Scale transform only (no rotate/translate yet)

pub mod animated_window;
pub mod builder;
pub mod easing;
pub mod interpolation;
pub mod keyframe;
pub mod runner;
pub mod transform;

pub use animated_window::AnimatedWindow;
pub use builder::{Animation, AnimationBuilder, Repeat};
pub use easing::Easing;
pub use interpolation::{interpolate, interpolate_color, lerp, InterpolatedFrame};
pub use keyframe::{Keyframe, KeyframeBuilder};
pub use transform::Transform;
