use crate::WindowList;

use anyhow::{anyhow, Result};
use core_foundation::base::{CFGetTypeID, CFTypeID, ToVoid};
use core_foundation::string::{kCFStringEncodingUTF8, CFString, CFStringGetCStringPtr};
use core_foundation_sys::number::*;
use core_foundation_sys::string::CFStringGetTypeID;
use core_graphics::display::*;
use log::error;
use std::any::TypeId;
use std::ffi::CStr;
use std::ops::Deref;
use std::os::raw::c_void;

#[derive(Debug)]
enum DictEntryValue {
    Number(i64),
    String(String),
    Unsupported,
}

/// hard nut to crack, some starting point was:
/// https://stackoverflow.com/questions/60117318/getting-window-owner-names-via-cgwindowlistcopywindowinfo-in-rust
/// then some more PRs where needed:
/// https://github.com/servo/core-foundation-rs/pulls?q=is%3Apr+author%3Asassman+
pub fn window_list() -> Result<WindowList> {
    let mut win_list = vec![];
    let window_list_info = unsafe {
        CGWindowListCopyWindowInfo(
            kCGWindowListOptionIncludingWindow
                | kCGWindowListOptionOnScreenOnly
                | kCGWindowListExcludeDesktopElements,
            kCGNullWindowID,
        )
    };
    if window_list_info.is_null() {
        return Err(anyhow!(
            "Cannot get window list results from low level C-API call `CGWindowListCopyWindowInfo` -> null"
        ));
    }
    let count = unsafe { CFArrayGetCount(window_list_info) };
    for i in 0..count {
        let dic_ref = unsafe { CFArrayGetValueAtIndex(window_list_info, i) as CFDictionaryRef };
        if dic_ref.is_null() {
            unsafe {
                CFRelease(window_list_info.cast());
            }
            return Err(anyhow!(
                "Cannot get a result from the window list from low level C-API call `CFArrayGetValueAtIndex` -> null"
            ));
        }
        let window_owner = get_from_dict(dic_ref, "kCGWindowOwnerName");
        let window_id = get_from_dict(dic_ref, "kCGWindowNumber");
        if let (DictEntryValue::String(name), DictEntryValue::Number(win_id)) =
            (window_owner, window_id)
        {
            win_list.push((Some(name), win_id as u64));
        }
    }

    unsafe {
        CFRelease(window_list_info.cast());
    }

    Ok(win_list)
}

fn convert_number<T: Default + 'static>(value: CFNumberRef) -> Option<T> {
    let v = unsafe { CFNumberGetType(value) };
    let mut value_i64 = T::default();
    let out_value: *mut T = &mut value_i64;
    let converted = unsafe { CFNumberGetValue(value, v, out_value.cast()) };
    if converted {
        Some(value_i64)
    } else {
        error!(
            "Error when converting a native number to type {:?} number {:?}",
            TypeId::of::<T>(),
            value
        );
        None
    }
}

fn get_from_dict(dict: CFDictionaryRef, key: &str) -> DictEntryValue {
    let key: CFString = key.into();
    let mut value: *const c_void = std::ptr::null();
    if unsafe { CFDictionaryGetValueIfPresent(dict, key.to_void(), &mut value) != 0 } {
        let type_id: CFTypeID = unsafe { CFGetTypeID(value) };
        if type_id == unsafe { CFNumberGetTypeID() } {
            let value = value as CFNumberRef;
            match unsafe { CFNumberGetType(value) } {
                v if v == kCFNumberSInt64Type => convert_number::<i64>(value)
                    .map(DictEntryValue::Number)
                    .unwrap_or(DictEntryValue::Unsupported),
                v if v == kCFNumberSInt32Type => convert_number::<i32>(value)
                    .map(|v| v as i64)
                    .map(DictEntryValue::Number)
                    .unwrap_or(DictEntryValue::Unsupported),
                n => {
                    error!("Unsupported Number of typeId: {}", n);
                    DictEntryValue::Unsupported
                }
            }
        } else if type_id == unsafe { CFBooleanGetTypeID() } {
            error!("Unexpected boolean, boolean should not come in our context");
            DictEntryValue::Unsupported // DictEntryValue::Bool(unsafe { CFBooleanGetValue(value.cast()) })
        } else if type_id == unsafe { CFStringGetTypeID() } {
            let c_ptr = unsafe { CFStringGetCStringPtr(value.cast(), kCFStringEncodingUTF8) };
            if !c_ptr.is_null() {
                let c_result = unsafe { CStr::from_ptr(c_ptr) };
                let result = String::from(c_result.to_str().unwrap());
                DictEntryValue::String(result)
            } else {
                // in this case there is a high chance we got a `NSString` instead of `CFString`
                // we have to use the objc runtime to fetch it
                use objc_foundation::{INSString, NSString};
                use objc_id::Id;
                let nss: Id<NSString> = unsafe { Id::from_ptr(value as *mut NSString) };
                let str = std::str::from_utf8(nss.deref().as_str().as_bytes());

                match str {
                    Ok(s) => DictEntryValue::String(s.to_owned()),
                    Err(_) => DictEntryValue::Unsupported,
                }
            }
        } else {
            error!("Unexpected type: {}", type_id);
            DictEntryValue::Unsupported
        }
    } else {
        error!("Unexpected type native type");
        DictEntryValue::Unsupported
    }
}
