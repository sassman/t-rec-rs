# Implementation Plan: Interactive Video Generation Prompt

## Overview

Add an interactive prompt at the end of recording that asks users if they want to generate an MP4 video, with a 10-second timeout that defaults to "no".

**Reference**: [GitHub Issue #219](https://github.com/sassman/t-rec-rs/issues/219)

## Requirements from Issue #219

1. **Trigger condition**: Only prompt when user did NOT use `--video` or `--video-only`
2. **Interactive prompt**: Ask if user wants MP4 video with default [n]o
3. **10-second timeout**: Auto-decline after 10 seconds of no input
4. **Generate video**: If user says yes, generate MP4 as if `--video` was passed

## User Experience

### Prompt Display

```
🎉 🚀 Generating t-rec.gif

📋 Recording summary
   ├─ fps: 4
   ├─ idle-pause: 3s
   ├─ frames: 127
   └─ output: t-rec

🎬 Also generate MP4 video? [y/N] (auto-skip in 10s): █
```

### Countdown Display

The countdown should update in-place:
```
🎬 Also generate MP4 video? [y/N] (auto-skip in 10s):
🎬 Also generate MP4 video? [y/N] (auto-skip in 9s):
...
🎬 Also generate MP4 video? [y/N] (auto-skip in 1s):
```

### Responses

- `y` or `Y` → Generate MP4
- `n`, `N`, or Enter → Skip MP4
- Timeout (10s) → Skip MP4 (with message "Skipping video generation")

---

## Implementation Steps

### Step 1: Add Prompt Module

**File**: `src/prompt.rs`

Create a new module for interactive prompts with timeout support:

```rust
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

/// Result of a timed prompt
pub enum PromptResult {
    Yes,
    No,
    Timeout,
}

/// Prompts the user with a yes/no question with a countdown timeout.
///
/// Returns `PromptResult::Yes` if user enters 'y' or 'Y',
/// `PromptResult::No` if user enters 'n', 'N', or just presses Enter,
/// `PromptResult::Timeout` if the timeout expires.
pub fn prompt_yes_no_with_timeout(question: &str, timeout_secs: u64) -> PromptResult {
    let (tx, rx) = mpsc::channel();

    // Spawn thread to read user input
    let tx_clone = tx.clone();
    thread::spawn(move || {
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let _ = tx_clone.send(Some(input.trim().to_lowercase()));
        }
    });

    // Countdown loop
    for remaining in (1..=timeout_secs).rev() {
        // Print prompt with countdown (overwrite previous line)
        print!("\r{} [y/N] (auto-skip in {}s): ", question, remaining);
        io::stdout().flush().unwrap();

        // Check for input with 1-second timeout
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(Some(input)) => {
                println!(); // Move to next line
                return match input.as_str() {
                    "y" | "yes" => PromptResult::Yes,
                    _ => PromptResult::No,
                };
            }
            Ok(None) => {
                println!();
                return PromptResult::No;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Continue countdown
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                println!();
                return PromptResult::No;
            }
        }
    }

    // Timeout reached
    println!("\r{} [y/N] (auto-skip in 0s): ", question);
    println!("Skipping video generation (timeout)");
    PromptResult::Timeout
}
```

### Step 2: Update Main to Use Prompt

**File**: `src/main.rs`

Add the prompt module and use it after GIF generation:

```rust
mod prompt;

use crate::prompt::{prompt_yes_no_with_timeout, PromptResult};

// In main(), after GIF generation, before the existing video generation block:

if should_generate_gif {
    time += prof! {
        generate_gif(/* ... */)?;
    };
}

// NEW: Interactive prompt for video generation
let should_generate_video = if should_generate_video {
    true // User already requested video via CLI
} else if !settings.quiet() {
    // Ask user if they want video (only if not in quiet mode)
    match prompt_yes_no_with_timeout("🎬 Also generate MP4 video?", 10) {
        PromptResult::Yes => {
            check_for_mp4()?; // Verify ffmpeg is available
            true
        }
        PromptResult::No | PromptResult::Timeout => false,
    }
} else {
    false // Quiet mode: don't prompt
};

if should_generate_video {
    time += prof! {
        generate_mp4(/* ... */)?;
    }
}
```

### Step 3: Handle Quiet Mode

In quiet mode (`-q`), skip the prompt entirely - users who want automation shouldn't be interrupted.

### Step 4: Handle Video-Only Mode

If `--video-only` is used, no GIF is generated and no prompt is shown (video is already being generated).

### Step 5: Config File Support (Optional)

Add an optional config setting to disable the prompt:

**File**: `src/config/profile.rs`

```rust
pub struct ProfileSettings {
    // ... existing fields ...
    pub prompt_video: Option<bool>,
}

impl ProfileSettings {
    pub fn prompt_video(&self) -> bool {
        self.prompt_video.unwrap_or(true) // Default: show prompt
    }
}
```

This allows users to disable the prompt in their config:
```toml
[default]
prompt-video = false
```

---

## File Changes Summary

| File | Change Type | Description |
|------|-------------|-------------|
| `src/prompt.rs` | **New** | Interactive prompt module with timeout |
| `src/main.rs` | Modify | Import prompt module, add video prompt logic |
| `src/config/profile.rs` | Modify | Add `prompt_video` setting (optional) |

---

## Edge Cases

### 1. Quiet Mode (`-q`)
- Skip prompt entirely
- Respect original `--video` / `--video-only` flags

### 2. Video Already Requested
- If `--video` or `--video-only` was passed, don't prompt
- Generate video as normal

### 3. ffmpeg Not Installed
- If user says "yes" but ffmpeg is not available, show error
- Call `check_for_mp4()` after user confirms

### 4. Piped Input / Non-TTY
- If stdin is not a terminal, skip prompt (non-interactive mode)
- Can detect with `atty::is(atty::Stream::Stdin)`

### 5. CI/CD Environments
- The timeout ensures the process doesn't hang
- Quiet mode can be used to skip prompt

---

## Dependencies

Consider adding `atty` crate for TTY detection:

```toml
[dependencies]
atty = "0.2"
```

This allows detecting if we're running interactively:
```rust
if atty::is(atty::Stream::Stdin) {
    // Show prompt
} else {
    // Non-interactive, skip prompt
}
```

---

## Testing Plan

### Unit Tests

1. **PromptResult enum**: Verify Yes/No/Timeout variants
2. **Input parsing**: "y", "Y", "yes", "n", "N", "no", "", etc.

### Manual Tests

1. Run `t-rec`, wait for prompt, press `y` → video generated
2. Run `t-rec`, wait for prompt, press `n` → no video
3. Run `t-rec`, wait for prompt, press Enter → no video (default)
4. Run `t-rec`, wait 10 seconds → timeout, no video
5. Run `t-rec --video` → no prompt, video generated
6. Run `t-rec --video-only` → no prompt, only video
7. Run `t-rec -q` → no prompt (quiet mode)
8. Run `echo "" | t-rec` → no prompt (non-interactive)

---

## Visual Flow

```
Recording...
[Ctrl+D pressed]

📋 Recording summary
   ├─ fps: 4
   ├─ idle-pause: 3s
   ├─ frames: 127
   └─ output: t-rec

🎆 Applying effects (might take a bit)
💡 Tip: For smoother typing animations, try `--fps 10` or `--fps 15`

🎉 🚀 Generating t-rec.gif

🎬 Also generate MP4 video? [y/N] (auto-skip in 10s): y

🎉 🚀 Generating t-rec.mp4

Time: 2.34s
```

---

## Future Enhancements (Out of Scope)

- Configurable timeout duration via CLI or config
- Remember user's choice for future runs
- Prompt for other optional features (e.g., upload to cloud)
