use koicore::{command::{Command, CompositeValue, Parameter}, Value};
use std::{ffi::c_char, ptr, slice};

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
    
    let command = &*(command as *mut Command);
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
    
    let param = &*(list as *const Parameter);
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
    
    let param = &*(list as *const Parameter);
    match param {
        Parameter::Composite(_, CompositeValue::List(values)) => {
            if index >= values.len() {
                KoiParamType::Invalid as i32
            } else {
                match &values[index] {
                    Value::Int(_) => KoiParamType::BasicInt as i32,
                    Value::Float(_) => KoiParamType::BasicFloat as i32,
                    Value::Literal(_) => KoiParamType::BasicLiteral as i32,
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
/// 1 on success, 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_GetIntValue(
    list: *mut KoiCompositeList,
    index: usize,
    out_value: *mut i64,
) -> i32 {
    if list.is_null() || out_value.is_null() {
        return 0;
    }
    
    let param = &*(list as *const Parameter);
    match param {
        Parameter::Composite(_, CompositeValue::List(values)) => {
            if index >= values.len() {
                return 0;
            }
            
            match &values[index] {
                Value::Int(value) => {
                    *out_value = *value;
                    1
                }
                _ => 0,
            }
        }
        _ => 0,
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
/// 1 on success, 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_GetFloatValue(
    list: *mut KoiCompositeList,
    index: usize,
    out_value: *mut f64,
) -> i32 {
    if list.is_null() || out_value.is_null() {
        return 0;
    }
    
    let param = &*(list as *const Parameter);
    match param {
        Parameter::Composite(_, CompositeValue::List(values)) => {
            if index >= values.len() {
                return 0;
            }
            
            match &values[index] {
                Value::Float(value) => {
                    *out_value = *value;
                    1
                }
                _ => 0,
            }
        }
        _ => 0,
    }
}

/// Get string Value from composite list by index
///
/// # Arguments
/// * `list` - Composite list parameter pointer
/// * `index` - Value index
/// * `out_value` - Pointer to store string value
///
/// # Returns
/// Actual string length (excluding null terminator), or required buffer size if insufficient
/// Returns 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_GetStringValue(
    list: *mut KoiCompositeList,
    index: usize,
    out_value: *mut c_char,
    buffer_size: usize,
) -> usize {
    if list.is_null() || out_value.is_null() {
        return 0;
    }
    
    let param = &*(list as *const Parameter);
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
            
            let buffer_slice = slice::from_raw_parts_mut(out_value as *mut u8, buffer_size);
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
/// Required buffer size (including null terminator), or 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_GetStringValueLen(
    list: *mut KoiCompositeList,
    index: usize,
) -> usize {
    if list.is_null() {
        return 0;
    }
    
    let param = &*(list as *const Parameter);
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
