//! CFRunLoop sequential window test.
//!
//! This example tests sequential window rendering.
//! It displays multiple windows one after another at different positions.
//!
//! Run with: cargo run -p osd-flash --example cfrunloop_only

use osd_flash::prelude::*;

fn main() -> osd_flash::Result<()> {
    println!("=== Sequential Window Test ===\n");
    println!("This test displays multiple windows sequentially.\n");

    let size = 80.0;

    // Window 1: Top-right (recording style)
    println!("Window 1: Recording indicator (top-right)");
    OsdBuilder::new()
        .size(size)
        .position(Position::TopRight)
        .margin(30.0)
        .level(WindowLevel::AboveAll)
        .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
        .corner_radius(14.0)
        .layer("glow", |l| {
            l.circle(50.0)
                .center()
                .fill(Color::rgba(1.0, 0.2, 0.2, 0.3))
                .animate(Animation::pulse_range(0.9, 1.1))
        })
        .layer("dot", |l| {
            l.circle(30.0)
                .center()
                .fill(Color::RED)
                .animate(Animation::pulse())
        })
        .show_for(1500.millis())?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Window 2: Top-left (camera style)
    println!("Window 2: Camera indicator (top-left)");
    OsdBuilder::new()
        .size(size)
        .position(Position::TopLeft)
        .margin(30.0)
        .level(WindowLevel::AboveAll)
        .background(Color::rgba(0.15, 0.45, 0.9, 0.92))
        .corner_radius(14.0)
        .layer("body", |l| {
            l.ellipse(50.0, 35.0).center().fill(Color::WHITE)
        })
        .layer("lens", |l| {
            l.circle(22.0)
                .center()
                .fill(Color::rgba(0.3, 0.5, 0.8, 1.0))
        })
        .show_for(1500.millis())?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Window 3: Bottom-right (recording style)
    println!("Window 3: Recording indicator (bottom-right)");
    OsdBuilder::new()
        .size(size)
        .position(Position::BottomRight)
        .margin(30.0)
        .level(WindowLevel::AboveAll)
        .background(Color::rgba(0.1, 0.1, 0.1, 0.9))
        .corner_radius(14.0)
        .layer("glow", |l| {
            l.circle(50.0)
                .center()
                .fill(Color::rgba(1.0, 0.2, 0.2, 0.3))
                .animate(Animation::pulse_range(0.9, 1.1))
        })
        .layer("dot", |l| {
            l.circle(30.0)
                .center()
                .fill(Color::RED)
                .animate(Animation::pulse())
        })
        .show_for(1500.millis())?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    // Window 4: Center (camera style)
    println!("Window 4: Camera indicator (center)");
    OsdBuilder::new()
        .size(size)
        .position(Position::Center)
        .level(WindowLevel::AboveAll)
        .background(Color::rgba(0.15, 0.45, 0.9, 0.92))
        .corner_radius(14.0)
        .layer("body", |l| {
            l.ellipse(50.0, 35.0).center().fill(Color::WHITE)
        })
        .layer("lens", |l| {
            l.circle(22.0)
                .center()
                .fill(Color::rgba(0.3, 0.5, 0.8, 1.0))
        })
        .show_for(1500.millis())?;

    println!("\n=== Test Complete ===");
    println!("All 4 windows should have appeared sequentially.");

    Ok(())
}
