// here are the patches bundled that are submitted here
// https://github.com/servo/core-foundation-rs/pulls?q=is%3Apr+author%3Asassman+
// they live here until the servo team publishes a new version for core-foundation-sys

use core_foundation_sys::number::CFNumberType;

// members of enum CFNumberType
#[allow(non_upper_case_globals, dead_code)]
pub const kCFNumberSInt32Type: CFNumberType = 3;
#[allow(non_upper_case_globals, dead_code)]
pub const kCFNumberSInt64Type: CFNumberType = 4;

/// Core Graphics window level type.
pub type CGWindowLevel = i32;

// Number of window levels reserved by Apple for internal use
#[allow(non_upper_case_globals)]
pub const kCGNumReservedWindowLevels: CGWindowLevel = 16;
#[allow(non_upper_case_globals)]
pub const kCGNumReservedBaseWindowLevels: CGWindowLevel = 5;

// Base and boundary levels
#[allow(non_upper_case_globals)]
pub const kCGBaseWindowLevel: CGWindowLevel = i32::MIN;
#[allow(non_upper_case_globals)]
pub const kCGMinimumWindowLevel: CGWindowLevel =
    kCGBaseWindowLevel + kCGNumReservedBaseWindowLevels;
#[allow(non_upper_case_globals)]
pub const kCGMaximumWindowLevel: CGWindowLevel = i32::MAX - kCGNumReservedWindowLevels;

// Standard window levels (in ascending z-order)
#[allow(non_upper_case_globals, dead_code)]
pub const kCGDesktopWindowLevel: CGWindowLevel = kCGMinimumWindowLevel + 20;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGDesktopIconWindowLevel: CGWindowLevel = kCGDesktopWindowLevel + 20;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGBackstopMenuLevel: CGWindowLevel = -20;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGNormalWindowLevel: CGWindowLevel = 0;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGFloatingWindowLevel: CGWindowLevel = 3;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGTornOffMenuWindowLevel: CGWindowLevel = 3;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGModalPanelWindowLevel: CGWindowLevel = 8;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGUtilityWindowLevel: CGWindowLevel = 19;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGDockWindowLevel: CGWindowLevel = 20;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGMainMenuWindowLevel: CGWindowLevel = 24;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGStatusWindowLevel: CGWindowLevel = 25;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGPopUpMenuWindowLevel: CGWindowLevel = 101;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGOverlayWindowLevel: CGWindowLevel = 102;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGHelpWindowLevel: CGWindowLevel = 200;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGDraggingWindowLevel: CGWindowLevel = 500;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGScreenSaverWindowLevel: CGWindowLevel = 1000;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGAssistiveTechHighWindowLevel: CGWindowLevel = 1500;
#[allow(non_upper_case_globals, dead_code)]
pub const kCGCursorWindowLevel: CGWindowLevel = kCGMaximumWindowLevel - 1;
