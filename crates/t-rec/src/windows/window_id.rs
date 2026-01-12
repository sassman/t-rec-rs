use crate::WindowList;
use anyhow::{anyhow, Result};
use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

/// Returns a list of all visible windows with their names and IDs.
///
/// Uses the win-screenshot crate to enumerate windows.
pub fn window_list() -> Result<WindowList> {
    let windows = win_screenshot::utils::window_list()
        .map_err(|e| anyhow!("Failed to enumerate windows: {:?}", e))?;

    let win_list = windows
        .into_iter()
        .map(|w| (Some(w.window_name), w.hwnd as u64))
        .collect();

    Ok(win_list)
}

/// Returns the window ID (HWND) of the currently active/foreground window.
///
/// Uses the Win32 GetForegroundWindow API.
pub fn get_foreground_window() -> Result<u64> {
    let hwnd = unsafe { GetForegroundWindow() };

    if hwnd.0.is_null() {
        return Err(anyhow!(
            r#"Cannot determine the active window.
 - No window is currently in the foreground
 - Use `t-rec -l` to list all windows with their IDs
 - Or specify a window ID explicitly with `t-rec -w <window_id>`
"#
        ));
    }

    Ok(hwnd.0 as u64)
}
