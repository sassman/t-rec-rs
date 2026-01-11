//! Test that verifies window visibility after show_for.
//!
//! This test ensures that:
//! 1. Window is visible after calling show()
//! 2. Window is NOT visible after show_for() returns (window is closed)
//!
//! Run with: cargo run -p core-animation --example test_window_visibility

use core_animation::prelude::*;
use std::time::Duration;

fn main() {
    println!("Testing window visibility behavior...\n");

    // Test 1: Basic show/hide cycle
    println!("Test 1: Basic show/hide cycle");
    {
        let window = WindowBuilder::new()
            .title("Visibility Test 1")
            .size(200.0, 100.0)
            .centered()
            .background_color(Color::rgb(0.2, 0.4, 0.8))
            .build();

        // Window should not be visible before show()
        assert!(
            !window.is_visible(),
            "FAIL: Window should not be visible before show()"
        );
        println!("  [PASS] Window is not visible before show()");

        // Window should be visible after show()
        window.show();
        assert!(
            window.is_visible(),
            "FAIL: Window should be visible after show()"
        );
        println!("  [PASS] Window is visible after show()");

        // Window should not be visible after hide()
        window.hide();
        assert!(
            !window.is_visible(),
            "FAIL: Window should not be visible after hide()"
        );
        println!("  [PASS] Window is not visible after hide()");

        // Verify show/hide cycle works multiple times
        for i in 1..=3 {
            window.show();
            assert!(
                window.is_visible(),
                "FAIL: Window should be visible in cycle {}",
                i
            );
            window.hide();
            assert!(
                !window.is_visible(),
                "FAIL: Window should be hidden in cycle {}",
                i
            );
        }
        println!("  [PASS] Show/hide cycle works correctly (3 cycles)");
    }

    // Test 2: show_for() should hide and close the window
    println!("\nTest 2: show_for() closes window after duration");
    {
        let window = WindowBuilder::new()
            .title("Visibility Test 2")
            .size(200.0, 100.0)
            .centered()
            .background_color(Color::rgb(0.8, 0.2, 0.4))
            .build();

        println!("  Showing window for 500ms...");
        window.show_for(Duration::from_millis(500));

        // Window should NOT be visible after show_for() returns
        assert!(
            !window.is_visible(),
            "FAIL: Window should NOT be visible after show_for() returns"
        );
        println!("  [PASS] Window is NOT visible after show_for() returns");
    }

    // Test 3: Multiple show_for() calls with separate windows
    println!("\nTest 3: Multiple show_for() calls");
    for i in 1..=3 {
        let window = WindowBuilder::new()
            .title(format!("Flash {}", i))
            .size(100.0, 100.0)
            .centered()
            .background_color(Color::rgb(0.2, 0.8, 0.2))
            .build();

        window.show_for(Duration::from_millis(200));

        assert!(
            !window.is_visible(),
            "FAIL: Window {} should NOT be visible after show_for()",
            i
        );
    }
    println!("  [PASS] All 3 flash windows closed correctly");

    println!("\n=== All tests passed! ===");
}
