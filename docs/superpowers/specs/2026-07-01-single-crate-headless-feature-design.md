# Simplify crate layout: default-install binary, opt-in `headless` lib

## Goal

Remove the current install friction (`cargo install t-rec --features cli`) and give library integrators a clean, minimal-dependency entry point.

After this change:

- `cargo install t-rec` installs the binary with no extra flags.
- Third-party binaries can depend on `t-rec` as a library through a `headless` feature and pull in a smaller dependency tree — no `clap`, `crossterm`, `dialoguer`, `env_logger`, `humantime`, `toml`, or `dirs`.
- The layout stays a single crate at `crates/t-rec/`. No file moves.

## Non-goals

- Splitting `t-rec` into a separate `t-rec-lib` + `t-rec` two-crate workspace. Considered and rejected — single-crate covers the use case with less churn.
- Reducing lib-core dependencies below the current always-on set. `tokio` (sync only) and `serde` remain required by `core::` internals.
- Publishing `HeadlessRecorder` as a stable public API. It is exposed under the `headless` feature and remains preview-quality.

## Current state (baseline)

Single crate `t-rec` at `crates/t-rec/` with both `src/lib.rs` and `src/main.rs`.

Feature layout today (`crates/t-rec/Cargo.toml`):

```toml
[features]
default = []
e2e_tests = []
cli = []

[[bin]]
name = "t-rec"
required-features = ["cli"]
```

Consequences of this shape:

1. `cargo install t-rec` fails: default features do not include `cli`, so `required-features` is unsatisfied.
2. `src/lib.rs` uses `#[cfg(feature = "cli")]` and `#[cfg(not(feature = "cli"))]` on module declarations and re-exports to swap which items are exposed. The `HeadlessRecorder` API is only visible when `cli` is off, and helpers used by the binary are only visible when `cli` is on. Public lib surface depends on build flavor.
3. `[package.metadata.docs.rs]` sets `features = ["lib"]` — a feature that does not exist.

## Design

### Feature layout

```toml
[features]
default = ["bin"]
bin = [
    "dep:clap",
    "dep:crossterm",
    "dep:dialoguer",
    "dep:env_logger",
    "dep:humantime",
    "dep:toml",
    "dep:dirs",
]
headless = []   # gates the HeadlessRecorder module; no extra deps
e2e_tests = []
```

Rationale:

- `default = ["bin"]` — `cargo install t-rec` picks up default features → `bin` on → binary compiles. No user-visible flags.
- `bin` bundles the CLI-only dependencies. Users never type this name; it lives inside Cargo.toml for readability. Called `bin` (not `cli`) to signal "the deps needed to build the binary target".
- `headless` gates the `HeadlessRecorder` module in `src/api/`. It carries no extra deps — its purpose is to expose lib-facing public API only when the consumer asks for it.
- The `cli` feature is removed.

### Dependency partitioning

Always-on (lib-core):

- `anyhow`, `log`, `tempfile`, `rayon`, `simplerand`, `image`
- `serde` — used in `core::types` and `core::wallpapers::types` derives.
- `tokio` (features = `["sync"]` only) — used by `core::event_router` and `core::capture` for `broadcast` channels.

Bin-only (marked `optional = true`, enabled via `bin` feature):

- `clap`, `crossterm`, `dialoguer`, `env_logger`, `humantime`, `toml`, `dirs`

Platform-specific deps (`objc2-*`, `nix`, `libc`, `x11rb`, `win-screenshot`, `windows`) stay under their current `[target.'cfg(...)'.dependencies]` blocks and remain always-on. They live inside `core::` and are needed by both the binary and library-mode integrators.

### Binary target

```toml
[[bin]]
name = "t-rec"
required-features = ["bin"]
```

`required-features` stays as a safety net. Since `default = ["bin"]`, it is satisfied for the common install path. If a user runs `cargo build --no-default-features` the binary target is skipped rather than failing to link.

### Consumer entry points

| User | Cargo invocation | Result |
|---|---|---|
| Installing the CLI | `cargo install t-rec` | Default features → `bin` on → binary builds. |
| Building the CLI locally | `cargo build --release` | Same as above. |
| Third-party binary embedding t-rec as a lib | `t-rec = { version = "0.9", default-features = false, features = ["headless"] }` | `HeadlessRecorder` API available. `clap`, `crossterm`, `dialoguer`, `env_logger`, `humantime`, `toml`, `dirs` not pulled in. |
| Someone doing `cargo add t-rec` casually | `t-rec = "0.9"` | Default features on — same fat tree as an install. They can opt out. |

### Code changes

**`src/lib.rs`.** Replace the current `#[cfg(feature = "cli")]` / `#[cfg(not(feature = "cli"))]` gymnastics with a single clean gate:

```rust
pub mod core;

#[cfg(feature = "headless")]
mod api;

#[cfg(feature = "headless")]
pub use api::{HeadlessRecorder, HeadlessRecorderBuilder, HeadlessRecorderConfig, RecordingOutput};

// Always-on public surface — used by the binary, available to any lib consumer.
pub use core::{
    Image, ImageOnHeap, Margin, PlatformApi, Result, WindowId, WindowList, WindowListEntry,
};
pub use core::error;
pub use core::types;
pub use core::types::{BackgroundColor, Decor};
pub use core::wallpapers;
pub use core::wallpapers::{load_and_validate_wallpaper, resolve_wallpaper, Wallpaper};
pub use core::common::{Platform, PlatformApiFactory};
pub use core::event_router::{CaptureEvent, Event, EventRouter, FlashEvent, LifecycleEvent};
#[cfg(target_os = "linux")]
pub use core::linux::DEFAULT_SHELL;
#[cfg(target_os = "macos")]
pub use core::macos::DEFAULT_SHELL;
pub use core::post_processing::post_process_screenshots;
pub use core::screenshot::{screenshot_file_name, screenshot_output_name, ScreenshotInfo};
```

The current re-exports guarded by `#[cfg(feature = "cli")]` become unconditional. They are used by the binary through `use t_rec::…`, and exposing them from the lib is harmless — third parties who want lower-level control can use them too.

**`src/main.rs` and `src/cli/*`.** Drop any `#[cfg(feature = "cli")]` guards. The binary only compiles when `required-features = ["bin"]` is met, so per-item guards are redundant.

**`[package.metadata.docs.rs]`.** Change `features = ["lib"]` → `features = ["headless"]` so docs.rs renders the `HeadlessRecorder` API.

**README.** Remove `--features cli` from the install section.

**`build.rs`.** No change expected; it links X11 unconditionally on the relevant targets and is not tied to features.

### Verification steps

Executed during implementation, before claiming done:

1. `cargo build --release` — bin builds with default features.
2. `cargo build --release --no-default-features` — lib-only build succeeds; no `bin` target output.
3. `cargo build --release --no-default-features --features headless` — lib with `HeadlessRecorder` exposed compiles.
4. `cargo tree --no-default-features --features headless` — confirm `clap`, `crossterm`, `dialoguer`, `env_logger`, `humantime`, `toml`, `dirs` are absent.
5. `cargo install --path crates/t-rec` — binary installs without extra flags.
6. `cargo test` — existing tests pass.
7. `cargo clippy --locked --all-targets -- -D warnings` — no warnings.
8. `cargo clippy --no-default-features --features headless -- -D warnings` — no warnings in lib-only shape.

### Breaking-change surface

- Anyone currently doing `cargo add t-rec` with default features and no additional flags will now pull the CLI dep tree. Acceptable because the crate is `0.9.0-preview3` — no stable API promise yet.
- Anyone currently doing `cargo install t-rec --features cli` continues to work (the `cli` feature is gone, but `--features cli` on an unknown feature currently errors; users who followed the README will need to drop the flag). Acceptable and desired — that flag is exactly the friction being removed. Callout in CHANGELOG.

### Out of scope

- Any renaming of the crate.
- Any workspace-level restructuring.
- Reducing the always-on dep set below `{anyhow, log, tempfile, rayon, simplerand, image, serde, tokio(sync)}`.
- Refactors to `core::` module boundaries.
