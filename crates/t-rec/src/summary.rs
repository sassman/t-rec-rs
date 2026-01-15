use crate::{common::utils::print_tree_list, config::ProfileSettings};

/// Prints a visual summary of the recording settings.
///
/// Only displays settings that are relevant or differ from defaults.
pub fn print_recording_summary(settings: &ProfileSettings, frame_count: usize) {
    println!();
    println!("📋 Recording summary");

    let mut lines: Vec<String> = Vec::new();

    // FPS (always show - it's a key setting for this feature)
    lines.push(format!("fps: {}", settings.fps()));

    // Idle pause
    lines.push(format!("idle-pause: {}", settings.idle_pause()));

    // Wallpaper (only if set)
    if let Some(ref wp) = settings.wallpaper {
        let padding = settings.wallpaper_padding();
        lines.push(format!("wallpaper: {} (padding: {}px)", wp, padding));
    }

    // Decor (only if not default)
    if settings.decor() != "none" {
        lines.push(format!("decor: {}", settings.decor()));
    }

    // Natural mode (only if enabled)
    if settings.natural() {
        lines.push("natural: enabled".to_string());
    }

    // Frame count
    lines.push(format!("frames: {}", frame_count));

    // Output file
    lines.push(format!("output: {}", settings.output()));

    print_tree_list(&lines);
}
