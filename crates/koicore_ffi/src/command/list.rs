use koicore::{command::{Command, CompositeValue, Parameter}, Value};
use std::{ffi::{c_char, CStr}, ptr, slice};

use crate::command::param::KoiParamType;

use super::command::KoiCommand;

/// Opaque handle for composite list parameter
#[repr(C)]
pub struct KoiCompositeList {
    _data: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Get composite list parameter from command
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
///
/// # Returns
/// Pointer to composite list parameter, or null on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetCompositeList(
    command: *mut KoiCommand,
    index: usize,
) -> *mut KoiCompositeList {
    if command.is_null() {
        return ptr::null_mut();
    }
    
    let command = unsafe { &*(command as *mut Command) };
    let params = command.params();
    
    if index >= params.len() {
        return ptr::null_mut();
    }
    
    match &params[index] {
        p @ &Parameter::Composite(_, CompositeValue::List(_)) => {
            // Cast the parameter reference to the opaque list type
            p as *const Parameter as *mut KoiCompositeList
        }
        _ => ptr::null_mut()
    }
}

/// Get composite list parameter length
///
/// # Arguments
/// * `list` - Composite list parameter pointer
///
/// # Returns
/// Number of Values in the list, or 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_GetLength(list: *mut KoiCompositeList) -> usize {
    if list.is_null() {
        return 0;
    }
    
    let param = unsafe { &*(list as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::List(values)) => values.len(),
        _ => 0,
    }
}

/// Get Value type from composite list by index
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `index` - Value index
///
/// # Returns
/// Parameter type of the Value, or KoiParamType::Invalid on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_GetValueType(
    list: *mut KoiCompositeList,
    index: usize,
) -> i32 {
    if list.is_null() {
        return KoiParamType::Invalid as i32;
    }
    
    let param = unsafe { &*(list as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::List(values)) => {
            if index >= values.len() {
                KoiParamType::Invalid as i32
            } else {
                match &values[index] {
                    Value::Int(_) => KoiParamType::BasicInt as i32,
                    Value::Float(_) => KoiParamType::BasicFloat as i32,
                    Value::String(_) => KoiParamType::BasicString as i32,
                }
            }
        }
        _ => KoiParamType::Invalid as i32,
    }
}

/// Get integer Value from composite list by index
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `index` - Value index
/// * `out_value` - Pointer to store integer value
///
/// # Returns
/// 0 on success, non-zero on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_GetIntValue(
    list: *mut KoiCompositeList,
    index: usize,
    out_value: *mut i64,
) -> i32 {
    if list.is_null() || out_value.is_null() {
        return -1;
    }
    
    let param = unsafe { &*(list as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::List(values)) => {
            if index >= values.len() {
                return -2;
            }
            
            match &values[index] {
                Value::Int(value) => {
                    unsafe { *out_value = *value };
                    0
                }
                _ => -3,
            }
        }
        _ => -4,
    }
}

/// Get float Value from composite list by index
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `index` - Value index
/// * `out_value` - Pointer to store float value
///
/// # Returns
/// 0 on success, non-zero on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_GetFloatValue(
    list: *mut KoiCompositeList,
    index: usize,
    out_value: *mut f64,
) -> i32 {
    if list.is_null() || out_value.is_null() {
        return -1;
    }
    
    let param = unsafe { &*(list as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::List(values)) => {
            if index >= values.len() {
                return -2;
            }
            
            match &values[index] {
                Value::Float(value) => {
                    unsafe { *out_value = *value };
                    0
                }
                _ => -3,
            }
        }
        _ => -4,
    }
}

/// Get string Value from composite list by index
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `index` - Value index
/// * `buffer` - Buffer for string output
/// * `buffer_size` - Buffer size
///
/// # Returns
/// Actual string length (excluding null terminator), or required buffer size if insufficient
/// Returns 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_GetStringValue(
    list: *mut KoiCompositeList,
    index: usize,
    buffer: *mut c_char,
    buffer_size: usize,
) -> usize {
    if list.is_null() || buffer.is_null() {
        return 0;
    }
    
    let param = unsafe { &*(list as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::List(values)) => {
            if index >= values.len() {
                return 0;
            }
            
            let value_str = match &values[index] {
                Value::String(value) => value,
                _ => return 0,
            };
            
            let value_bytes = value_str.as_bytes();
            let value_len = value_bytes.len();
            let required_size = value_len + 1;
            
            if buffer_size < required_size {
                return required_size;
            }
            
            let buffer_slice = unsafe { slice::from_raw_parts_mut(buffer as *mut u8, buffer_size) };
            buffer_slice[..value_len].copy_from_slice(value_bytes);
            buffer_slice[value_len] = 0;
            
            required_size
        }
        _ => 0,
    }
}

/// Get string Value length from composite list by index
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `index` - Value index
///
/// # Returns
/// Required buffer size (including null terminator)
/// Returns 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_GetStringValueLen(
    list: *mut KoiCompositeList,
    index: usize,
) -> usize {
    if list.is_null() {
        return 0;
    }
    
    let param = unsafe { &*(list as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::List(values)) => {
            if index >= values.len() {
                return 0;
            }
            
            match &values[index] {
                Value::String(value) => value.len() + 1,
                _ => 0,
            }
        }
        _ => 0,
    }
}

/// Create a new empty composite list
///
/// # Returns
/// Pointer to new composite list parameter, or null on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_New() -> *mut KoiCompositeList {
    let param = Parameter::Composite(
        "list".to_string(),
        CompositeValue::List(Vec::new())
    );
    Box::into_raw(Box::new(param)) as *mut KoiCompositeList
}

/// Add integer value to composite list
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `value` - Integer value to add
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_AddIntValue(
    list: *mut KoiCompositeList,
    value: i64,
) -> i32 {
    if list.is_null() {
        return -1;
    }
    
    let param = unsafe { &mut *(list as *mut Parameter) };
    match param {
        Parameter::Composite(name, CompositeValue::List(values)) => {
            values.push(Value::Int(value));
            0
        }
        _ => -3,
    }
}

/// Add float value to composite list
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `value` - Float value to add
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_AddFloatValue(
    list: *mut KoiCompositeList,
    value: f64,
) -> i32 {
    if list.is_null() {
        return -1;
    }
    
    let param = unsafe { &mut *(list as *mut Parameter) };
    match param {
        Parameter::Composite(name, CompositeValue::List(values)) => {
            values.push(Value::Float(value));
            0
        }
        _ => -3,
    }
}

/// Add string value to composite list
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `value` - String value to add (null-terminated C string)
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_AddStringValue(
    list: *mut KoiCompositeList,
    value: *const c_char,
) -> i32 {
    if list.is_null() || value.is_null() {
        return -1;
    }
    
    let value_str = match unsafe { CStr::from_ptr(value) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -2,
    };
    
    let param = unsafe { &mut *(list as *mut Parameter) };
    match param {
        Parameter::Composite(name, CompositeValue::List(values)) => {
            values.push(Value::String(value_str));
            0
        }
        _ => -3,
    }
}

/// Set integer value in composite list by index
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `index` - Value index
/// * `value` - New integer value
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_SetIntValue(
    list: *mut KoiCompositeList,
    index: usize,
    value: i64,
) -> i32 {
    if list.is_null() {
        return -1;
    }
    
    let param = unsafe { &mut *(list as *mut Parameter) };
    match param {
        Parameter::Composite(name, CompositeValue::List(values)) => {
            if index >= values.len() {
                return -2;
            }
            
            values[index] = Value::Int(value);
            0
        }
        _ => -3,
    }
}

/// Set float value in composite list by index
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `index` - Value index
/// * `value` - New float value
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_SetFloatValue(
    list: *mut KoiCompositeList,
    index: usize,
    value: f64,
) -> i32 {
    if list.is_null() {
        return -1;
    }
    
    let param = unsafe { &mut *(list as *mut Parameter) };
    match param {
        Parameter::Composite(name, CompositeValue::List(values)) => {
            if index >= values.len() {
                return -2;
            }
            
            values[index] = Value::Float(value);
            0
        }
        _ => -3,
    }
}

/// Set string value in composite list by index
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `index` - Value index
/// * `value` - New string value (null-terminated C string)
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_SetStringValue(
    list: *mut KoiCompositeList,
    index: usize,
    value: *const c_char,
) -> i32 {
    if list.is_null() || value.is_null() {
        return -1;
    }
    
    let value_str = match unsafe { CStr::from_ptr(value) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -2,
    };
    
    let param = unsafe { &mut *(list as *mut Parameter) };
    match param {
        Parameter::Composite(name, CompositeValue::List(values)) => {
            if index >= values.len() {
                return -2;
            }
            
            values[index] = Value::String(value_str);
            0
        }
        _ => -3,
    }
}

/// Remove value from composite list by index
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `index` - Value index
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_RemoveValue(
    list: *mut KoiCompositeList,
    index: usize,
) -> i32 {
    if list.is_null() {
        return -1;
    }
    
    let param = unsafe { &mut *(list as *mut Parameter) };
    match param {
        Parameter::Composite(name, CompositeValue::List(values)) => {
            if index >= values.len() {
                return -2;
            }
            
            values.remove(index);
            0
        }
        _ => -3,
    }
}

/// Clear all values from composite list
///
/// # Arguments
/// * `list` - Composite list parameter pointer
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_Clear(
    list: *mut KoiCompositeList,
) -> i32 {
    if list.is_null() {
        return -1;
    }
    
    let param = unsafe { &mut *(list as *mut Parameter) };
    match param {
        Parameter::Composite(name, CompositeValue::List(values)) => {
            values.clear();
            0
        }
        _ => -3,
    }
}

/// Free composite list parameter
///
/// # Arguments
/// * `list` - Composite list parameter pointer
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_Del(list: *mut KoiCompositeList) {
    if list.is_null() {
        return;
    }
    
    unsafe { drop(Box::from_raw(list as *mut Parameter)) };
}
