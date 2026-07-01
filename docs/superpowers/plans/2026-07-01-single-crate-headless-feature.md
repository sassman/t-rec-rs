# Single-crate `headless` feature — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the `cli` opt-in feature with a `bin` default + opt-in `headless` feature, so that `cargo install t-rec` works with no flags and third-party integrators can depend on `t-rec` as a lib with a slim dep tree.

**Architecture:** Single-crate at `crates/t-rec/`. Collapse the existing dual-mode (`cli` on/off) `core::` code paths to the CLI-mode superset — the not-cli branches were only there because of the current feature model and are logically dead once the model changes. Introduce `headless` as a no-extra-deps feature that gates the public `HeadlessRecorder` API (in `src/api/`). Move CLI-only deps behind an on-by-default `bin` feature so lib users can opt out with `--no-default-features --features headless`.

**Tech Stack:** Rust 2018 edition, Cargo features (`dep:` syntax), single-crate lib+bin.

**Spec:** `docs/superpowers/specs/2026-07-01-single-crate-headless-feature-design.md`

**Working directory (all commands):** `/Users/d34dl0ck/workspaces/terminal-recorder/final/t-rec/.worktrees/fix/cli`

---

## File overview

Modified (no files created, no files deleted):

- `crates/t-rec/Cargo.toml` — features + optional deps + `required-features`.
- `crates/t-rec/src/lib.rs` — replace all `#[cfg(feature = "cli")]` guards with unconditional re-exports; add `#[cfg(feature = "headless")]` gate for `api::HeadlessRecorder`.
- `crates/t-rec/src/main.rs` — drop `#[cfg(feature = "cli")]` guards (there aren't any today, but re-verify).
- `crates/t-rec/src/core/capture.rs` — collapse `cfg(feature="cli")` / `cfg(not(feature="cli"))` dual branches to the cli branch. Delete the not-cli variants.
- `crates/t-rec/src/core/event_router.rs` — remove all `cfg(feature="cli")` guards on `Screenshot`, `Flash`, `Lifecycle`, `try_send`, `shutdown`. `#[cfg(all(test, feature = "cli"))]` → `#[cfg(test)]`.
- `crates/t-rec/src/core/screenshot.rs` — remove all `cfg(feature="cli")` guards. Test-mod guard → `#[cfg(test)]`.
- `crates/t-rec/src/core/post_processing.rs` — remove `cfg(feature="cli")` guard on `ScreenshotInfo` import + related.
- `crates/t-rec/src/core/wallpapers/mod.rs` — remove `cfg(not(feature="cli"))` on `pub use load_and_validate_wallpaper`.
- `crates/t-rec/src/core/wallpapers/validation.rs` — remove `cfg(not(feature="cli"))` on `load_and_validate_wallpaper` fn.
- `crates/t-rec/src/core/macos/mod.rs` — remove `cfg(feature="cli")` on `DEFAULT_SHELL`.
- `crates/t-rec/src/core/windows/mod.rs` — remove `cfg(feature="cli")` on `DEFAULT_SHELL`.
- `crates/t-rec/src/api/headless.rs` — remove residual `cfg(feature="cli")` guard inside a test path.

Optional touch-ups (in the same series):

- `CHANGELOG.md` — note the feature-flag change and slim-lib entry point.
- `crates/t-rec/CHANGELOG.md` — same, if that file is separately maintained.

---

## Task 1: Baseline verification (record current state)

**Files:** none — reproduces the current failure.

- [ ] **Step 1: Confirm install fails today**

Run: `cargo install --path crates/t-rec --locked --dry-run 2>&1 | tail -20`
Expected: an error like `no binaries are available for install`. This is the friction we're removing.

- [ ] **Step 2: Confirm current tests pass**

Run: `cargo test --features cli --locked`
Expected: PASS. Baseline for regression checks.

- [ ] **Step 3: Confirm lib-only build passes today**

Run: `cargo build --no-default-features --locked -p t-rec --lib`
Expected: PASS. This is the current "not cli" path; after refactor it must still pass.

- [ ] **Step 4: No commit** — this task is just a snapshot of the "before" state.

---

## Task 2: Collapse `core::capture.rs` dual branches to the CLI superset

**Files:** Modify `crates/t-rec/src/core/capture.rs`

The `cli`-gated branches are the intended runtime path; the `not(cli)` branches are simplified duplicates. Delete the `not(cli)` alternatives and unconditionalize the `cli` ones. `LifecycleEvent`, `CaptureEvent::Screenshot`, and `ScreenshotInfo` become always-available (Task 3 unlocks them at their declaration sites).

- [ ] **Step 1: Unconditionalize the `log` import at top of file**

At `crates/t-rec/src/core/capture.rs:4-5`, change:

```rust
#[cfg(feature = "cli")]
use log::{debug, error};
```

to:

```rust
use log::{debug, error};
```

- [ ] **Step 2: Unconditionalize the event/screenshot imports**

At `crates/t-rec/src/core/capture.rs:14-20`, change:

```rust
#[cfg(feature = "cli")]
use super::event_router::LifecycleEvent;
use super::event_router::{CaptureEvent, Event};
#[cfg(feature = "cli")]
use super::screenshot::screenshot_file_name;
#[cfg(feature = "cli")]
use super::screenshot::ScreenshotInfo;
```

to:

```rust
use super::event_router::LifecycleEvent;
use super::event_router::{CaptureEvent, Event};
use super::screenshot::screenshot_file_name;
use super::screenshot::ScreenshotInfo;
```

- [ ] **Step 3: Unconditionalize the `screenshots` field on `CaptureContext`**

At `crates/t-rec/src/core/capture.rs:41-43`, remove the `#[cfg(feature = "cli")]` above the field. Result:

```rust
    /// List of captured screenshots.
    pub screenshots: Option<Arc<Mutex<Vec<ScreenshotInfo>>>>,
```

- [ ] **Step 4: Collapse the wait-for-start block (keep the cli path)**

At `crates/t-rec/src/core/capture.rs:63-78`, replace the whole `#[cfg(feature = "cli")] loop { ... } #[cfg(not(feature = "cli"))] match ...` block with the cli variant, uncfg'd:

```rust
    // Wait for Start event before beginning capture
    loop {
        match rx.blocking_recv() {
            Ok(Event::Capture(CaptureEvent::Start)) => break,
            Ok(Event::Capture(CaptureEvent::Stop))
            | Ok(Event::Lifecycle(LifecycleEvent::Shutdown)) => return Ok(()),
            Ok(_) => continue, // Ignore Flash events and Screenshot in wait-for-start phase
            Err(_) => return Ok(()),
        }
    }
```

- [ ] **Step 5: Collapse the `screenshot_event_tc` match**

At `crates/t-rec/src/core/capture.rs:98-119`, replace the dual match with the cli variant only:

```rust
        let screenshot_event_tc = match rx.try_recv() {
            Ok(Event::Capture(CaptureEvent::Stop))
            | Ok(Event::Lifecycle(LifecycleEvent::Shutdown)) => break,
            Ok(Event::Capture(CaptureEvent::Start)) => continue,
            Ok(Event::Capture(CaptureEvent::Screenshot { timecode_ms })) => {
                debug!("Received Screenshot event with timecode {}", timecode_ms);
                Some(timecode_ms)
            }
            Ok(_) => None, // Ignore Flash events
            Err(TryRecvError::Closed) => break,
            Err(TryRecvError::Empty) => None,
            Err(_) => None,
        };
```

- [ ] **Step 6: Collapse the screenshot-handling block**

At `crates/t-rec/src/core/capture.rs:129-141`, replace:

```rust
        // Handle screenshot if triggered by event (CLI only)
        #[cfg(feature = "cli")]
        if let Some(screenshot_tc) = screenshot_event_tc {
            debug!("Taking screenshot at tc={}", screenshot_tc);
            if let Err(e) = save_screenshot(&image, screenshot_tc, &ctx) {
                error!("Failed to save screenshot: {}", e);
            } else {
                debug!("Screenshot saved successfully to tempdir");
            }
        }
        // Suppress unused variable warning for lib builds
        #[cfg(not(feature = "cli"))]
        let _ = screenshot_event_tc;
```

with:

```rust
        // Handle screenshot if triggered by event
        if let Some(screenshot_tc) = screenshot_event_tc {
            debug!("Taking screenshot at tc={}", screenshot_tc);
            if let Err(e) = save_screenshot(&image, screenshot_tc, &ctx) {
                error!("Failed to save screenshot: {}", e);
            } else {
                debug!("Screenshot saved successfully to tempdir");
            }
        }
```

- [ ] **Step 7: Unconditionalize `save_screenshot` and its callers**

At `crates/t-rec/src/core/capture.rs:203-204`, remove `#[cfg(feature = "cli")]` above `fn save_screenshot`. Do the same for any remaining `#[cfg(feature = "cli")]` inside the file (around line 204 and its test block).

- [ ] **Step 8: Fix the test cfg**

At `crates/t-rec/src/core/capture.rs:253`, change `#[cfg(all(test, feature = "cli"))]` → `#[cfg(test)]`.

- [ ] **Step 9: Verify the file has no remaining `cli` guards**

Run: `grep -n 'feature = "cli"' crates/t-rec/src/core/capture.rs`
Expected: no matches.

- [ ] **Step 10: Do NOT build yet** — the file references types (`LifecycleEvent`, `Screenshot { .. }`) that are still cfg-gated in `event_router.rs`. Task 3 unlocks them. Postpone the build check.

---

## Task 3: Unconditionalize `core::event_router.rs`

**Files:** Modify `crates/t-rec/src/core/event_router.rs`

- [ ] **Step 1: Remove all `cfg(feature="cli")` guards**

Rewrite the file so the enums and helpers are unconditional. Replace the whole file body (lines 1-138) with:

```rust
use tokio::sync::broadcast::Sender;

/// Capture commands for the Photographer actor.
#[derive(Debug, Clone)]
pub enum CaptureEvent {
    Start,
    /// Manual screenshot request.
    Screenshot { timecode_ms: u128 },
    Stop,
}

/// Visual feedback events for the Presenter actor.
#[derive(Debug, Clone)]
pub enum FlashEvent {
    ScreenshotTaken,
    RecordingStarted,
}

/// Lifecycle events for actor coordination.
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    Shutdown,
}

/// Unified event type for all actors.
#[derive(Debug, Clone)]
pub enum Event {
    Capture(CaptureEvent),
    Flash(FlashEvent),
    Lifecycle(LifecycleEvent),
}

/// Broadcasts events to all actors.
#[derive(Clone)]
pub struct EventRouter {
    tx: Sender<Event>,
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventRouter {
    pub fn new() -> Self {
        let (tx, _rx) = tokio::sync::broadcast::channel::<Event>(100);
        Self { tx }
    }

    /// Broadcast an event to all subscribed actors.
    pub fn send(&self, event: Event) {
        let _ = self.tx.send(event);
    }

    /// like `send` but returns a `Result`.
    pub fn try_send(
        &self,
        event: Event,
    ) -> Result<usize, tokio::sync::broadcast::error::SendError<Event>> {
        self.tx.send(event)
    }

    /// Sends a shutdown event to all actors.
    pub fn shutdown(&self) {
        let _ = self.tx.send(Event::Lifecycle(LifecycleEvent::Shutdown));
    }

    /// Subscribes to events from the router.
    /// Returns a receiver that listens for broadcasted events.
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_router() {
        let router = EventRouter::new();
        let mut receiver = router.subscribe();

        router.send(Event::Capture(CaptureEvent::Start));
        router.send(Event::Flash(FlashEvent::ScreenshotTaken));

        assert!(matches!(
            receiver.blocking_recv(),
            Ok(Event::Capture(CaptureEvent::Start))
        ));
        assert!(matches!(
            receiver.blocking_recv(),
            Ok(Event::Flash(FlashEvent::ScreenshotTaken))
        ));
    }

    #[test]
    fn test_event_broadcast() {
        let router = EventRouter::new();
        let mut receiver1 = router.subscribe();
        let mut receiver2 = router.subscribe();

        router.send(Event::Capture(CaptureEvent::Start));
        assert!(matches!(
            receiver1.blocking_recv(),
            Ok(Event::Capture(CaptureEvent::Start))
        ));
        assert!(matches!(
            receiver2.blocking_recv(),
            Ok(Event::Capture(CaptureEvent::Start))
        ));
    }

    #[test]
    fn test_event_router_shutdown() {
        let router = EventRouter::new();
        let mut receiver = router.subscribe();

        router.shutdown();

        assert!(matches!(
            receiver.blocking_recv(),
            Ok(Event::Lifecycle(LifecycleEvent::Shutdown))
        ));
    }
}
```

- [ ] **Step 2: Verify no remaining `cli` guards**

Run: `grep -n 'feature = "cli"' crates/t-rec/src/core/event_router.rs`
Expected: no matches.

---

## Task 4: Unconditionalize `core::screenshot.rs`

**Files:** Modify `crates/t-rec/src/core/screenshot.rs`

- [ ] **Step 1: Remove header cfg + import guard**

Change the top of the file (lines 1-4) from:

```rust
//! Screenshot capture during recording (CLI only).

#[cfg(feature = "cli")]
use std::path::PathBuf;
```

to:

```rust
//! Screenshot capture during recording.

use std::path::PathBuf;
```

- [ ] **Step 2: Remove item-level guards**

Delete every remaining `#[cfg(feature = "cli")]` line from the file. Update the `#[cfg(all(test, feature = "cli"))]` mod guard to `#[cfg(test)]`. Doc comments that mention "(CLI only)" can be simplified — trim the parenthetical where present.

- [ ] **Step 3: Verify**

Run: `grep -n 'feature = "cli"' crates/t-rec/src/core/screenshot.rs`
Expected: no matches.

---

## Task 5: Unconditionalize the remaining `core::` guards

**Files:**
- Modify `crates/t-rec/src/core/post_processing.rs`
- Modify `crates/t-rec/src/core/wallpapers/mod.rs`
- Modify `crates/t-rec/src/core/wallpapers/validation.rs`
- Modify `crates/t-rec/src/core/macos/mod.rs`
- Modify `crates/t-rec/src/core/windows/mod.rs`

- [ ] **Step 1: `post_processing.rs`**

At `crates/t-rec/src/core/post_processing.rs:17-19`, change:

```rust
use super::decors::{apply_corner_to_file, apply_shadow_to_file};
#[cfg(feature = "cli")]
use super::screenshot::ScreenshotInfo;
```

to:

```rust
use super::decors::{apply_corner_to_file, apply_shadow_to_file};
use super::screenshot::ScreenshotInfo;
```

Remove any other `#[cfg(feature = "cli")]` guards in the file (e.g. around line 88).

- [ ] **Step 2: `wallpapers/mod.rs`**

At `crates/t-rec/src/core/wallpapers/mod.rs:9-11`, change:

```rust
// Library-only exports (used by api/headless.rs)
#[cfg(not(feature = "cli"))]
pub use validation::load_and_validate_wallpaper;
```

to:

```rust
pub use validation::load_and_validate_wallpaper;
```

- [ ] **Step 3: `wallpapers/validation.rs`**

At `crates/t-rec/src/core/wallpapers/validation.rs:123-125`, remove the `#[cfg(not(feature = "cli"))]` line above `pub fn load_and_validate_wallpaper`.

- [ ] **Step 4: `macos/mod.rs`**

At `crates/t-rec/src/core/macos/mod.rs:16-18`, change:

```rust
/// Default shell for macOS (CLI only).
#[cfg(feature = "cli")]
pub const DEFAULT_SHELL: &str = "/bin/sh";
```

to:

```rust
/// Default shell for macOS.
pub const DEFAULT_SHELL: &str = "/bin/sh";
```

- [ ] **Step 5: `windows/mod.rs`**

Same treatment for `crates/t-rec/src/core/windows/mod.rs` — remove `#[cfg(feature = "cli")]` above `DEFAULT_SHELL`, trim "(CLI only)" from the doc comment.

- [ ] **Step 6: Verify no `cli` guards remain in `core::`**

Run: `grep -rn 'feature = "cli"' crates/t-rec/src/core/`
Expected: no matches.

---

## Task 6: Unconditionalize `src/api/headless.rs`

**Files:** Modify `crates/t-rec/src/api/headless.rs`

- [ ] **Step 1: Locate the residual guard**

Run: `grep -n 'feature = "cli"' crates/t-rec/src/api/headless.rs`
Expected: one match near line 565 — a `#[cfg(feature = "cli")]` on a struct-field init inside `CaptureContext { ..., screenshots: None, ... }`.

- [ ] **Step 2: Remove it**

Delete the `#[cfg(feature = "cli")]` line above `screenshots: None,`. The field is unconditional now (Task 2).

- [ ] **Step 3: Verify no `cli` guards remain in `api::`**

Run: `grep -rn 'feature = "cli"' crates/t-rec/src/api/`
Expected: no matches.

---

## Task 7: Restructure `Cargo.toml` — features + optional deps + docs.rs

**Files:** Modify `crates/t-rec/Cargo.toml`

- [ ] **Step 1: Rewrite the `[features]` block**

Replace lines 72-77 (the current `[features]` block):

```toml
[features]
default = []
e2e_tests = []
# Feature flag for CLI functionality - required for binary builds.
# Library users get a minimal lib without CLI-only code by default.
cli = []
```

with:

```toml
[features]
default = ["bin"]
# Bundles the deps needed to build the `t-rec` binary. Users never type this
# name — `cargo install t-rec` picks it up via `default`. Library users opt
# out via `--no-default-features`.
bin = [
    "dep:clap",
    "dep:crossterm",
    "dep:dialoguer",
    "dep:env_logger",
    "dep:humantime",
    "dep:toml",
    "dep:dirs",
]
# Enables the `HeadlessRecorder` public API. Carries no extra deps of its own.
headless = []
e2e_tests = []
```

- [ ] **Step 2: Mark bin-only deps `optional = true`**

At lines 21-34 (the `[dependencies]` block), change:

```toml
[dependencies]
anyhow.workspace = true
tempfile = "3.23"
rayon = "1.11"
log.workspace = true
env_logger = "0.11"
humantime = "2.3"
simplerand = "1.6"
serde = { version = "1.0", features = ["derive"] }
toml = "1.0"
dirs = "6.0"
dialoguer = { version = "0.12", default-features = false }
crossterm = "0.29"
tokio = { version = "1", default-features = false, features = ["sync"] }
```

to:

```toml
[dependencies]
# Always-on (lib-core)
anyhow.workspace = true
log.workspace = true
tempfile = "3.23"
rayon = "1.11"
simplerand = "1.6"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1", default-features = false, features = ["sync"] }

# Bin-only (pulled in by the `bin` feature)
env_logger = { version = "0.11", optional = true }
humantime = { version = "2.3", optional = true }
toml = { version = "1.0", optional = true }
dirs = { version = "6.0", optional = true }
dialoguer = { version = "0.12", optional = true, default-features = false }
crossterm = { version = "0.29", optional = true }
```

- [ ] **Step 3: Mark `clap` optional**

At lines 36-39, change:

```toml
[dependencies.clap]
version = "4.5"
features = ["cargo", "derive", "help", "color", "std"]
default-features = false
```

to:

```toml
[dependencies.clap]
version = "4.5"
features = ["cargo", "derive", "help", "color", "std"]
default-features = false
optional = true
```

- [ ] **Step 4: Update the `[[bin]]` target**

At lines 79-82, change:

```toml
[[bin]]
name = "t-rec"
path = "src/main.rs"
required-features = ["cli"]
```

to:

```toml
[[bin]]
name = "t-rec"
path = "src/main.rs"
required-features = ["bin"]
```

- [ ] **Step 5: Fix `docs.rs` metadata**

At lines 109-110, change:

```toml
[package.metadata.docs.rs]
features = ["lib"]
```

to:

```toml
[package.metadata.docs.rs]
features = ["headless"]
```

- [ ] **Step 6: Sanity-parse the manifest**

Run: `cargo metadata --format-version=1 --no-deps --manifest-path crates/t-rec/Cargo.toml >/dev/null`
Expected: exit 0, no error output.

---

## Task 8: Clean up `src/lib.rs` — headless gate, unconditional re-exports

**Files:** Modify `crates/t-rec/src/lib.rs`

- [ ] **Step 1: Replace the guard block**

Replace lines 66-107 of `crates/t-rec/src/lib.rs` with:

```rust
// Core shared modules — used by both the binary and library consumers.
pub mod core;

// Public library API — opt-in via the `headless` feature.
#[cfg(feature = "headless")]
mod api;

#[cfg(feature = "headless")]
pub use api::{HeadlessRecorder, HeadlessRecorderBuilder, HeadlessRecorderConfig, RecordingOutput};

// Re-export core types.
pub use core::{
    Image, ImageOnHeap, Margin, PlatformApi, Result, WindowId, WindowList, WindowListEntry,
};

// Re-export public modules.
pub use core::error;
pub use core::types;
pub use core::types::{BackgroundColor, Decor};
pub use core::wallpapers;
pub use core::wallpapers::{load_and_validate_wallpaper, resolve_wallpaper, Wallpaper};

// Re-exports used by the binary and available to any consumer.
pub use core::common::{Platform, PlatformApiFactory};
pub use core::event_router::{CaptureEvent, Event, EventRouter, FlashEvent, LifecycleEvent};
#[cfg(target_os = "linux")]
pub use core::linux::DEFAULT_SHELL;
#[cfg(target_os = "macos")]
pub use core::macos::DEFAULT_SHELL;
#[cfg(target_os = "windows")]
pub use core::windows::DEFAULT_SHELL;
pub use core::post_processing::post_process_screenshots;
pub use core::screenshot::{screenshot_file_name, screenshot_output_name, ScreenshotInfo};
```

Note: I've also added the missing `target_os = "windows"` variant for `DEFAULT_SHELL` — the current lib.rs re-exports the linux and macos ones only. If `core::windows::DEFAULT_SHELL` does not exist, delete that one line; if it does, keep it. Verify with:

Run: `grep -n 'DEFAULT_SHELL' crates/t-rec/src/core/windows/mod.rs`
If no match: remove the `#[cfg(target_os = "windows")] pub use core::windows::DEFAULT_SHELL;` line.

- [ ] **Step 2: Verify no `cli` guards remain**

Run: `grep -n 'feature = "cli"' crates/t-rec/src/lib.rs`
Expected: no matches.

- [ ] **Step 3: Build the lib in default (bin) mode**

Run: `cargo build --release --locked -p t-rec`
Expected: PASS. The binary compiles.

- [ ] **Step 4: Build the lib without default features**

Run: `cargo build --release --no-default-features --locked -p t-rec --lib`
Expected: PASS. `HeadlessRecorder` API not exposed, but the core surface compiles.

- [ ] **Step 5: Build with `--features headless` only**

Run: `cargo build --release --no-default-features --features headless --locked -p t-rec --lib`
Expected: PASS. `HeadlessRecorder` compiles.

---

## Task 9: Sweep `src/main.rs` and `src/cli/**` for residual guards

**Files:** Modify (if any residual guards found) `crates/t-rec/src/main.rs` and files under `crates/t-rec/src/cli/`.

- [ ] **Step 1: Scan**

Run: `grep -rn 'feature = "cli"' crates/t-rec/src/main.rs crates/t-rec/src/cli/`
Expected on a clean codebase: no matches. If matches exist, remove them — the binary only compiles when `required-features = ["bin"]` is satisfied, so `feature = "cli"` guards are redundant.

- [ ] **Step 2: Whole-tree final scan**

Run: `grep -rn 'feature = "cli"' crates/t-rec/`
Expected: no matches anywhere.

---

## Task 10: Verification suite

**Files:** none — commands only.

- [ ] **Step 1: Install path (the whole point of the change)**

Run: `cargo install --path crates/t-rec --locked --force`
Expected: PASS. `t-rec` binary installed into `~/.cargo/bin/`.

- [ ] **Step 2: Default build**

Run: `cargo build --release --locked`
Expected: PASS.

- [ ] **Step 3: Lib-only build (slim)**

Run: `cargo build --release --no-default-features --locked -p t-rec --lib`
Expected: PASS.

- [ ] **Step 4: Lib with headless build**

Run: `cargo build --release --no-default-features --features headless --locked -p t-rec --lib`
Expected: PASS.

- [ ] **Step 5: Verify slim dep tree**

Run: `cargo tree --no-default-features --features headless -p t-rec --edges normal --prefix depth --charset ascii 2>/dev/null | grep -E '^\d+(clap|crossterm|dialoguer|env_logger|humantime|toml|dirs)( |$)'`
Expected: no matches — none of the seven bin-only crates appear in the lib-only tree.

- [ ] **Step 6: Test suite**

Run: `cargo test --locked`
Expected: PASS.

- [ ] **Step 7: Clippy — default**

Run: `cargo clippy --locked --all-targets -- -D warnings`
Expected: PASS.

- [ ] **Step 8: Clippy — lib-only**

Run: `cargo clippy --no-default-features --features headless --locked -- -D warnings`
Expected: PASS.

- [ ] **Step 9: `cargo package` dry run**

Run: `cargo package --list -p t-rec | head -40`
Expected: PASS. No errors about missing manifest fields.

---

## Task 11: Commit the series

**Files:** all of the above.

- [ ] **Step 1: Review the working tree**

Run: `git status --short && git diff --stat`
Expected: modifications only under `crates/t-rec/` and one new spec + plan file under `docs/superpowers/`.

- [ ] **Step 2: Stage the code changes**

Run:

```bash
git add \
  crates/t-rec/Cargo.toml \
  crates/t-rec/src/lib.rs \
  crates/t-rec/src/main.rs \
  crates/t-rec/src/api/headless.rs \
  crates/t-rec/src/core/capture.rs \
  crates/t-rec/src/core/event_router.rs \
  crates/t-rec/src/core/post_processing.rs \
  crates/t-rec/src/core/screenshot.rs \
  crates/t-rec/src/core/wallpapers/mod.rs \
  crates/t-rec/src/core/wallpapers/validation.rs \
  crates/t-rec/src/core/macos/mod.rs \
  crates/t-rec/src/core/windows/mod.rs
```

(Add any additional files if Task 9 turned up hits.)

- [ ] **Step 3: Compose the commit message using the `conventional-commits` skill**

Draft (subject line ≤72 chars, matches personal style — bare `refactor` prefix, verb-only):

```
refactor(cargo): default-install binary, add opt-in headless lib feature

Removes the `cli` feature and the install friction it caused
(`cargo install t-rec` no longer needs `--features cli`).

- `[features] default = ["bin"]` bundles CLI-only deps (clap, crossterm,
  dialoguer, env_logger, humantime, toml, dirs). Users never type it.
- `[features] headless = []` gates the public `HeadlessRecorder` API.
  Third-party integrators depend on t-rec via
  `default-features = false, features = ["headless"]` for a slim dep tree.
- Collapses the previous `cfg(feature = "cli")` dual paths inside
  `core::` to the fuller (CLI) branch. The `not(cli)` variants were
  simplified duplicates and now go away.
- Updates `docs.rs` metadata to render the `headless` feature.
```

Invoke the `conventional-commits` skill for the final commit-message form and any personal-style adjustments Sven prefers. Then run:

```bash
git commit -S --signoff -m "<final subject>" -m "<final body>"
```

(Note: `git cm` alias covers `-S --signoff -m`. Prefer a HEREDOC for the body so newlines render correctly.)

- [ ] **Step 4: Stage and commit the design + plan docs (separate commit for a clean history)**

```bash
git add docs/superpowers/specs/2026-07-01-single-crate-headless-feature-design.md \
        docs/superpowers/plans/2026-07-01-single-crate-headless-feature.md
```

Commit message (invoke `conventional-commits` skill):

```
docs: brainstorming spec + implementation plan for headless feature
```

- [ ] **Step 5: Post-commit verification**

Run: `git log --oneline -3`
Expected: two new commits on top of `fix/cli`.

Run: `git status --short`
Expected: clean tree.

---

## Notes for the executing engineer

- **Do not use `--no-verify`.** The repo has pre-commit hooks (formatting, sign-off enforcement). Fix issues; don't skip.
- **Do not amend.** Create new commits if a hook fails after fixing.
- **`git cm` is `git commit -S --signoff -m`** in Sven's shell. In a subshell you must expand it — see the alias table at the top of the session.
- **Remote is named `o`, not `origin`.** Do not push unless Sven asks.
- **If `Task 8 / Step 1` shows `core::windows::DEFAULT_SHELL` does not exist**, delete the `target_os = "windows"` re-export line; the current lib.rs never exposed it.
- **If any `cargo build --no-default-features` step fails with an "unused import" or "dead_code" warning in `core::`** treated as an error, the collapsing in Tasks 2-6 missed a spot. `grep` the failing symbol name and un-gate its declaration site.

---

## Self-review pass (author)

**Spec coverage:**

- Feature layout (`default = ["bin"]`, `bin`, `headless`) → Task 7.
- Dep partitioning (7 bin-only + 8 lib-core) → Task 7.
- `[[bin]] required-features = ["bin"]` → Task 7 Step 4.
- `src/lib.rs` unconditional re-exports + `headless` gate on `api` → Task 8.
- Drop `#[cfg(feature = "cli")]` in `main.rs` and `cli/*` → Task 9.
- Update `[package.metadata.docs.rs]` → Task 7 Step 5.
- README removal of `--features cli`: not needed — grep confirmed the README already uses `cargo install -f t-rec` and does not reference the feature. **The README broken today (it says to install without the flag) becomes correct after this change.** No README edit required.
- Verification steps from spec (`cargo build` variants, `cargo tree`, `cargo test`, clippy, `cargo install`) → Task 10.
- CHANGELOG callout: the spec calls for a note. Left as an optional item in "File overview". If Sven wants it enforced, add a Task 11.5 to append to `CHANGELOG.md` before commit.

**Placeholder scan:** no TBD / TODO / "handle appropriately" left.

**Type consistency:** identifiers referenced (`HeadlessRecorder`, `HeadlessRecorderBuilder`, `HeadlessRecorderConfig`, `RecordingOutput`, `Event`, `CaptureEvent`, `FlashEvent`, `LifecycleEvent`, `ScreenshotInfo`, `EventRouter`, `Platform`, `PlatformApiFactory`, `Wallpaper`, `resolve_wallpaper`, `load_and_validate_wallpaper`, `DEFAULT_SHELL`) match the current codebase (verified via `grep`).

**Scope check:** single-crate refactor, all changes limited to `crates/t-rec/` + docs. Fits one implementation session.
