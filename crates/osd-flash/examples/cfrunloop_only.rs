//! CFRunLoop sequential window test.
//!
//! This example tests sequential window rendering using CFRunLoop.
//! It displays multiple windows one after another to verify the run loop works correctly.
//!
//! Run with: cargo run -p osd-flash --example cfrunloop_only

use osd_flash::backends::{SkylightCanvas, SkylightWindowBuilder, WindowLevel};
use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    println!("=== CFRunLoop Sequential Window Test ===\n");
    println!("This test displays multiple windows sequentially.\n");

    // Window 1
    println!("Window 1: Recording icon (top-right)");
    show_window(FlashPosition::TopRight, true)?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Window 2
    println!("Window 2: Camera icon (top-left)");
    show_window(FlashPosition::TopLeft, false)?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Window 3
    println!("Window 3: Recording icon (bottom-right)");
    show_window(FlashPosition::BottomRight, true)?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Window 4
    println!("Window 4: Camera icon (center)");
    show_window(FlashPosition::Center, false)?;

    println!("\n=== Test Complete ===");
    println!("All 4 windows should have appeared sequentially.");

    Ok(())
}

fn show_window(position: FlashPosition, use_recording_icon: bool) -> osd_flash::Result<()> {
    let frame = get_frame_for_position(position);
    let size = 80.0;

    // Create window using the builder
    let mut window = SkylightWindowBuilder::new()
        .frame(frame)
        .level(WindowLevel::AboveAll)
        .build()?;

    // Draw the icon
    let mut canvas = unsafe { SkylightCanvas::new(window.context_ptr(), window.size()) };

    if use_recording_icon {
        RecordingIcon::new(size).build().draw(&mut canvas);
    } else {
        CameraIcon::new(size).build().draw(&mut canvas);
    }

    // Show for 1.5 seconds
    window.show(1.5)?;

    println!("  Done!");
    Ok(())
}

fn get_frame_for_position(position: FlashPosition) -> Rect {
    use core_graphics::display::{CGDisplayBounds, CGMainDisplayID};

    let display_bounds = unsafe {
        let id = CGMainDisplayID();
        let b = CGDisplayBounds(id);
        Rect::from_xywh(b.origin.x, b.origin.y, b.size.width, b.size.height)
    };

    let size = 80.0;
    let margin = 30.0;

    let (x, y) = match position {
        FlashPosition::TopRight => (
            display_bounds.origin.x + display_bounds.size.width - size - margin,
            display_bounds.origin.y + margin + 25.0,
        ),
        FlashPosition::TopLeft => (
            display_bounds.origin.x + margin,
            display_bounds.origin.y + margin + 25.0,
        ),
        FlashPosition::BottomRight => (
            display_bounds.origin.x + display_bounds.size.width - size - margin,
            display_bounds.origin.y + display_bounds.size.height - size - margin,
        ),
        FlashPosition::BottomLeft => (
            display_bounds.origin.x + margin,
            display_bounds.origin.y + display_bounds.size.height - size - margin,
        ),
        FlashPosition::Center => (
            display_bounds.origin.x + (display_bounds.size.width - size) / 2.0,
            display_bounds.origin.y + (display_bounds.size.height - size) / 2.0,
        ),
        FlashPosition::Custom { x, y } => (x, y),
    };

    // Scale for Retina displays
    Rect::from_xywh(x / 2.0, y / 2.0, size, size).rounded()
}
