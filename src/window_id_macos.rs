use core_foundation::base::{CFGetTypeID, CFTypeID, ToVoid};
use core_foundation::number::*;
use core_foundation::string::{kCFStringEncodingUTF8, CFString, CFStringGetCStringPtr};
use core_foundation_sys::string::CFStringGetTypeID;
use core_graphics::display::*;
use std::ffi::CStr;
use std::os::raw::c_void;

#[derive(Debug)]
enum DictEntryValue {
    _Number(i64),
    _Bool(bool),
    _String(String),
    _Unknown,
}

fn window_list() -> Vec<(DictEntryValue, DictEntryValue, DictEntryValue)> {
    let mut win_list = Vec::new();
    let window_list_info = unsafe {
        CGWindowListCopyWindowInfo(
            kCGWindowListOptionIncludingWindow
                | kCGWindowListOptionOnScreenOnly
                | kCGWindowListExcludeDesktopElements,
            kCGNullWindowID,
        )
    };
    let count = unsafe { CFArrayGetCount(window_list_info) };
    for i in 0..count {
        let dic_ref =
            unsafe { CFArrayGetValueAtIndex(window_list_info, i as isize) as CFDictionaryRef };
        let window_owner = get_from_dict(dic_ref, "kCGWindowOwnerName");
        let window_id = get_from_dict(dic_ref, "kCGWindowNumber");
        let is_onscreen = get_from_dict(dic_ref, "kCGWindowIsOnscreen");
        win_list.push((window_owner, window_id, is_onscreen));
    }

    unsafe {
        CFRelease(window_list_info as CFTypeRef);
    }

    return win_list;
}

fn get_from_dict(dict: CFDictionaryRef, key: &str) -> DictEntryValue {
    let key: CFString = key.into();
    let mut value: *const c_void = std::ptr::null();
    if unsafe { CFDictionaryGetValueIfPresent(dict, key.to_void(), &mut value) != 0 } {
        let type_id: CFTypeID = unsafe { CFGetTypeID(value) };
        if type_id == unsafe { CFNumberGetTypeID() } {
            let value = value as CFNumberRef;
            match unsafe { CFNumberGetType(value) } {
                kCFNumberSInt64Type => {
                    let mut value_i64 = 0_i64;
                    let out_value: *mut i64 = &mut value_i64;
                    let converted =
                        unsafe { CFNumberGetValue(value, kCFNumberSInt64Type, out_value.cast()) };
                    if converted {
                        return DictEntryValue::_Number(value_i64);
                    }
                }
                kCFNumberSInt32Type => {
                    let mut value_i32 = 0_i32;
                    let out_value: *mut i32 = &mut value_i32;
                    let converted =
                        unsafe { CFNumberGetValue(value, kCFNumberSInt32Type, out_value.cast()) };
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
                let nss: Id<NSString> = unsafe { Id::from_retained_ptr(value as *mut NSString) };

                DictEntryValue::_String(nss.as_str().to_string())
            };
        } else {
            eprintln!("Unexpected type: {}", type_id);
        }
    }

    DictEntryValue::_Unknown
}

pub fn ls_win() {
    println!("Window | Id");
    for (window_owner, window_id, is_onscreen) in window_list() {
        match (window_owner, window_id) {
            (DictEntryValue::_String(window_owner), DictEntryValue::_Number(window_id)) => {
                println!("{} | {}", window_owner, window_id)
            }
            (_, _) => {}
        }
    }
}

/// hard nut to crack, some starting point was:
/// https://stackoverflow.com/questions/60117318/getting-window-owner-names-via-cgwindowlistcopywindowinfo-in-rust
/// then some more PRs where needed:
/// https://github.com/servo/core-foundation-rs/pulls?q=is%3Apr+author%3Asassman+
///
pub fn get_window_id_for(terminal: String) -> Option<u32> {
    // if let Ok(pids) = proc_pid::listpids(proc_pid::ProcType::ProcTTYOnly) {
    //     println!("Found {} processes using listpids()", pids.len());
    //     pids.iter().for_each(|pid| println!("PID: {}", pid));
    // }

    for term in terminal.to_lowercase().split(".") {
        for (window_owner, window_id, is_onscreen) in window_list() {
            // println!(
            //     "window owner: {:?}, {:?}, {:?}",
            //     window_owner, window_id, is_onscreen
            // );
            if let DictEntryValue::_Number(window_id) = window_id {
                if let DictEntryValue::_String(window_owner) = window_owner {
                    let window = &window_owner.to_lowercase();
                    let terminal = &terminal.to_lowercase();
                    // println!("checking for: {:?}", term);
                    if window.contains(term) || terminal.contains(window) {
                        dbg!(window_owner);
                        return Some(window_id as u32);
                    }
                }
            }
        }
    }

    None
}
