//! Particle emitter builders.
//!
//! Particles are small images spawned continuously from an emitter. Each particle
//! has properties like velocity, lifetime, and color. The GPU handles rendering,
//! so thousands of particles run smoothly.
//!
//! ```ignore
//! use std::f64::consts::PI;
//! use core_animation::prelude::*;
//!
//! let emitter = CAEmitterLayerBuilder::new()
//!     .position(320.0, 320.0)
//!     .shape(EmitterShape::Point)
//!     .particle(|p| p
//!         .birth_rate(100.0)
//!         .lifetime(5.0)
//!         .velocity(80.0)
//!         .emission_range(PI * 2.0)  // all directions
//!         .color(Color::CYAN)
//!         .image(ParticleImage::soft_glow(64))
//!     )
//!     .build();
//! ```
//!
//! For simple point bursts, use [`PointBurstBuilder`] instead.

use crate::color::Color;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_core_foundation::{CFRetained, CGPoint, CGRect, CGSize};
use objc2_core_graphics::{
    CGBitmapContextCreate, CGBitmapContextCreateImage, CGColor, CGColorSpace, CGContext, CGImage,
    CGImageAlphaInfo,
};
use objc2_foundation::NSArray;
use objc2_quartz_core::{
    kCAEmitterLayerAdditive, kCAEmitterLayerBackToFront, kCAEmitterLayerCircle,
    kCAEmitterLayerCuboid, kCAEmitterLayerLine, kCAEmitterLayerOldestFirst,
    kCAEmitterLayerOldestLast, kCAEmitterLayerOutline, kCAEmitterLayerPoint, kCAEmitterLayerPoints,
    kCAEmitterLayerRectangle, kCAEmitterLayerSphere, kCAEmitterLayerSurface,
    kCAEmitterLayerUnordered, kCAEmitterLayerVolume, CAEmitterCell, CAEmitterLayer,
};

// ============================================================================
// Enums
// ============================================================================

/// Shape of the emitter - where particles spawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmitterShape {
    /// Particles spawn from a single point.
    #[default]
    Point,
    /// Particles spawn along a line.
    Line,
    /// Particles spawn within a rectangle.
    Rectangle,
    /// Particles spawn on/in a circle (2D sphere).
    Circle,
    /// Particles spawn on/in a cuboid (3D box).
    Cuboid,
    /// Particles spawn on/in a sphere.
    Sphere,
}

/// Mode determining where on the shape particles spawn.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmitterMode {
    /// Particles spawn at discrete points on the shape.
    #[default]
    Points,
    /// Particles spawn on the outline/edge of the shape.
    Outline,
    /// Particles spawn on the surface of the shape.
    Surface,
    /// Particles spawn throughout the volume of the shape.
    Volume,
}

/// Render mode determining how particles are composited.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RenderMode {
    /// Particles rendered in no particular order.
    #[default]
    Unordered,
    /// Oldest particles rendered first (behind newer ones).
    OldestFirst,
    /// Oldest particles rendered last (in front of newer ones).
    OldestLast,
    /// Particles sorted back-to-front by depth.
    BackToFront,
    /// Particles use additive blending (colors add together).
    Additive,
}

/// Pre-built particle images.
#[derive(Clone, Debug)]
pub enum ParticleImage {
    /// Radial gradient - white center fading to transparent.
    SoftGlow(u32),
    /// Solid filled circle.
    Circle(u32),
    /// Star shape with specified number of points.
    Star { size: u32, points: u32 },
    /// Elongated spark/streak shape.
    Spark(u32),
}

impl ParticleImage {
    /// Create a soft glow particle image (radial gradient).
    pub fn soft_glow(size: u32) -> Self {
        Self::SoftGlow(size)
    }

    /// Create a solid circle particle image.
    pub fn circle(size: u32) -> Self {
        Self::Circle(size)
    }

    /// Create a star particle image with the specified number of points.
    ///
    /// Common values: 4, 5, 6, or 8 points.
    pub fn star(size: u32, points: u32) -> Self {
        Self::Star { size, points }
    }

    /// Create an elongated spark/streak particle image.
    ///
    /// Good for motion trails, fire sparks, or shooting stars.
    pub fn spark(size: u32) -> Self {
        Self::Spark(size)
    }

    /// Generate the CGImage for this particle.
    pub fn to_cgimage(&self) -> CFRetained<CGImage> {
        match self {
            Self::SoftGlow(size) => create_soft_glow_image(*size as usize),
            Self::Circle(size) => create_circle_image(*size as usize),
            Self::Star { size, points } => create_star_image(*size as usize, *points as usize),
            Self::Spark(size) => create_spark_image(*size as usize),
        }
    }
}

// ============================================================================
// CAEmitterCellBuilder
// ============================================================================

/// Builder for a particle type.
///
/// Used via `CAEmitterLayerBuilder::particle(|p| p.birth_rate(...))`.
pub struct CAEmitterCellBuilder {
    birth_rate: f32,
    lifetime: f32,
    lifetime_range: f32,
    velocity: f64,
    velocity_range: f64,
    emission_longitude: f64,
    emission_range: f64,
    scale: f64,
    scale_range: f64,
    scale_speed: f64,
    alpha_speed: f32,
    spin: f64,
    spin_range: f64,
    acceleration: (f64, f64),
    color: Option<Color>,
    image: Option<ParticleImage>,
}

impl CAEmitterCellBuilder {
    /// Create a new cell builder with default values.
    pub fn new() -> Self {
        Self {
            birth_rate: 1.0,
            lifetime: 1.0,
            lifetime_range: 0.0,
            velocity: 0.0,
            velocity_range: 0.0,
            emission_longitude: 0.0,
            emission_range: 0.0,
            scale: 1.0,
            scale_range: 0.0,
            scale_speed: 0.0,
            alpha_speed: 0.0,
            spin: 0.0,
            spin_range: 0.0,
            acceleration: (0.0, 0.0),
            color: None,
            image: None,
        }
    }

    /// Set the number of particles spawned per second.
    pub fn birth_rate(mut self, rate: f32) -> Self {
        self.birth_rate = rate;
        self
    }

    /// Set how long each particle lives (in seconds).
    pub fn lifetime(mut self, seconds: f32) -> Self {
        self.lifetime = seconds;
        self
    }

    /// Set random variation in lifetime (+/- seconds).
    pub fn lifetime_range(mut self, range: f32) -> Self {
        self.lifetime_range = range;
        self
    }

    /// Set initial velocity (points per second).
    pub fn velocity(mut self, v: f64) -> Self {
        self.velocity = v;
        self
    }

    /// Set random variation in velocity.
    pub fn velocity_range(mut self, range: f64) -> Self {
        self.velocity_range = range;
        self
    }

    /// Set the direction of emission (radians, 0 = right, PI/2 = up).
    pub fn emission_longitude(mut self, radians: f64) -> Self {
        self.emission_longitude = radians;
        self
    }

    /// Set emission direction to point toward a target position.
    ///
    /// The `from` parameter is the emitter position.
    pub fn emission_toward(mut self, from: (f64, f64), target: (f64, f64)) -> Self {
        let dx = target.0 - from.0;
        let dy = target.1 - from.1;
        self.emission_longitude = dy.atan2(dx);
        self
    }

    /// Set the spread of emission (radians).
    ///
    /// Use `PI * 2.0` for emission in all directions (360°).
    /// Use `PI / 6.0` for a 30° spread.
    pub fn emission_range(mut self, radians: f64) -> Self {
        self.emission_range = radians;
        self
    }

    /// Set the scale of particles (1.0 = original size).
    pub fn scale(mut self, s: f64) -> Self {
        self.scale = s;
        self
    }

    /// Set random variation in scale.
    pub fn scale_range(mut self, range: f64) -> Self {
        self.scale_range = range;
        self
    }

    /// Set rate of scale change per second.
    pub fn scale_speed(mut self, speed: f64) -> Self {
        self.scale_speed = speed;
        self
    }

    /// Set rate of alpha change per second (negative = fade out).
    pub fn alpha_speed(mut self, speed: f32) -> Self {
        self.alpha_speed = speed;
        self
    }

    /// Set rotation speed (radians per second).
    pub fn spin(mut self, radians_per_sec: f64) -> Self {
        self.spin = radians_per_sec;
        self
    }

    /// Set random variation in spin.
    pub fn spin_range(mut self, range: f64) -> Self {
        self.spin_range = range;
        self
    }

    /// Set acceleration (points per second squared).
    ///
    /// Use negative y for gravity effect.
    pub fn acceleration(mut self, x: f64, y: f64) -> Self {
        self.acceleration = (x, y);
        self
    }

    /// Set particle color using a `Color` value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// .color(Color::RED)
    /// .color(Color::rgb(0.3, 0.8, 1.0))
    /// .color(Color::WHITE.with_alpha(0.5))
    /// ```
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set particle color (RGB, 0.0-1.0).
    pub fn color_rgb(mut self, r: f64, g: f64, b: f64) -> Self {
        self.color = Some(Color::rgb(r, g, b));
        self
    }

    /// Set particle color (RGBA, 0.0-1.0).
    pub fn color_rgba(mut self, r: f64, g: f64, b: f64, a: f64) -> Self {
        self.color = Some(Color::rgba(r, g, b, a));
        self
    }

    /// Set the particle image.
    pub fn image(mut self, img: ParticleImage) -> Self {
        self.image = Some(img);
        self
    }

    /// Build the CAEmitterCell.
    pub fn build(self) -> Retained<CAEmitterCell> {
        let cell = CAEmitterCell::new();

        cell.setBirthRate(self.birth_rate);
        cell.setLifetime(self.lifetime);
        cell.setLifetimeRange(self.lifetime_range);
        cell.setVelocity(self.velocity);
        cell.setVelocityRange(self.velocity_range);
        cell.setEmissionLongitude(self.emission_longitude);
        cell.setEmissionRange(self.emission_range);
        cell.setScale(self.scale);
        cell.setScaleRange(self.scale_range);
        cell.setScaleSpeed(self.scale_speed);
        cell.setAlphaSpeed(self.alpha_speed);
        cell.setSpin(self.spin);
        cell.setSpinRange(self.spin_range);
        cell.setXAcceleration(self.acceleration.0);
        cell.setYAcceleration(self.acceleration.1);

        if let Some(color) = self.color {
            let cgcolor: CFRetained<CGColor> = color.into();
            cell.setColor(Some(&cgcolor));
        }

        if let Some(img) = self.image {
            let cgimage = img.to_cgimage();
            unsafe {
                let image_obj: &AnyObject = std::mem::transmute(&*cgimage);
                cell.setContents(Some(image_obj));
            }
        }

        cell
    }
}

impl Default for CAEmitterCellBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CAEmitterLayerBuilder
// ============================================================================

/// Builder for a particle emitter layer.
///
/// ```ignore
/// let emitter = CAEmitterLayerBuilder::new()
///     .position(320.0, 320.0)
///     .shape(EmitterShape::Point)
///     .particle(|p| p
///         .birth_rate(100.0)
///         .lifetime(5.0)
///         .velocity(80.0)
///         .color(Color::CYAN)
///     )
///     .build();
/// ```
pub struct CAEmitterLayerBuilder {
    position: (f64, f64),
    size: (f64, f64),
    shape: EmitterShape,
    mode: EmitterMode,
    render_mode: RenderMode,
    birth_rate: f32,
    cells: Vec<Retained<CAEmitterCell>>,
}

impl CAEmitterLayerBuilder {
    /// Create a new emitter layer builder.
    pub fn new() -> Self {
        Self {
            position: (0.0, 0.0),
            size: (0.0, 0.0),
            shape: EmitterShape::Point,
            mode: EmitterMode::Points,
            render_mode: RenderMode::Unordered,
            birth_rate: 1.0,
            cells: Vec::new(),
        }
    }

    /// Set the emitter position.
    pub fn position(mut self, x: f64, y: f64) -> Self {
        self.position = (x, y);
        self
    }

    /// Set the emitter size (for Line, Rectangle, etc.).
    pub fn size(mut self, width: f64, height: f64) -> Self {
        self.size = (width, height);
        self
    }

    /// Set the emitter shape.
    pub fn shape(mut self, shape: EmitterShape) -> Self {
        self.shape = shape;
        self
    }

    /// Set the emitter mode.
    pub fn mode(mut self, mode: EmitterMode) -> Self {
        self.mode = mode;
        self
    }

    /// Set the render mode.
    pub fn render_mode(mut self, mode: RenderMode) -> Self {
        self.render_mode = mode;
        self
    }

    /// Set the overall birth rate multiplier.
    ///
    /// This multiplies the birth rate of all cells.
    /// Use 0.0 to pause emission, 1.0 for normal rate.
    pub fn birth_rate(mut self, rate: f32) -> Self {
        self.birth_rate = rate;
        self
    }

    /// Add a particle type using a closure to configure it.
    ///
    /// # Example
    ///
    /// ```ignore
    /// .particle(|p| p
    ///     .birth_rate(100.0)
    ///     .lifetime(10.0)
    ///     .velocity(100.0)
    /// )
    /// ```
    pub fn particle<F>(mut self, configure: F) -> Self
    where
        F: FnOnce(CAEmitterCellBuilder) -> CAEmitterCellBuilder,
    {
        let builder = CAEmitterCellBuilder::new();
        let configured = configure(builder);
        self.cells.push(configured.build());
        self
    }

    /// Add a pre-built cell directly.
    ///
    /// Prefer using `.particle()` for most cases.
    pub fn cell(mut self, cell: Retained<CAEmitterCell>) -> Self {
        self.cells.push(cell);
        self
    }

    /// Build the CAEmitterLayer.
    pub fn build(self) -> Retained<CAEmitterLayer> {
        let emitter = CAEmitterLayer::new();

        emitter.setEmitterPosition(CGPoint::new(self.position.0, self.position.1));
        emitter.setEmitterSize(CGSize::new(self.size.0, self.size.1));
        emitter.setBirthRate(self.birth_rate);

        // Set shape
        unsafe {
            let shape = match self.shape {
                EmitterShape::Point => kCAEmitterLayerPoint,
                EmitterShape::Line => kCAEmitterLayerLine,
                EmitterShape::Rectangle => kCAEmitterLayerRectangle,
                EmitterShape::Circle => kCAEmitterLayerCircle,
                EmitterShape::Cuboid => kCAEmitterLayerCuboid,
                EmitterShape::Sphere => kCAEmitterLayerSphere,
            };
            emitter.setEmitterShape(shape);
        }

        // Set mode
        unsafe {
            let mode = match self.mode {
                EmitterMode::Points => kCAEmitterLayerPoints,
                EmitterMode::Outline => kCAEmitterLayerOutline,
                EmitterMode::Surface => kCAEmitterLayerSurface,
                EmitterMode::Volume => kCAEmitterLayerVolume,
            };
            emitter.setEmitterMode(mode);
        }

        // Set render mode
        unsafe {
            let render = match self.render_mode {
                RenderMode::Unordered => kCAEmitterLayerUnordered,
                RenderMode::OldestFirst => kCAEmitterLayerOldestFirst,
                RenderMode::OldestLast => kCAEmitterLayerOldestLast,
                RenderMode::BackToFront => kCAEmitterLayerBackToFront,
                RenderMode::Additive => kCAEmitterLayerAdditive,
            };
            emitter.setRenderMode(render);
        }

        // Set cells
        if !self.cells.is_empty() {
            let cells = NSArray::from_retained_slice(&self.cells);
            emitter.setEmitterCells(Some(&cells));
        }

        emitter
    }
}

impl Default for CAEmitterLayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PointBurstBuilder (convenience builder)
// ============================================================================

/// Simpler builder for particles bursting outward from a point.
///
/// ```ignore
/// let burst = PointBurstBuilder::new(320.0, 320.0)
///     .velocity(100.0)
///     .color(Color::ORANGE)
///     .build();
/// ```
pub struct PointBurstBuilder {
    position: (f64, f64),
    birth_rate: f32,
    lifetime: f32,
    lifetime_range: f32,
    velocity: f64,
    velocity_range: f64,
    scale: f64,
    scale_range: f64,
    scale_speed: f64,
    alpha_speed: f32,
    color: Option<Color>,
    image: Option<ParticleImage>,
    render_mode: RenderMode,
}

impl PointBurstBuilder {
    /// Create a new point burst builder at the specified position.
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            position: (x, y),
            birth_rate: 100.0,
            lifetime: 5.0,
            lifetime_range: 1.0,
            velocity: 100.0,
            velocity_range: 20.0,
            scale: 0.1,
            scale_range: 0.02,
            scale_speed: 0.0,
            alpha_speed: 0.0,
            color: None,
            image: None,
            render_mode: RenderMode::Additive,
        }
    }

    /// Set the number of particles spawned per second.
    pub fn birth_rate(mut self, rate: f32) -> Self {
        self.birth_rate = rate;
        self
    }

    /// Set how long each particle lives (in seconds).
    pub fn lifetime(mut self, seconds: f32) -> Self {
        self.lifetime = seconds;
        self
    }

    /// Set random variation in lifetime.
    pub fn lifetime_range(mut self, range: f32) -> Self {
        self.lifetime_range = range;
        self
    }

    /// Set initial velocity (points per second).
    pub fn velocity(mut self, v: f64) -> Self {
        self.velocity = v;
        self
    }

    /// Set random variation in velocity.
    pub fn velocity_range(mut self, range: f64) -> Self {
        self.velocity_range = range;
        self
    }

    /// Set the scale of particles.
    pub fn scale(mut self, s: f64) -> Self {
        self.scale = s;
        self
    }

    /// Set random variation in scale.
    pub fn scale_range(mut self, range: f64) -> Self {
        self.scale_range = range;
        self
    }

    /// Set rate of scale change per second.
    pub fn scale_speed(mut self, speed: f64) -> Self {
        self.scale_speed = speed;
        self
    }

    /// Set rate of alpha change per second (negative = fade out).
    pub fn alpha_speed(mut self, speed: f32) -> Self {
        self.alpha_speed = speed;
        self
    }

    /// Set particle color using a `Color` value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// .color(Color::CYAN)
    /// .color(Color::rgb(1.0, 0.8, 0.2))
    /// ```
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set particle color (RGB, 0.0-1.0).
    pub fn color_rgb(mut self, r: f64, g: f64, b: f64) -> Self {
        self.color = Some(Color::rgb(r, g, b));
        self
    }

    /// Set particle color (RGBA, 0.0-1.0).
    pub fn color_rgba(mut self, r: f64, g: f64, b: f64, a: f64) -> Self {
        self.color = Some(Color::rgba(r, g, b, a));
        self
    }

    /// Set the particle image. Defaults to soft_glow(64) if not set.
    pub fn image(mut self, img: ParticleImage) -> Self {
        self.image = Some(img);
        self
    }

    /// Set the render mode. Defaults to Additive.
    pub fn render_mode(mut self, mode: RenderMode) -> Self {
        self.render_mode = mode;
        self
    }

    /// Build the CAEmitterLayer configured for point burst.
    pub fn build(self) -> Retained<CAEmitterLayer> {
        use std::f64::consts::PI;

        let image = self.image.unwrap_or_else(|| ParticleImage::soft_glow(64));

        CAEmitterLayerBuilder::new()
            .position(self.position.0, self.position.1)
            .shape(EmitterShape::Point)
            .render_mode(self.render_mode)
            .particle(|p| {
                let mut builder = p
                    .birth_rate(self.birth_rate)
                    .lifetime(self.lifetime)
                    .lifetime_range(self.lifetime_range)
                    .velocity(self.velocity)
                    .velocity_range(self.velocity_range)
                    .emission_range(PI * 2.0) // All directions
                    .scale(self.scale)
                    .scale_range(self.scale_range)
                    .scale_speed(self.scale_speed)
                    .alpha_speed(self.alpha_speed)
                    .image(image);

                if let Some(color) = self.color {
                    builder = builder.color(color);
                }

                builder
            })
            .build()
    }
}

// ============================================================================
// Image creation helpers
// ============================================================================

/// Creates a soft glow particle image (radial gradient, white center fading to transparent).
fn create_soft_glow_image(size: usize) -> CFRetained<CGImage> {
    let color_space = CGColorSpace::new_device_rgb().expect("Failed to create color space");

    let context = unsafe {
        CGBitmapContextCreate(
            std::ptr::null_mut(),
            size,
            size,
            8,
            size * 4,
            Some(&color_space),
            CGImageAlphaInfo::PremultipliedLast.0,
        )
    }
    .expect("Failed to create bitmap context");

    // Draw radial gradient (white center fading to transparent)
    let center = (size / 2) as f64;
    let radius = center;

    for r in (1..=size / 2).rev() {
        let alpha = 1.0 - (r as f64 / radius);
        CGContext::set_rgb_fill_color(Some(&context), 1.0, 1.0, 1.0, alpha);
        CGContext::fill_ellipse_in_rect(
            Some(&context),
            CGRect::new(
                CGPoint::new(center - r as f64, center - r as f64),
                CGSize::new(r as f64 * 2.0, r as f64 * 2.0),
            ),
        );
    }

    CGBitmapContextCreateImage(Some(&context)).expect("Failed to create image")
}

/// Creates a solid circle particle image.
fn create_circle_image(size: usize) -> CFRetained<CGImage> {
    let color_space = CGColorSpace::new_device_rgb().expect("Failed to create color space");

    let context = unsafe {
        CGBitmapContextCreate(
            std::ptr::null_mut(),
            size,
            size,
            8,
            size * 4,
            Some(&color_space),
            CGImageAlphaInfo::PremultipliedLast.0,
        )
    }
    .expect("Failed to create bitmap context");

    // Draw solid white circle
    CGContext::set_rgb_fill_color(Some(&context), 1.0, 1.0, 1.0, 1.0);
    CGContext::fill_ellipse_in_rect(
        Some(&context),
        CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(size as f64, size as f64),
        ),
    );

    CGBitmapContextCreateImage(Some(&context)).expect("Failed to create image")
}

/// Creates a star particle image with the specified number of points.
fn create_star_image(size: usize, points: usize) -> CFRetained<CGImage> {
    use std::f64::consts::PI;

    let color_space = CGColorSpace::new_device_rgb().expect("Failed to create color space");

    let context = unsafe {
        CGBitmapContextCreate(
            std::ptr::null_mut(),
            size,
            size,
            8,
            size * 4,
            Some(&color_space),
            CGImageAlphaInfo::PremultipliedLast.0,
        )
    }
    .expect("Failed to create bitmap context");

    let center = size as f64 / 2.0;
    let outer_radius = center * 0.95;
    let inner_radius = center * 0.4;
    let points = points.max(3); // At least 3 points

    // Draw star by filling triangular segments with gradient
    // Start from top (-PI/2) and go clockwise
    let angle_step = PI / points as f64;

    for i in 0..(points * 2) {
        let is_outer = i % 2 == 0;
        let radius = if is_outer { outer_radius } else { inner_radius };
        let angle = -PI / 2.0 + (i as f64) * angle_step;

        let x = center + radius * angle.cos();
        let y = center + radius * angle.sin();

        // Draw radial line from center with gradient
        let steps = 20;
        for s in 0..steps {
            let t = s as f64 / steps as f64;
            let px = center + (x - center) * t;
            let py = center + (y - center) * t;
            let alpha = 1.0 - t * 0.5; // Fade slightly toward tips

            CGContext::set_rgb_fill_color(Some(&context), 1.0, 1.0, 1.0, alpha);
            let dot_size = 2.0 + (1.0 - t) * 2.0;
            CGContext::fill_ellipse_in_rect(
                Some(&context),
                CGRect::new(
                    CGPoint::new(px - dot_size / 2.0, py - dot_size / 2.0),
                    CGSize::new(dot_size, dot_size),
                ),
            );
        }
    }

    // Draw bright center
    for r in (1..=(size / 6)).rev() {
        let alpha = 1.0 - (r as f64 / (size as f64 / 6.0)) * 0.3;
        CGContext::set_rgb_fill_color(Some(&context), 1.0, 1.0, 1.0, alpha);
        CGContext::fill_ellipse_in_rect(
            Some(&context),
            CGRect::new(
                CGPoint::new(center - r as f64, center - r as f64),
                CGSize::new(r as f64 * 2.0, r as f64 * 2.0),
            ),
        );
    }

    CGBitmapContextCreateImage(Some(&context)).expect("Failed to create image")
}

/// Creates an elongated spark/streak particle image.
fn create_spark_image(size: usize) -> CFRetained<CGImage> {
    let color_space = CGColorSpace::new_device_rgb().expect("Failed to create color space");

    // Spark is wider than tall (elongated horizontally)
    let width = size;
    let height = size / 3;

    let context = unsafe {
        CGBitmapContextCreate(
            std::ptr::null_mut(),
            width,
            height,
            8,
            width * 4,
            Some(&color_space),
            CGImageAlphaInfo::PremultipliedLast.0,
        )
    }
    .expect("Failed to create bitmap context");

    let center_x = width as f64 / 2.0;
    let center_y = height as f64 / 2.0;

    // Draw elongated gradient streak
    for x in 0..width {
        let dx = (x as f64 - center_x) / center_x;
        let x_alpha = 1.0 - dx.abs().powf(1.5); // Fade toward ends

        for y in 0..height {
            let dy = (y as f64 - center_y) / center_y;
            let y_alpha = 1.0 - dy.abs().powf(2.0); // Sharp vertical falloff

            let alpha = (x_alpha * y_alpha).max(0.0);
            if alpha > 0.01 {
                CGContext::set_rgb_fill_color(Some(&context), 1.0, 1.0, 1.0, alpha);
                CGContext::fill_rect(
                    Some(&context),
                    CGRect::new(CGPoint::new(x as f64, y as f64), CGSize::new(1.0, 1.0)),
                );
            }
        }
    }

    CGBitmapContextCreateImage(Some(&context)).expect("Failed to create image")
}
