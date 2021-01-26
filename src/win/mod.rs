winrt::import!(
    dependencies
        "os"
    modules
        "windows.graphics"
        "windows.graphics.capture"
        "windows.graphics.directx"
        "windows.graphics.directx.direct3d11"
);

use std::sync::{Arc, Mutex};

use winapi::shared::minwindef::{BOOL, FALSE, INT, LPARAM, MAX_PATH, TRUE};
// use winapi::shared::ntdef::LONG;
use winapi::shared::windef::{HWND, RECT};
use winapi::um::winnt::WCHAR;
use winapi::um::winuser::{
    EnumWindows, GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, GetWindowTextLengthW,
    GetWindowTextW, GWL_STYLE, WS_SYSMENU, WS_VISIBLE,
};

use crate::common::identify_transparency::identify_transparency;
use crate::{ImageOnHeap, Margin, PlatformApi, Result, WindowId, WindowList};

mod capture;
mod d3d;
mod encoder;
mod snapshot;

pub const DEFAULT_SHELL: &str = "cmd.exe";

struct CaptureTarget {
    hwnd: HWND,
    // capture_item: crate::windows::graphics::capture::GraphicsCaptureItem,
    capture_session: snapshot::CaptureSnapshot,
    // device: IDirect3DDevice,
}

unsafe impl Send for CaptureTarget {}

pub fn enumerate_windows<F>(mut callback: F)
where
    F: FnMut(HWND) -> bool,
{
    use core::mem;
    let mut trait_obj: &mut dyn FnMut(HWND) -> bool = &mut callback;
    let closure_pointer_pointer: *mut core::ffi::c_void = unsafe { mem::transmute(&mut trait_obj) };

    let lparam = closure_pointer_pointer as LPARAM;
    unsafe { EnumWindows(Some(enumerate_callback), lparam) };
}

// pub fn enumerate_child_windows<F>(hwnd: HWND, mut callback: F)
// where
//     F: FnMut(HWND) -> bool,
// {
//     use core::mem;
//     let mut trait_obj: &mut dyn FnMut(HWND) -> bool = &mut callback;
//     let closure_pointer_pointer: *mut c_void = unsafe { mem::transmute(&mut trait_obj) };

//     let lparam = closure_pointer_pointer as LPARAM;
//     unsafe { EnumChildWindows(hwnd, Some(enumerate_callback), lparam) };
// }

unsafe extern "system" fn enumerate_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    use core::mem;
    let closure: &mut &mut dyn FnMut(HWND) -> bool =
        mem::transmute(lparam as *mut core::ffi::c_void);
    if closure(hwnd) {
        TRUE
    } else {
        FALSE
    }
}

pub struct WinApi {
    cap: Arc<Mutex<Option<CaptureTarget>>>,
    margin: Option<Margin>,
}

impl WinApi {
    pub fn new() -> Result<Self> {
        // let factory =
        //     winrt::activation::factory::<GraphicsCaptureItem, winrt::IActivationFactory>()?;
        // cap = factory.try_into()?;

        Ok(WinApi {
            cap: Arc::new(Mutex::new(None)),
            margin: None,
        })
    }
}

pub fn setup() -> Result<Box<dyn PlatformApi>> {
    Ok(Box::new(WinApi::new()?))
}

impl PlatformApi for WinApi {
    /// 1. it does check for the screenshot
    /// 2. it checks for transparent margins and configures the api
    ///     to cut them away in further screenshots
    fn calibrate(&mut self, window_id: WindowId) -> Result<()> {
        let image = self.capture_window_screenshot(window_id)?;
        self.margin = identify_transparency(*image)?;

        Ok(())
    }

    fn window_list(&self) -> Result<WindowList> {
        use std::ffi::OsString;
        use std::os::windows::prelude::*;
        let mut wins = vec![];
        enumerate_windows(|hwnd| {
            // Skip invisible windows
            let style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) } as u32;
            if ((style & WS_VISIBLE) != WS_VISIBLE) || ((style & WS_SYSMENU) != WS_SYSMENU) {
                return true;
            }

            // Skip empty window titles
            let length = unsafe { GetWindowTextLengthW(hwnd) } as usize;
            if length == 0 {
                return true;
            }

            // Retrieve the title. Add 1 for the null terminator.
            let mut title = [0 as WCHAR; MAX_PATH as usize];
            unsafe { GetWindowTextW(hwnd, title.as_mut_ptr(), 1 + length as INT) };

            // Convert the title to a UTF-8 string
            let string = OsString::from_wide(&title[0..length]);
            if let Ok(s) = string.into_string() {
                let mut rect = core::mem::MaybeUninit::<RECT>::uninit();
                let name = if TRUE == unsafe { GetWindowRect(hwnd, rect.as_mut_ptr() as _) } {
                    let rect = unsafe { rect.assume_init() };
                    format!(
                        "{} ({}x{})",
                        s,
                        rect.right - rect.left,
                        rect.bottom - rect.top
                    )
                } else {
                    s
                };
                wins.push((Some(name), hwnd as WindowId));
            }
            true
        });
        Ok(wins)
    }

    fn capture_window_screenshot(&self, window_id: WindowId) -> Result<ImageOnHeap> {
        let hwnd = window_id as HWND;
        let mut mtx = self.cap.lock().unwrap();
        match &*mtx {
            None => {
                let cap = capture::create_capture_item_for_window(hwnd).unwrap();
                let d3d_device = d3d::D3D11Device::new().unwrap();
                let device = d3d_device.to_direct3d_device().unwrap();
                let session = snapshot::CaptureSnapshot::create_session(&device, &cap).unwrap();
                *mtx = Some(CaptureTarget {
                    hwnd,
                    capture_session: session,
                });
            }
            Some(ct) => {
                if ct.hwnd != hwnd {
                    let cap = capture::create_capture_item_for_window(hwnd).unwrap();
                    let d3d_device = d3d::D3D11Device::new().unwrap();
                    let device = d3d_device.to_direct3d_device().unwrap();
                    let session = snapshot::CaptureSnapshot::create_session(&device, &cap).unwrap();

                    *mtx = Some(CaptureTarget {
                        hwnd,
                        capture_session: session,
                    });
                }
            }
        }
        let ct = mtx.as_ref().unwrap();
        let buffer = ct.capture_session.snapshot().unwrap();
        Ok(ImageOnHeap::new(buffer))
    }

    fn get_active_window(&self) -> Result<WindowId> {
        Ok(unsafe { GetForegroundWindow() } as WindowId)
    }
}
