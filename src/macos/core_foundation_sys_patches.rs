// here are the patches bundled that are submitted here
// https://github.com/servo/core-foundation-rs/pulls?q=is%3Apr+author%3Asassman+
// they live here until the servo team publishes a new version for core-foundation-sys

use core_foundation_sys::number::{CFBooleanRef, CFNumberRef, CFNumberType};

// members of enum CFNumberType
#[allow(non_upper_case_globals)]
pub const kCFNumberSInt32Type: CFNumberType = 3;
#[allow(non_upper_case_globals)]
pub const kCFNumberSInt64Type: CFNumberType = 4;

extern "C" {
    pub fn CFBooleanGetValue(boolean: CFBooleanRef) -> bool;
    pub fn CFNumberGetType(number: CFNumberRef) -> CFNumberType;
}
