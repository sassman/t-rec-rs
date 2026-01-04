//! Typewriter example - displays keyboard keys as you type.
//!
//! Shows each keypress as a Mac-style keyboard key (light external keyboard aesthetic).
//! Press 'q' or ESC to quit.
//!
//! Run with: cargo run -p osd-flash --example typewriter

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use osd_flash::prelude::*;
use std::io::stdout;

/// Colors for Mac Magic Keyboard key (silver/light version)
mod key_colors {
    use osd_flash::prelude::Color;

    // Drop shadow (beneath the entire key)
    pub const DROP_SHADOW: Color = Color::rgba(0.0, 0.0, 0.0, 0.3);

    // Key body/base (the "sides" of the key)
    pub const KEY_BODY: Color = Color::rgba(0.78, 0.78, 0.80, 1.0);

    // Key surface (the top face you press)
    pub const KEY_SURFACE: Color = Color::rgba(0.96, 0.96, 0.97, 1.0);

    // Top edge highlight (light reflection)
    pub const TOP_HIGHLIGHT: Color = Color::rgba(1.0, 1.0, 1.0, 0.9);

    // Bottom edge (darker, creates depth)
    pub const BOTTOM_EDGE: Color = Color::rgba(0.70, 0.70, 0.72, 1.0);

    // Inner shadow on key surface (subtle recess)
    pub const INNER_SHADOW: Color = Color::rgba(0.88, 0.88, 0.89, 1.0);

    // Text color
    pub const TEXT: Color = Color::rgba(0.20, 0.20, 0.22, 1.0);
}

fn main() -> osd_flash::Result<()> {
    println!("=== Typewriter Demo ===");
    println!("Type any key to see it displayed as a Mac keyboard key.");
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
            show_keyboard_key(&key_label)?;
        }
    }

    Ok(())
}

/// Get display label for a key
fn get_key_label(code: KeyCode, modifiers: KeyModifiers) -> String {
    let base = match code {
        KeyCode::Char(c) => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                format!("^{}", c.to_uppercase())
            } else if c == ' ' {
                "space".to_string()
            } else {
                c.to_uppercase().to_string()
            }
        }
        KeyCode::Enter => "return".to_string(),
        KeyCode::Tab => "tab".to_string(),
        KeyCode::Backspace => "delete".to_string(),
        KeyCode::Delete => "del".to_string(),
        KeyCode::Left => "◀".to_string(),
        KeyCode::Right => "▶".to_string(),
        KeyCode::Up => "▲".to_string(),
        KeyCode::Down => "▼".to_string(),
        KeyCode::Home => "home".to_string(),
        KeyCode::End => "end".to_string(),
        KeyCode::PageUp => "pg up".to_string(),
        KeyCode::PageDown => "pg dn".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        KeyCode::CapsLock => "caps".to_string(),
        _ => String::new(),
    };

    base
}

/// Display a keyboard key on screen
fn show_keyboard_key(label: &str) -> osd_flash::Result<()> {
    // Size based on label length
    let is_wide = label.len() > 2;
    let key_width = if is_wide { 140.0 } else { 80.0 };
    let key_height = 80.0;
    let corner_radius = 8.0;
    let font_size = if is_wide { 18.0 } else { 32.0 };

    // Key dimensions for 3D effect
    let depth = 4.0; // 3D depth of the key
    let inset = 3.0; // Inset for the top surface

    // Calculate text position (center based on actual character count)
    // Approximate character width as 0.6 * font_size for proportional fonts
    let char_width = font_size * 0.6;
    let text_width = label.len() as f64 * char_width;
    let text_x = (key_width - text_width) / 2.0;
    let text_y = (key_height - font_size) / 2.0 + 1.0;

    OsdFlashBuilder::new()
        .dimensions(Size::new(key_width, key_height))
        .position(FlashPosition::Center)
        .build()?
        // Layer 1: Drop shadow (soft shadow beneath the key)
        .draw(StyledShape::new(
            Shape::rounded_rect(
                Rect::from_xywh(3.0, 5.0, key_width - 4.0, key_height - 4.0),
                corner_radius + 2.0,
            ),
            key_colors::DROP_SHADOW,
        ))
        // Layer 2: Key body/base (the "sides" of the 3D key)
        .draw(StyledShape::new(
            Shape::rounded_rect(
                Rect::from_xywh(0.0, 0.0, key_width - 2.0, key_height - 2.0),
                corner_radius,
            ),
            key_colors::KEY_BODY,
        ))
        // Layer 3: Bottom edge (darker edge for depth)
        .draw(StyledShape::new(
            Shape::rounded_rect(
                Rect::from_xywh(1.0, key_height - depth - 4.0, key_width - 4.0, depth + 2.0),
                corner_radius - 2.0,
            ),
            key_colors::BOTTOM_EDGE,
        ))
        // Layer 4: Key surface (the top face you press)
        .draw(StyledShape::new(
            Shape::rounded_rect(
                Rect::from_xywh(inset, inset, key_width - inset * 2.0 - 2.0, key_height - inset * 2.0 - depth),
                corner_radius - 2.0,
            ),
            key_colors::KEY_SURFACE,
        ))
        // Layer 5: Inner shadow (subtle recess on the surface)
        .draw(StyledShape::new(
            Shape::rounded_rect(
                Rect::from_xywh(inset + 2.0, inset + 2.0, key_width - inset * 2.0 - 6.0, 3.0),
                2.0,
            ),
            key_colors::INNER_SHADOW,
        ))
        // Layer 6: Top highlight (light reflection at the top edge)
        .draw(StyledShape::new(
            Shape::rounded_rect(
                Rect::from_xywh(inset + 4.0, inset + 1.0, key_width - inset * 2.0 - 10.0, 2.0),
                1.0,
            ),
            key_colors::TOP_HIGHLIGHT,
        ))
        // Layer 7: Key label
        .draw(StyledText::at(label, text_x, text_y, font_size, key_colors::TEXT))
        .show_for_seconds(0.9)?;

    Ok(())
}
