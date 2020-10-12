mod core_foundation_sys_patches;
mod screenshot;
mod window_id;

pub use screenshot::capture_window_screenshot;
pub use window_id::{get_window_id_for, ls_win};
