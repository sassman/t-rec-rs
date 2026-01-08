# Core Animation Examples

Examples demonstrating the `core-animation` crate's builder APIs.

## Running Examples

```bash
cargo run -p core-animation --example <name>
```

## Examples

### `window_builder`

Basic window creation with the `WindowBuilder` API.

![window_builder](screenshots/window_builder.png)

```bash
cargo run -p core-animation --example window_builder
```

---

### `emitter`

Particle emitter using `CAEmitterLayerBuilder` with the closure-based particle configuration.

![emitter](screenshots/emitter.png)

```bash
cargo run -p core-animation --example emitter
```

---

### `particle_images`

Showcases all `ParticleImage` types side by side:
- `soft_glow` - Radial gradient (top-left)
- `circle` - Solid circle (top-right)
- `star` - Multi-pointed star (bottom-left)
- `spark` - Elongated streak (bottom-right)

![particle_images](screenshots/particle_images.png)

```bash
cargo run -p core-animation --example particle_images
```

---

### `point_burst`

Demonstrates `PointBurstBuilder` - a convenience API for the common pattern of particles bursting from a point in all directions.

![point_burst](screenshots/point_burst.png)

```bash
cargo run -p core-animation --example point_burst
```

---

### `emitter_simple`

Direct port of Apple's CAEmitterLayer documentation example. Shows the fundamental concept: ONE emitter spawns MANY particles.

![emitter_simple](screenshots/emitter_simple.png)

```bash
cargo run -p core-animation --example emitter_simple
```

---

### `sierpinski_particles`

Advanced example: Sierpinski triangle fractal using particle emitters. Particles stream from vertices and manifest into the final fractal shape.

![sierpinski_particles](screenshots/sierpinski_particles.png)

```bash
cargo run -p core-animation --example sierpinski_particles
```

---

## Generating Screenshots

Screenshots can be automatically captured by running examples with the `screenshot` feature:

```bash
cargo run -p core-animation --example emitter --features screenshot
```

This captures a screenshot after a short delay and saves it to the `screenshots/` directory.
