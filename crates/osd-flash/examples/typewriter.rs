//! Typewriter example - displays keyboard keys as you type.
//!
//! Shows each keypress as a simple key indicator.
//! Press 'q' or ESC to quit.
//!
//! Run with: cargo run -p osd-flash --example typewriter

#[cfg(target_os = "macos")]
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
#[cfg(target_os = "macos")]
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
#[cfg(target_os = "macos")]
use crossterm::ExecutableCommand;
#[cfg(target_os = "macos")]
use osd_flash::prelude::*;
#[cfg(target_os = "macos")]
use std::io::stdout;

#[cfg(target_os = "macos")]
fn main() -> osd_flash::Result<()> {
    println!("=== Typewriter Demo ===");
    println!("Type any key to see it displayed.");
    println!("Press 'q' or ESC to quit.\n");

    // Enable raw mode to capture individual keypresses
    terminal::enable_raw_mode().expect("Failed to enable raw mode");
    stdout()
        .execute(EnterAlternateScreen)
        .expect("Failed to enter alternate screen");

    let result = run_typewriter();

    // Restore terminal
    stdout()
        .execute(LeaveAlternateScreen)
        .expect("Failed to leave alternate screen");
    terminal::disable_raw_mode().expect("Failed to disable raw mode");

    result
}

#[cfg(target_os = "macos")]
fn run_typewriter() -> osd_flash::Result<()> {
    loop {
        // Wait for a key event
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read().expect("Failed to read event")
        {
            // Check for quit keys
            if code == KeyCode::Esc || (code == KeyCode::Char('q') && modifiers.is_empty()) {
                break;
            }

            // Get the display string for this key
            let key_label = get_key_label(code, modifiers);

            // Skip if no label (unknown key)
            if key_label.is_empty() {
                continue;
            }

            // Show the key
            show_key(&key_label)?;
        }
    }

    Ok(())
}

/// Get display label for a key
#[cfg(target_os = "macos")]
fn get_key_label(code: KeyCode, modifiers: KeyModifiers) -> String {
    match code {
        KeyCode::Char(c) => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                format!("^{}", c.to_uppercase())
            } else if c == ' ' {
                "SPC".to_string()
            } else {
                c.to_uppercase().to_string()
            }
        }
        KeyCode::Enter => "RET".to_string(),
        KeyCode::Tab => "TAB".to_string(),
        KeyCode::Backspace => "DEL".to_string(),
        KeyCode::Delete => "DEL".to_string(),
        KeyCode::Left => "←".to_string(),
        KeyCode::Right => "→".to_string(),
        KeyCode::Up => "↑".to_string(),
        KeyCode::Down => "↓".to_string(),
        KeyCode::Home => "HOM".to_string(),
        KeyCode::End => "END".to_string(),
        KeyCode::PageUp => "PGU".to_string(),
        KeyCode::PageDown => "PGD".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        KeyCode::CapsLock => "CAP".to_string(),
        _ => String::new(),
    }
}

/// Display a key on screen
#[cfg(target_os = "macos")]
fn show_key(label: &str) -> osd_flash::Result<()> {
    let size = if label.len() > 2 { 120.0 } else { 80.0 };
    let font_size = if label.len() > 2 { 20.0 } else { 32.0 };

    OsdBuilder::new()
        .size(size)
        .position(Position::Center)
        .background(Color::rgba(0.96, 0.96, 0.97, 0.98))
        .corner_radius(12.0)
        // Key surface shadow
        .layer("shadow", |l| {
            l.circle(size * 0.7)
                .center_offset(2.0, 4.0)
                .fill(Color::rgba(0.0, 0.0, 0.0, 0.15))
        })
        // Key surface
        .layer("surface", |l| {
            l.circle(size * 0.65)
                .center()
                .fill(Color::rgba(0.92, 0.92, 0.93, 1.0))
        })
        // Key label
        .layer("label", |l| {
            l.text(label)
                .center()
                .font_size(font_size)
                .font_weight(FontWeight::Bold)
                .text_color(Color::rgba(0.2, 0.2, 0.22, 1.0))
        })
        .show_for(800.millis())
}


#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("This example only runs on macOS");
}
