//! CGWindow constants for objc2-core-graphics.
//!
//! These constants are defined here as the objc2-core-graphics crate
//! uses newtype wrappers for the option types.

use objc2_core_graphics::{CGWindowImageOption, CGWindowListOption};

// CGWindowListOption constants
pub const K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY: CGWindowListOption = CGWindowListOption(1 << 0);
pub const K_CG_WINDOW_LIST_OPTION_INCLUDING_WINDOW: CGWindowListOption = CGWindowListOption(1 << 3);
pub const K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS: CGWindowListOption =
    CGWindowListOption(1 << 4);

// CGWindowImageOption constants
pub const K_CG_WINDOW_IMAGE_BOUNDS_IGNORE_FRAMING: CGWindowImageOption =
    CGWindowImageOption(1 << 0);
pub const K_CG_WINDOW_IMAGE_SHOULD_BE_OPAQUE: CGWindowImageOption = CGWindowImageOption(1 << 1);
pub const K_CG_WINDOW_IMAGE_NOMINAL_RESOLUTION: CGWindowImageOption = CGWindowImageOption(1 << 4);

// Null window ID
pub const K_CG_NULL_WINDOW_ID: u32 = 0;
