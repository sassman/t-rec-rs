winrt::import!(
    dependencies
        "os"
    modules
        "windows.graphics"
        "windows.graphics.capture"
        "windows.graphics.directx"
        "windows.graphics.directx.direct3d11"
);

use crate::{ImageOnHeap, PlatformApi, Result, WindowId, WindowList};

use winapi::shared::minwindef::{BOOL, FALSE, INT, LPARAM, MAX_PATH, TRUE};
// use winapi::shared::ntdef::LONG;
use winapi::shared::windef::{HWND, RECT};
use winapi::um::winnt::WCHAR;
use winapi::um::winuser::{
    EnumWindows, GetForegroundWindow, GetWindowLongPtrW, GetWindowRect, GetWindowTextLengthW,
    GetWindowTextW, GWL_STYLE, WS_SYSMENU, WS_VISIBLE,
};

mod capture;
mod d3d;
mod encoder;
mod snapshot;

pub const DEFAULT_SHELL: &str = "cmd.exe";

#[derive(Debug)]
pub struct Margin {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

struct CaptureTarget {
    hwnd: HWND,
    // capture_item: crate::windows::graphics::capture::GraphicsCaptureItem,
    capture_session: snapshot::CaptureSnapshot,
    // device: IDirect3DDevice,
}

unsafe impl Send for CaptureTarget {}

// impl Margin {
//     pub fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
//         Self {
//             top,
//             right,
//             bottom,
//             left,
//         }
//     }

//     pub fn new_equal(margin: u16) -> Self {
//         Self::new(margin, margin, margin, margin)
//     }

//     pub fn zero() -> Self {
//         Self::new_equal(0)
//     }

//     pub fn is_zero(&self) -> bool {
//         self.top == 0
//             && self.right == self.left
//             && self.left == self.bottom
//             && self.bottom == self.top
//     }
// }

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

use std::sync::{Arc, Mutex};

pub struct WinApi {
    cap: Arc<Mutex<Option<CaptureTarget>>>,
}

impl WinApi {
    pub fn new() -> Result<Self> {
        // let factory =
        //     winrt::activation::factory::<GraphicsCaptureItem, winrt::IActivationFactory>()?;
        // cap = factory.try_into()?;

        Ok(WinApi {
            cap: Arc::new(Mutex::new(None)),
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
    fn calibrate(&mut self, _window_id: WindowId) -> Result<()> {
        // let image = self.capture_window_screenshot(window_id)?;
        // let image: View<_, Bgra<u8>> = image.as_view()?;
        // let (width, height) = image.dimensions();
        // let half_width = width / 2;
        // let half_height = height / 2;

        // let mut margin = Margin::zero();
        // // identify top margin
        // for y in 0..half_height {
        //     let Bgra([_, _, _, a]) = image.get_pixel(half_width, y);
        //     if a == 0xff {
        //         // the end of the transparent area
        //         margin.top = y as u16;
        //         dbg!(margin.top);
        //         break;
        //     }
        // }
        // // identify bottom margin
        // for y in (half_height..height).rev() {
        //     let Bgra([_, _, _, a]) = image.get_pixel(half_width, y);
        //     if a == 0xff {
        //         // the end of the transparent area
        //         margin.bottom = (height - y - 1) as u16;
        //         dbg!(margin.bottom);
        //         break;
        //     }
        // }
        // // identify left margin
        // for x in 0..half_width {
        //     let Bgra([_, _, _, a]) = image.get_pixel(x, half_height);
        //     if a == 0xff {
        //         // the end of the transparent area
        //         margin.left = x as u16;
        //         dbg!(margin.left);
        //         break;
        //     }
        // }
        // // identify right margin
        // for x in (half_width..width).rev() {
        //     let Bgra([_, _, _, a]) = image.get_pixel(x, half_height);
        //     if a == 0xff {
        //         // the end of the transparent area
        //         margin.right = (width - x - 1) as u16;
        //         dbg!(margin.right);
        //         break;
        //     }
        // }
        // self.margin = if margin.is_zero() { None } else { Some(margin) };

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
