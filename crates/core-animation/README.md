# core-animation

Rust bindings for macOS Core Animation with builder APIs.

Core Animation is Apple's GPU-accelerated rendering system. This crate wraps it with ergonomic builders, focusing on **particle effects** and **layer composition**.

## Quick Start

```rust
use std::f64::consts::PI;
use core_animation::prelude::*;

let emitter = CAEmitterLayerBuilder::new()
    .position(320.0, 240.0)
    .shape(EmitterShape::Point)
    .particle(|p| {
        p.birth_rate(100.0)           // spawn rate
            .lifetime(5.0)            // seconds until particle disappears
            .velocity(80.0)           // movement speed
            .emission_range(PI * 2.0) // spread angle (full circle)
            .color(Color::CYAN)
            .image(ParticleImage::soft_glow(64))
    })
    .build();

window.container().add_sublayer(&emitter);
window.show_for(10.seconds());
```

Simpler API for point bursts:

```rust
let burst = PointBurstBuilder::new(320.0, 240.0)
    .velocity(100.0)
    .color(Color::PINK)
    .build();
```

## Examples

```bash
cargo run -p core-animation --example emitter
```

| Example | Description |
|---------|-------------|
| `emitter` | Particle emitter with closure-based configuration |
| `point_burst` | Simplified API for radial particle bursts |
| `particle_images` | All particle image types side by side |
| `window_builder` | Basic window creation |
| `sierpinski_particles` | Fractal using particle system |

See [examples/README.md](examples/README.md) for screenshots.

## Modules

- **`particles`** - Particle emitter builders
  - `CAEmitterLayerBuilder` - Full emitter configuration
  - `PointBurstBuilder` - Convenience builder for radial bursts
  - `ParticleImage` - Pre-built particle images (soft_glow, circle, star, spark)
  - `EmitterShape`, `EmitterMode`, `RenderMode` - Configuration enums

- **`window`** - Test window for examples (not for production)

## Types

**This crate:**
- `Color` - RGBA with presets (`Color::CYAN`, `Color::rgb(...)`)
- `CALayerBuilder` - Generic layer builder
- `CAShapeLayerBuilder` - Vector shape builder
- `DurationExt` - `5.seconds()`, `500.millis()` syntax

**Re-exported from Apple frameworks:**
- `CALayer`, `CAShapeLayer`, `CATransform3D` - Core Animation
- `CGPoint`, `CGSize`, `CGRect` - Geometry
- `CGPath`, `CGColor` - Graphics

Use `prelude` to import common types:

```rust
use core_animation::prelude::*;
```

## Platform

macOS only. Requires `objc2` ecosystem.

## License

Same as parent crate.
