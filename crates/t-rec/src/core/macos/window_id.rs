use super::cg_window_constants::{
    K_CG_NULL_WINDOW_ID, K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS,
    K_CG_WINDOW_LIST_OPTION_INCLUDING_WINDOW, K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY,
};
use crate::WindowList;
use anyhow::{anyhow, Result};
use objc2_core_foundation::{
    CFArray, CFBoolean, CFDictionary, CFGetTypeID, CFNumber, CFRetained, CFString, CFType,
    ConcreteType,
};
use objc2_core_graphics::CGWindowListCopyWindowInfo;
use std::ffi::c_void;

#[derive(Debug)]
enum DictEntryValue {
    Number(i64),
    /// Boolean value - actual value not used, just presence matters.
    Bool(()),
    String(String),
    Unknown,
}

pub fn window_list() -> Result<WindowList> {
    let mut win_list = vec![];

    #[allow(deprecated)] // CGWindowListCopyWindowInfo is deprecated but we still need it
    let window_list_info: Option<CFRetained<CFArray>> = CGWindowListCopyWindowInfo(
        K_CG_WINDOW_LIST_OPTION_INCLUDING_WINDOW
            | K_CG_WINDOW_LIST_OPTION_ON_SCREEN_ONLY
            | K_CG_WINDOW_LIST_EXCLUDE_DESKTOP_ELEMENTS,
        K_CG_NULL_WINDOW_ID,
    );

    let window_list_info = window_list_info.ok_or_else(|| {
        anyhow!(
            "Cannot get window list results from low level C-API call `CGWindowListCopyWindowInfo` -> null"
        )
    })?;

    let count = window_list_info.count();
    for i in 0..count {
        // SAFETY: index is within bounds, get raw pointer to dictionary
        let dict_ptr = unsafe { window_list_info.value_at_index(i) };

        // Cast to CFDictionary - the array contains dictionaries
        let dict: &CFDictionary = unsafe { &*(dict_ptr as *const CFDictionary) };

        let window_owner = get_from_dict(dict, "kCGWindowOwnerName");
        let window_id = get_from_dict(dict, "kCGWindowNumber");

        if let (DictEntryValue::String(name), DictEntryValue::Number(win_id)) =
            (window_owner, window_id)
        {
            win_list.push((Some(name), win_id as u64));
        }
    }

    Ok(win_list)
}

fn get_from_dict(dict: &CFDictionary, key: &str) -> DictEntryValue {
    let key_cfstring = CFString::from_str(key);

    // Try to get value from dictionary
    let value: *const c_void = unsafe {
        let key_ptr: *const c_void = (&*key_cfstring as *const CFString).cast();
        dict.value(key_ptr)
    };

    if value.is_null() {
        return DictEntryValue::Unknown;
    }

    // Get type IDs for comparison
    let cf_number_type_id = CFNumber::type_id();
    let cf_boolean_type_id = CFBoolean::type_id();
    let cf_string_type_id = CFString::type_id();

    let value_type_id = {
        let cf_type: &CFType = unsafe { &*(value as *const CFType) };
        CFGetTypeID(Some(cf_type))
    };

    if value_type_id == cf_number_type_id {
        let number: &CFNumber = unsafe { &*(value as *const CFNumber) };
        if let Some(n) = number.as_i64() {
            return DictEntryValue::Number(n);
        }
        if let Some(n) = number.as_i32() {
            return DictEntryValue::Number(n as i64);
        }
    } else if value_type_id == cf_boolean_type_id {
        let _boolean: &CFBoolean = unsafe { &*(value as *const CFBoolean) };
        return DictEntryValue::Bool(());
    } else if value_type_id == cf_string_type_id {
        let cf_str: &CFString = unsafe { &*(value as *const CFString) };
        return DictEntryValue::String(cf_str.to_string());
    }

    DictEntryValue::Unknown
}
