//! Crossterm-based keyboard input interception.
//!
//! This module provides keyboard event handling using the crossterm crate.
//! It intercepts keyboard input in raw mode, detects hotkeys (F2 for screenshot,
//! F3 for toggle), and forwards regular input to the shell.
//!
//! ## Architecture
//!
//! ```text
//! stdin → [Raw Mode] → [crossterm::event] → KeyEvent
//!                              ↓
//!                     ┌───────┴───────┐
//!                     │               │
//!                     ▼               ▼
//!              [Hotkey Handler]  [Shell Forward]
//!                     │               │
//!                     ▼               ▼
//!              [EventRouter]    [PTY/Shell stdin]
//! ```
//!
//! ## Cross-platform
//!
//! Works on macOS, Linux, and Windows without special permissions.

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::capture::{CaptureEvent, Event as RouterEvent, EventRouter, FlashEvent};

/// Hotkey actions triggered by configured keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum HotkeyAction {
    /// Take a screenshot (F2 by default).
    Screenshot,
    /// Toggle keystroke overlay on/off (F3 by default).
    ToggleKeystrokeCapture,
}

/// Key combination for hotkey configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyCombo {
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

impl KeyCombo {
    /// Convert a crossterm KeyCode to a KeyCombo (if it's a function key).
    fn from_keycode(code: KeyCode) -> Option<Self> {
        match code {
            KeyCode::F(1) => Some(KeyCombo::F1),
            KeyCode::F(2) => Some(KeyCombo::F2),
            KeyCode::F(3) => Some(KeyCombo::F3),
            KeyCode::F(4) => Some(KeyCombo::F4),
            KeyCode::F(5) => Some(KeyCombo::F5),
            KeyCode::F(6) => Some(KeyCombo::F6),
            KeyCode::F(7) => Some(KeyCombo::F7),
            KeyCode::F(8) => Some(KeyCombo::F8),
            KeyCode::F(9) => Some(KeyCombo::F9),
            KeyCode::F(10) => Some(KeyCombo::F10),
            KeyCode::F(11) => Some(KeyCombo::F11),
            KeyCode::F(12) => Some(KeyCombo::F12),
            _ => None,
        }
    }
}

/// Hotkey configuration.
#[derive(Debug, Clone)]
pub struct HotkeyConfig {
    pub screenshot: Option<KeyCombo>,
    pub toggle_keystroke_capturing: Option<KeyCombo>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            screenshot: Some(KeyCombo::F2),
            toggle_keystroke_capturing: Some(KeyCombo::F3),
        }
    }
}

/// Events produced by the keyboard monitor.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum InputEvent {
    /// A keystroke to be displayed in overlay.
    Keystroke {
        /// Human-readable key name ("A", "Return", "Ctrl+C").
        key_name: String,
        /// When the key was pressed.
        instant: Instant,
        /// Adjusted timecode in milliseconds (for frame sync).
        timecode_ms: u128,
    },
}

/// Shared state for keyboard-driven features.
pub struct InputState {
    /// Collected keystroke events for overlay.
    pub keystrokes: Mutex<Vec<InputEvent>>,
    /// Whether keystroke capture is currently enabled.
    pub keystroke_capture_enabled: AtomicBool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            keystrokes: Mutex::new(Vec::new()),
            keystroke_capture_enabled: AtomicBool::new(false),
        }
    }

    /// Toggle keystroke capture on/off.
    pub fn toggle_capture(&self) -> bool {
        let current = self.keystroke_capture_enabled.load(Ordering::Acquire);
        self.keystroke_capture_enabled
            .store(!current, Ordering::Release);
        !current
    }

    /// Check if keystroke capture is enabled.
    pub fn is_capture_enabled(&self) -> bool {
        self.keystroke_capture_enabled.load(Ordering::Acquire)
    }

    /// Push a keystroke event to the collection.
    pub fn push_keystroke(&self, key_name: String, instant: Instant, timecode_ms: u128) {
        self.keystrokes.lock().unwrap().push(InputEvent::Keystroke {
            key_name,
            instant,
            timecode_ms,
        });
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of handling a key event.
enum KeyAction {
    /// Hotkey was handled, don't forward to shell.
    Handled,
    /// Forward these bytes to shell.
    Forward(Vec<u8>),
    /// Exit the keyboard monitor (Ctrl+D or shell exit).
    Exit,
}

/// Crossterm-based keyboard monitor.
///
/// Reads keyboard input in raw mode, detects hotkeys, and forwards
/// regular input to the shell.
pub struct KeyboardMonitor {
    input_state: Arc<InputState>,
    idle_duration: Arc<Mutex<Duration>>,
    recording_start: Instant,
    hotkey_config: HotkeyConfig,
    router: EventRouter,
}

impl KeyboardMonitor {
    /// Create a new keyboard monitor.
    pub fn new(
        input_state: Arc<InputState>,
        idle_duration: Arc<Mutex<Duration>>,
        recording_start: Instant,
        hotkey_config: HotkeyConfig,
        router: EventRouter,
    ) -> Self {
        Self {
            input_state,
            idle_duration,
            recording_start,
            hotkey_config,
            router,
        }
    }

    /// Run the keyboard monitor loop.
    ///
    /// This function:
    /// 1. Enables raw mode
    /// 2. Polls for keyboard events
    /// 3. Handles hotkeys (F2/F3)
    /// 4. Forwards regular keys to the shell
    /// 5. Restores terminal on exit
    ///
    /// # Arguments
    /// * `shell_stdin` - Writer to send input to the shell
    /// * `should_exit` - Atomic flag to signal exit (set by shell exit)
    pub fn run<W: Write>(
        &self,
        mut shell_stdin: W,
        should_exit: Arc<AtomicBool>,
    ) -> anyhow::Result<()> {
        log::debug!("Keyboard monitor starting - F2=screenshot, F3=toggle capture, Ctrl+D=exit");
        enable_raw_mode()?;
        log::debug!("Raw mode enabled");

        let result = self.run_loop(&mut shell_stdin, should_exit);

        // Always restore terminal, even on error
        let _ = disable_raw_mode();

        result
    }

    fn run_loop<W: Write>(
        &self,
        shell_stdin: &mut W,
        should_exit: Arc<AtomicBool>,
    ) -> anyhow::Result<()> {
        loop {
            // Check if we should exit
            if should_exit.load(Ordering::Acquire) {
                break;
            }

            // Poll with timeout to allow checking exit flag
            if event::poll(Duration::from_millis(50))? {
                match event::read()? {
                    Event::Key(key_event) => {
                        // Only handle key press, not release
                        if key_event.kind != KeyEventKind::Press {
                            continue;
                        }

                        match self.handle_key(key_event) {
                            KeyAction::Handled => {}
                            KeyAction::Forward(bytes) => {
                                shell_stdin.write_all(&bytes)?;
                                shell_stdin.flush()?;
                            }
                            KeyAction::Exit => break,
                        }
                    }
                    Event::Resize(_, _) => {
                        // Terminal resized, could handle this if needed
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn handle_key(&self, key: KeyEvent) -> KeyAction {
        let code = key.code;
        let modifiers = key.modifiers;

        log::debug!("Key event: {:?} modifiers: {:?}", code, modifiers);

        // Check for hotkeys (function keys)
        if let Some(key_combo) = KeyCombo::from_keycode(code) {
            log::debug!("Function key detected: {:?}", key_combo);

            // Screenshot hotkey
            if self.hotkey_config.screenshot.as_ref() == Some(&key_combo) {
                log::debug!("F2 screenshot hotkey detected");
                self.trigger_screenshot();
                return KeyAction::Handled;
            }

            // Toggle keystroke capture hotkey
            if self.hotkey_config.toggle_keystroke_capturing.as_ref() == Some(&key_combo) {
                let enabled = self.input_state.toggle_capture();
                log::debug!("Keystroke capture: {}", if enabled { "ON" } else { "OFF" });
                return KeyAction::Handled;
            }
        }

        // Check for Ctrl+D (exit)
        if code == KeyCode::Char('d') && modifiers.contains(KeyModifiers::CONTROL) {
            return KeyAction::Exit;
        }

        // Record keystroke if capture enabled
        if self.input_state.is_capture_enabled() {
            let key_name = self.format_key_name(&key);
            let idle = *self.idle_duration.lock().unwrap();
            let timecode_ms = Instant::now()
                .duration_since(self.recording_start)
                .saturating_sub(idle)
                .as_millis();
            self.input_state
                .push_keystroke(key_name, Instant::now(), timecode_ms);
        }

        // Forward to shell
        KeyAction::Forward(self.key_to_bytes(&key))
    }

    fn trigger_screenshot(&self) {
        let idle = *self.idle_duration.lock().unwrap();
        let timecode_ms = Instant::now()
            .duration_since(self.recording_start)
            .saturating_sub(idle)
            .as_millis();

        // Send events via router
        self.router
            .send(RouterEvent::Capture(CaptureEvent::Screenshot {
                timecode_ms,
            }));
        self.router
            .send(RouterEvent::Flash(FlashEvent::ScreenshotTaken));

        log::debug!("Screenshot triggered at timecode {}", timecode_ms);
    }

    fn format_key_name(&self, key: &KeyEvent) -> String {
        let mut name = String::new();

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            name.push_str("Ctrl+");
        }
        if key.modifiers.contains(KeyModifiers::ALT) {
            name.push_str("Alt+");
        }
        if key.modifiers.contains(KeyModifiers::SHIFT) && !matches!(key.code, KeyCode::Char(_)) {
            name.push_str("Shift+");
        }

        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    name.push(c.to_ascii_uppercase());
                } else {
                    name.push(c);
                }
            }
            KeyCode::Enter => name.push_str("Return"),
            KeyCode::Tab => name.push_str("Tab"),
            KeyCode::Backspace => name.push_str("Backspace"),
            KeyCode::Esc => name.push_str("Escape"),
            KeyCode::Delete => name.push_str("Delete"),
            KeyCode::F(n) => name.push_str(&format!("F{}", n)),
            KeyCode::Left => name.push_str("Left"),
            KeyCode::Right => name.push_str("Right"),
            KeyCode::Up => name.push_str("Up"),
            KeyCode::Down => name.push_str("Down"),
            KeyCode::Home => name.push_str("Home"),
            KeyCode::End => name.push_str("End"),
            KeyCode::PageUp => name.push_str("PageUp"),
            KeyCode::PageDown => name.push_str("PageDown"),
            KeyCode::Insert => name.push_str("Insert"),
            _ => name.push_str("Unknown"),
        }

        name
    }

    fn key_to_bytes(&self, key: &KeyEvent) -> Vec<u8> {
        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+A = 0x01, Ctrl+B = 0x02, etc.
                    let ctrl_code = (c.to_ascii_lowercase() as u8)
                        .wrapping_sub(b'a')
                        .wrapping_add(1);
                    if ctrl_code <= 26 {
                        vec![ctrl_code]
                    } else {
                        // Non-letter control characters
                        let mut buf = [0u8; 4];
                        let s = c.encode_utf8(&mut buf);
                        s.as_bytes().to_vec()
                    }
                } else {
                    let mut buf = [0u8; 4];
                    let s = c.encode_utf8(&mut buf);
                    s.as_bytes().to_vec()
                }
            }
            KeyCode::Enter => vec![0x0D],
            KeyCode::Tab => vec![0x09],
            KeyCode::Backspace => vec![0x7F],
            KeyCode::Esc => vec![0x1B],
            KeyCode::Delete => vec![0x1B, b'[', b'3', b'~'],
            // Function keys (xterm style)
            KeyCode::F(1) => vec![0x1B, b'O', b'P'],
            KeyCode::F(2) => vec![0x1B, b'O', b'Q'],
            KeyCode::F(3) => vec![0x1B, b'O', b'R'],
            KeyCode::F(4) => vec![0x1B, b'O', b'S'],
            KeyCode::F(5) => vec![0x1B, b'[', b'1', b'5', b'~'],
            KeyCode::F(6) => vec![0x1B, b'[', b'1', b'7', b'~'],
            KeyCode::F(7) => vec![0x1B, b'[', b'1', b'8', b'~'],
            KeyCode::F(8) => vec![0x1B, b'[', b'1', b'9', b'~'],
            KeyCode::F(9) => vec![0x1B, b'[', b'2', b'0', b'~'],
            KeyCode::F(10) => vec![0x1B, b'[', b'2', b'1', b'~'],
            KeyCode::F(11) => vec![0x1B, b'[', b'2', b'3', b'~'],
            KeyCode::F(12) => vec![0x1B, b'[', b'2', b'4', b'~'],
            // Arrow keys
            KeyCode::Up => vec![0x1B, b'[', b'A'],
            KeyCode::Down => vec![0x1B, b'[', b'B'],
            KeyCode::Right => vec![0x1B, b'[', b'C'],
            KeyCode::Left => vec![0x1B, b'[', b'D'],
            // Navigation keys
            KeyCode::Home => vec![0x1B, b'[', b'H'],
            KeyCode::End => vec![0x1B, b'[', b'F'],
            KeyCode::PageUp => vec![0x1B, b'[', b'5', b'~'],
            KeyCode::PageDown => vec![0x1B, b'[', b'6', b'~'],
            KeyCode::Insert => vec![0x1B, b'[', b'2', b'~'],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycombo_from_keycode() {
        assert_eq!(KeyCombo::from_keycode(KeyCode::F(1)), Some(KeyCombo::F1));
        assert_eq!(KeyCombo::from_keycode(KeyCode::F(2)), Some(KeyCombo::F2));
        assert_eq!(KeyCombo::from_keycode(KeyCode::F(12)), Some(KeyCombo::F12));
        assert_eq!(KeyCombo::from_keycode(KeyCode::Char('a')), None);
    }

    #[test]
    fn test_hotkey_config_default() {
        let config = HotkeyConfig::default();
        assert_eq!(config.screenshot, Some(KeyCombo::F2));
        assert_eq!(config.toggle_keystroke_capturing, Some(KeyCombo::F3));
    }

    #[test]
    fn test_input_state_default() {
        let state = InputState::new();
        assert!(!state.keystroke_capture_enabled.load(Ordering::Acquire));
        assert!(state.keystrokes.lock().unwrap().is_empty());
    }

    #[test]
    fn test_input_state_toggle_capture() {
        let state = InputState::new();
        assert!(!state.is_capture_enabled());

        let result = state.toggle_capture();
        assert!(result);
        assert!(state.is_capture_enabled());

        let result = state.toggle_capture();
        assert!(!result);
        assert!(!state.is_capture_enabled());
    }
}
