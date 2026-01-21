//! Default values shared between CLI and config
//!
//! These constants are the single source of truth for all default values.
//! They are used in `ProfileSettings` accessor methods to apply defaults after config merging.
//!
//! Note: The help text in `cli.rs` also shows these defaults (e.g., "[default: 4]").
//! Unfortunately, these must be kept in sync manually because Rust's `concat!` macro
//! only works with literal strings, not const references.

pub const FPS: u8 = 4;
pub const DECOR: &str = "none";
pub const BG: &str = "transparent";
pub const WALLPAPER_PADDING: u32 = 60;
pub const IDLE_PAUSE: &str = "3s";
pub const OUTPUT: &str = "t-rec";
