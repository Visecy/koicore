use koicore::{command::{Command, CompositeValue, Parameter}, Value};
use std::{ffi::{c_char, CStr}, ptr, slice};

use super::command::KoiCommand;

/// Opaque handle for composite dict parameter
#[repr(C)]
pub struct KoiCompositeDict {
    _data: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Get composite dict parameter from command
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
///
/// # Returns
/// Pointer to composite dict parameter, or null on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetCompositeDict(
    command: *mut KoiCommand,
    index: usize,
) -> *mut KoiCompositeDict {
    if command.is_null() {
        return ptr::null_mut();
    }
    
    let command = &*(command as *mut Command);
    let params = command.params();
    
    if index >= params.len() {
        return ptr::null_mut();
    }
    
    match &params[index] {
        p @ &Parameter::Composite(_, CompositeValue::Dict(_)) => {
            // Cast the parameter reference to the opaque dict type
            p as *const Parameter as *mut KoiCompositeDict
        }
        _ => ptr::null_mut()
    }
}

/// Get composite dict parameter length
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
///
/// # Returns
/// Number of key-value pairs in the dict, or 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetLength(dict: *mut KoiCompositeDict) -> usize {
    if dict.is_null() {
        return 0;
    }
    
    let param = &*(dict as *const Parameter);
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => entries.len(),
        _ => 0,
    }
}

/// Get dict key by index into provided buffer
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `index` - Entry index
/// * `buffer` - Buffer for key output
/// * `buffer_size` - Buffer size
///
/// # Returns
/// Actual key length (excluding null terminator), or required buffer size if insufficient
/// Returns 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetKey(
    dict: *mut KoiCompositeDict,
    index: usize,
    buffer: *mut c_char,
    buffer_size: usize,
) -> usize {
    if dict.is_null() {
        return 0;
    }
    
    let param = &*(dict as *const Parameter);
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            if index >= entries.len() {
                return 0;
            }
            
            let key = &entries[index].0;
            let key_bytes = key.as_bytes();
    let key_len = key_bytes.len();
    let required_size = key_len + 1;
    
    if buffer.is_null() || buffer_size < required_size {
                return required_size;
            }
            
            let buffer_slice = slice::from_raw_parts_mut(buffer as *mut u8, buffer_size);
    buffer_slice[..key_len].copy_from_slice(key_bytes);
    buffer_slice[key_len] = 0;
    
    required_size
        }
        _ => 0,
    }
}

/// Get dict key length by index
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `index` - Entry index
///
/// # Returns
/// Required buffer size (including null terminator), or 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetKeyLen(
    dict: *mut KoiCompositeDict,
    index: usize,
) -> usize {
    if dict.is_null() {
        return 0;
    }
    
    let param = &*(dict as *const Parameter);
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            if index >= entries.len() {
                return 0;
            }
            
            entries[index].0.len() + 1
        }
        _ => 0,
    }
}

/// Get integer value from dict by key
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key to search for (null-terminated C string)
///
/// # Returns
/// 1 on success, 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetIntValue(
    dict: *mut KoiCompositeDict,
    key: *const c_char,
    out_value: *mut i64,
) -> i32 {
    if dict.is_null() || key.is_null() || out_value.is_null() {
        return 0;
    }
    
    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return 0,
    };
    
    let param = &*(dict as *const Parameter);
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            for (k, v) in entries {
                if *k == key_str {
                    match v {
                        Value::Int(value) => {
                            *out_value = *value;
                            return 1;
                        }
                        _ => return 0,
                    }
                }
            }
            0
        }
        _ => 0,
    }
}

/// Get float value from dict by key
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key to search for (null-terminated C string)
///
/// # Returns
/// 1 on success, 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetFloatValue(
    dict: *mut KoiCompositeDict,
    key: *const c_char,
    out_value: *mut f64,
) -> i32 {
    if dict.is_null() || key.is_null() || out_value.is_null() {
        return 0;
    }
    
    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return 0,
    };
    
    let param = &*(dict as *const Parameter);
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            for (k, v) in entries {
                if *k == key_str {
                    match v {
                        Value::Float(value) => {
                            *out_value = *value;
                            return 1;
                        }
                        _ => return 0,
                    }
                }
            }
            0
        }
        _ => 0,
    }
}

/// Get string value from dict by key into provided buffer
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key to search for (null-terminated C string)
/// * `buffer` - Buffer for string output
/// * `buffer_size` - Buffer size
///
/// # Returns
/// Actual string length (excluding null terminator), or required buffer size if insufficient
/// Returns 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetStringValue(
    dict: *mut KoiCompositeDict,
    key: *const c_char,
    buffer: *mut c_char,
    buffer_size: usize,
) -> usize {
    if dict.is_null() || key.is_null() {
        return 0;
    }
    
    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return 0,
    };
    
    let param = &*(dict as *const Parameter);
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            for (k, v) in entries {
                if *k == key_str {
                    match v {
                        Value::String(value) => {
                            let value_bytes = value.as_bytes();
                            let value_len = value_bytes.len();
                            let required_size = value_len + 1;
                            
                            if buffer.is_null() || buffer_size < required_size {
                                return required_size;
                            }
                            
                            let buffer_slice = slice::from_raw_parts_mut(buffer as *mut u8, buffer_size);
                            buffer_slice[..value_len].copy_from_slice(value_bytes);
                            buffer_slice[value_len] = 0;
                            
                            return required_size;
                        }
                        _ => return 0,
                    }
                }
            }
            0
        }
        _ => 0,
    }
}

/// Get string value length from dict by key
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key to search for (null-terminated C string)
///
/// # Returns
/// Required buffer size (including null terminator), or 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetStringValueLen(
    dict: *mut KoiCompositeDict,
    key: *const c_char,
) -> usize {
    if dict.is_null() || key.is_null() {
        return 0;
    }
    
    let key_str = match CStr::from_ptr(key).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return 0,
    };
    
    let param = &*(dict as *const Parameter);
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            for (k, v) in entries {
                if *k == key_str {
                    match v {
                        Value::String(value) => return value.len() + 1,
                        _ => return 0,
                    }
                }
            }
            0
        }
        _ => 0,
    }
}
