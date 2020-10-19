use crate::macos::core_foundation_sys_patches::{
    kCFNumberSInt32Type as I32, kCFNumberSInt64Type as I64, CFBooleanGetValue, CFNumberGetType,
};
use crate::WindowList;
use anyhow::{anyhow, Result};
use core_foundation::base::{CFGetTypeID, CFTypeID, ToVoid};
use core_foundation::string::{kCFStringEncodingUTF8, CFString, CFStringGetCStringPtr};
use core_foundation_sys::number::{
    CFBooleanGetTypeID, CFNumberGetTypeID, CFNumberGetValue, CFNumberRef,
};
use core_foundation_sys::string::CFStringGetTypeID;
use core_graphics::display::*;
use std::ffi::CStr;
use std::ops::Deref;
use std::os::raw::c_void;

#[derive(Debug)]
enum DictEntryValue {
    _Number(i64),
    _Bool(bool),
    _String(String),
    _Unknown,
}

///
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
        let dic_ref =
            unsafe { CFArrayGetValueAtIndex(window_list_info, i as isize) as CFDictionaryRef };
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
        // let is_onscreen = get_from_dict(dic_ref, "kCGWindowIsOnscreen");
        match (window_owner, window_id) {
            (DictEntryValue::_String(name), DictEntryValue::_Number(win_id)) => {
                win_list.push((Some(name), win_id as u64));
            }
            _ => {}
        }
    }

    unsafe {
        CFRelease(window_list_info.cast());
    }

    Ok(win_list)
}

fn get_from_dict(dict: CFDictionaryRef, key: &str) -> DictEntryValue {
    let key: CFString = key.into();
    let mut value: *const c_void = std::ptr::null();
    if unsafe { CFDictionaryGetValueIfPresent(dict, key.to_void(), &mut value) != 0 } {
        let type_id: CFTypeID = unsafe { CFGetTypeID(value) };
        if type_id == unsafe { CFNumberGetTypeID() } {
            let value = value as CFNumberRef;
            match unsafe { CFNumberGetType(value) } {
                I64 => {
                    let mut value_i64 = 0_i64;
                    let out_value: *mut i64 = &mut value_i64;
                    let converted = unsafe { CFNumberGetValue(value, I64, out_value.cast()) };
                    if converted {
                        return DictEntryValue::_Number(value_i64);
                    }
                }
                I32 => {
                    let mut value_i32 = 0_i32;
                    let out_value: *mut i32 = &mut value_i32;
                    let converted = unsafe { CFNumberGetValue(value, I32, out_value.cast()) };
                    if converted {
                        return DictEntryValue::_Number(value_i32 as i64);
                    }
                }
                n => {
                    eprintln!("Unsupported Number of typeId: {}", n);
                }
            }
        } else if type_id == unsafe { CFBooleanGetTypeID() } {
            return DictEntryValue::_Bool(unsafe { CFBooleanGetValue(value.cast()) });
        } else if type_id == unsafe { CFStringGetTypeID() } {
            let c_ptr = unsafe { CFStringGetCStringPtr(value.cast(), kCFStringEncodingUTF8) };
            return if !c_ptr.is_null() {
                let c_result = unsafe { CStr::from_ptr(c_ptr) };
                let result = String::from(c_result.to_str().unwrap());
                DictEntryValue::_String(result)
            } else {
                // in this case there is a high chance we got a `NSString` instead of `CFString`
                // we have to use the objc runtime to fetch it
                use objc_foundation::{INSString, NSString};
                use objc_id::Id;
                let nss: Id<NSString> = unsafe { Id::from_ptr(value as *mut NSString) };
                let str = std::str::from_utf8(nss.deref().as_str().as_bytes());

                match str {
                    Ok(s) => DictEntryValue::_String(s.to_owned()),
                    Err(_) => DictEntryValue::_Unknown,
                }
            };
        } else {
            eprintln!("Unexpected type: {}", type_id);
        }
    }

    DictEntryValue::_Unknown
}
