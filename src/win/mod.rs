use crate::{ImageOnHeap, PlatformApi, Result, WindowId, WindowList};
use image::flat::{SampleLayout, View};
use image::{Bgra, ColorType, FlatSamples, GenericImageView};
use std::{convert::TryInto, ops::DerefMut};

use winapi::shared::minwindef::{BOOL, FALSE, INT, LPARAM, MAX_PATH, TRUE};
// use winapi::shared::ntdef::LONG;
use winapi::shared::windef::{HDC, HGDIOBJ, HWND, LPRECT, RECT};
use winapi::um::wingdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteObject, GetDIBits, GetObjectW,
    SelectObject, SetStretchBltMode, StretchBlt, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS, HALFTONE, SRCCOPY,
};
use winapi::um::winnt::WCHAR;
use winapi::um::winuser::{
    EnumChildWindows, EnumWindows, FindWindowW, GetClientRect, GetDC, GetDesktopWindow,
    GetSystemMetrics, GetWindowLongPtrW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, MonitorFromWindow, ReleaseDC, GWL_STYLE, MONITOR_DEFAULTTOPRIMARY,
    SM_CXSCREEN, SM_CYSCREEN, WS_SYSMENU, WS_VISIBLE,
};

use std::{mem::MaybeUninit, os::raw::c_void};

pub const DEFAULT_SHELL: &str = "cmd.exe";

#[derive(Debug)]
pub struct Margin {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl Margin {
    pub fn new(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn new_equal(margin: u16) -> Self {
        Self::new(margin, margin, margin, margin)
    }

    pub fn zero() -> Self {
        Self::new_equal(0)
    }

    pub fn is_zero(&self) -> bool {
        self.top == 0
            && self.right == self.left
            && self.left == self.bottom
            && self.bottom == self.top
    }
}

pub fn enumerate_windows<F>(mut callback: F)
where
    F: FnMut(HWND) -> bool,
{
    use core::mem;
    let mut trait_obj: &mut FnMut(HWND) -> bool = &mut callback;
    let closure_pointer_pointer: *mut c_void = unsafe { mem::transmute(&mut trait_obj) };

    let lparam = closure_pointer_pointer as LPARAM;
    unsafe { EnumWindows(Some(enumerate_callback), lparam) };
}

pub fn enumerate_child_windows<F>(hwnd: HWND, mut callback: F)
where
    F: FnMut(HWND) -> bool,
{
    use core::mem;
    let mut trait_obj: &mut FnMut(HWND) -> bool = &mut callback;
    let closure_pointer_pointer: *mut c_void = unsafe { mem::transmute(&mut trait_obj) };

    let lparam = closure_pointer_pointer as LPARAM;
    unsafe { EnumChildWindows(hwnd, Some(enumerate_callback), lparam) };
}

unsafe extern "system" fn enumerate_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    use core::mem;
    let closure: &mut &mut FnMut(HWND) -> bool = mem::transmute(lparam as *mut c_void);
    if closure(hwnd) {
        TRUE
    } else {
        FALSE
    }
}

pub struct WinApi {
    // conn: RustConnection<DefaultStream>,
    // screen_num: usize,
    // atoms: Atoms,
    margin: Option<Margin>,
}

impl WinApi {
    pub fn new() -> Result<Self> {
        Ok(WinApi { margin: None })
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
        let window_id = 198478;
        let hwnd = window_id as HWND;

        // let (_, _, mut width, mut height) = self.get_window_geometry(&window_id)?;
        // let (mut x, mut y) = (0_i16, 0_i16);
        // if self.margin.is_some() {
        //     let margin = self.margin.as_ref().unwrap();
        //     width -= margin.left + margin.right;
        //     height -= margin.top + margin.bottom;
        //     x = margin.left as i16;
        //     y = margin.top as i16;
        // }
        // let image = self
        //     .conn
        //     // NOTE: x and y are not the absolute coordinates but relative to the windows dimensions, that is why 0, 0
        //     .get_image(
        //         ImageFormat::ZPixmap,
        //         window_id as Drawable,
        //         x,
        //         y,
        //         width,
        //         height,
        //         !0,
        //     )?
        //     .reply()
        //     .context(format!(
        //         "Cannot fetch the image data for window {}",
        //         window_id
        //     ))?;

        // println!("Buffer data: {:?}", raw_data);
        let buffer = capture_window_screenshot_1(hwnd)?;

        // // NOTE: in this case the alpha channel is 0, but should be set to 0xff
        // if image.depth == 24 {
        //     // the index into the alpha channel
        //     let mut i = 3;
        //     let len = buffer.samples.len();
        //     while i < len {
        //         let alpha = buffer.samples.get_mut(i).unwrap();
        //         if alpha == &0 {
        //             *alpha = 0xff;
        //         } else {
        //             // NOTE: the assumption here is, if one pixel is fine, then all might be fine :)
        //             break;
        //         }

        //         // going one pixel further, still pointing to the alpha channel
        //         i += buffer.layout.width_stride;
        //     }
        // }

        Ok(ImageOnHeap::new(buffer))
    }

    fn get_active_window(&self) -> Result<WindowId> {
        println!("Getting active window");
        // let screen = self.screen();
        // let conn = &self.conn;
        // let atoms = &self.atoms;
        // let prop = conn
        //     .get_property(
        //         false,
        //         screen.root,
        //         atoms._NET_ACTIVE_WINDOW,
        //         AtomEnum::WINDOW,
        //         0,
        //         u32::MAX,
        //     )?
        //     .reply()?;
        // let window = prop.value32().unwrap().next().unwrap();

        Ok(0 as WindowId)
    }
}

struct DeviceContext {
    hdc: HDC,
    hwnd: HWND,
}

impl DeviceContext {
    pub fn get_dc(hwnd: Option<HWND>) -> Result<DeviceContext> {
        let hwnd = hwnd.unwrap_or(0 as HWND);
        let hdc = unsafe { GetDC(hwnd) };
        if hdc.is_null() {
            return Err(anyhow::Error::msg("Unable to get device context"));
        }
        Ok(DeviceContext { hdc, hwnd })
    }
    pub fn as_mut_ptr(&mut self) -> HDC {
        self.hdc
    }
}

impl Drop for DeviceContext {
    fn drop(&mut self) {
        unsafe { ReleaseDC(self.hwnd, self.hdc) };
    }
}

/// Capture a screenshot of the specified window. This function works
/// by capturing the entire screen and then cropping the coordinates of the
/// specified window.
fn capture_window_screenshot_1(hwnd: HWND) -> Result<FlatSamples<Vec<u8>>> {
    println!("Capturing window");

    // Retrieve the handle to a display device context for the client
    // area of the window.
    let mut screen = DeviceContext::get_dc(None)?; //unsafe { GetDC(0 as HWND) };
    let mut window = DeviceContext::get_dc(Some(hwnd))?; //unsafe { GetDC(hwnd) };

    // Create a compatible DC, which is used in a BitBlt from the window DC.
    let mem_dc = unsafe { CreateCompatibleDC(window.as_mut_ptr()) };
    if mem_dc.is_null() {
        return Err(anyhow::Error::msg(
            "Unable to create drawing context for window",
        ));
    }

    // Figure out the dimensions and position of the window of interest.
    let mut client_rect = core::mem::MaybeUninit::<RECT>::uninit();
    unsafe { GetClientRect(hwnd, client_rect.as_mut_ptr() as LPRECT) };
    let client_rect = unsafe { client_rect.assume_init() };

    // This is the best stretch mode.
    unsafe { SetStretchBltMode(window.as_mut_ptr(), HALFTONE) };

    // The source DC is the entire screen, and the destination DC is the current window (HWND).
    if unsafe {
        StretchBlt(
            mem_dc,
            0,
            0,
            client_rect.right,
            client_rect.bottom,
            screen.as_mut_ptr(),
            0,
            0,
            GetSystemMetrics(SM_CXSCREEN),
            GetSystemMetrics(SM_CYSCREEN),
            SRCCOPY,
        ) == FALSE
    } {
        // Clean up
        // ReleaseDC(NULL, screen);
        // ReleaseDC(hwnd, window);
        // MessageBox(hWnd, L"CreateCompatibleDC has failed", L"Failed", MB_OK);
        // goto done;
        return Err(anyhow::Error::msg("StretchBlit failed"));
    }

    // Create a compatible bitmap from the Window DC.
    let screen_bmp = unsafe {
        CreateCompatibleBitmap(
            window.as_mut_ptr(),
            client_rect.right - client_rect.left,
            client_rect.bottom - client_rect.top,
        )
    };

    if screen_bmp.is_null() {
        // Clean up
        // DeleteObject(screen);
        // ReleaseDC(NULL, screen);
        // ReleaseDC(hwnd, window);
        // MessageBox(hWnd, L"CreateCompatibleDC has failed", L"Failed", MB_OK);
        // goto done;
        return Err(anyhow::Error::msg("Unable to create screen bitmap"));
    }

    // Select the compatible bitmap into the compatible memory DC.
    unsafe { SelectObject(mem_dc, screen_bmp as HGDIOBJ) };

    // Bit block transfer into our compatible memory DC.
    if unsafe {
        BitBlt(
            mem_dc,
            0,
            0,
            client_rect.right - client_rect.left,
            client_rect.bottom - client_rect.top,
            window.as_mut_ptr(),
            0,
            0,
            SRCCOPY,
        )
    } == FALSE
    {
        // // Clean up
        // unsafe { DeleteObject(mem_dc as _) };
        // unsafe { DeleteObject(screen as _) };
        // unsafe { ReleaseDC(0 as HWND, screen) };
        // unsafe { ReleaseDC(hwnd, window.as_mut_ptr()) };
        // goto done;
        return Err(anyhow::Error::msg("BitBlt has failed"));
    }

    // Get the BITMAP from the HBITMAP.
    let mut bmp_screen = core::mem::MaybeUninit::<BITMAP>::uninit();
    unsafe {
        GetObjectW(
            screen_bmp as *mut _,
            core::mem::size_of::<BITMAP>().try_into().unwrap(),
            bmp_screen.as_mut_ptr() as *mut _,
        )
    };
    let bmp_screen = unsafe { bmp_screen.assume_init() };

    let mut bi = BITMAPINFOHEADER {
        biSize: core::mem::size_of::<BITMAPINFOHEADER>().try_into().unwrap(),
        biWidth: bmp_screen.bmWidth,
        biHeight: bmp_screen.bmHeight,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    };

    println!(
        "Width: {}  Height: {}  Bitcount: {}",
        bmp_screen.bmWidth, bmp_screen.bmHeight, bi.biBitCount
    );
    let mut raw_data = vec![
        0u8;
        (((bmp_screen.bmWidth as usize) * (bi.biBitCount as usize) + 31) / 32)
            * 4
            * (bmp_screen.bmHeight as usize)
    ];

    // Gets the "bits" from the bitmap, and copies them into a buffer
    // that's pointed to by lpbitmap.
    unsafe {
        GetDIBits(
            window.as_mut_ptr(),
            screen_bmp,
            0,
            bmp_screen.bmHeight as _,
            raw_data.as_mut_ptr() as *mut _,
            &mut bi as *mut BITMAPINFOHEADER as *mut BITMAPINFO,
            DIB_RGB_COLORS,
        )
    };

    let color = ColorType::Bgra8;
    let channels = 4;

    Ok(FlatSamples {
        samples: raw_data,
        layout: SampleLayout::row_major_packed(
            channels,
            bmp_screen.bmWidth as u32,
            bmp_screen.bmHeight as u32,
        ),
        color_hint: Some(color),
    })
}

// fn capture_window_screenshot_2(hwnd: HWND) -> Result<FlatSamples<Vec<u8>>>{

//     // let window: Vec<u16> = OsStr::new(name).encode_wide().chain(once(0)).collect();

//     // let hwnd = unsafe { FindWindowW(null_mut(), window.as_ptr()) };

//     // if hwnd != null_mut() {
//         println!("Window found");

//         let mut my_rect = unsafe { core::mem::zeroed::<RECT>() };
//         let _client_rect = unsafe { GetClientRect(hwnd, &mut my_rect) };
//         let w = my_rect.right - my_rect.left;
//         let h = my_rect.bottom - my_rect.top;

//         let hwnd_dc = unsafe { GetWindowDC(hwnd) };
//         let mem_dc = unsafe { CreateCompatibleDC(hwnd_dc) };
//         let bmp = unsafe { CreateCompatibleBitmap(mem_dc, w, h) };

//         //SelectObject(mem_dc, bmp); <== Problem is here

//         //DeleteObject(bmp); <== Same problem here
//         unsafe { DeleteDC(mem_dc) };
//         unsafe { ReleaseDC(hwnd, hwnd_dc) };
//     // }
//     // else {
//     //     println!("Window not found");
//     // }
// }

// references for winRT
// https://github.com/robmikh/wgc-rust-demo/blob/master/src/main.rs
