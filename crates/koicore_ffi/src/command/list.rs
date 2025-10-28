use koicore::{command::{Command, CompositeValue, Parameter}, Value};
use std::{ffi::{c_char, CStr}, ptr, slice};

use crate::command::param::KoiParamType;

use super::command::KoiCommand;

/// Opaque handle for composite list parameter
///
/// This structure represents a list parameter in a KoiLang command.
/// Lists can contain values of different types (integers, floats, strings).
#[repr(C)]
pub struct KoiCompositeList {
    _data: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Get composite list parameter from command
///
/// This function retrieves a list parameter from a command at the specified index.
/// The parameter must be of list type, otherwise NULL is returned.
///
/// # Ownership and Lifetime
///
/// The returned pointer is a borrowed reference to data owned by the command.
/// It must NOT be freed with KoiCompositeList_Del. The pointer is only valid
/// as long as the command object exists and is not modified or destroyed.
///
/// # Arguments
///
/// * `command` - Pointer to the command object
/// * `index` - Zero-based index of the parameter to retrieve
///
/// # Returns
///
/// Pointer to the composite list parameter, or NULL if:
/// - command is NULL
/// - index is out of bounds
/// - the parameter at the specified index is not a list
///
/// # Safety
///
/// The `command` pointer must be either NULL or point to a valid KoiCommand object.
/// The returned pointer must NOT be freed with KoiCompositeList_Del as it is owned by the command.
/// The returned pointer becomes invalid if the command is destroyed or modified.
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
/// This function returns the number of elements in the list parameter.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
///
/// # Returns
///
/// Number of elements in the list, or 0 if the list pointer is NULL or invalid.
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object
/// obtained from KoiCommand_GetCompositeList or KoiCompositeList_New.
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

/// Get value type from composite list by index
///
/// This function determines the type of a value at the specified index in the list.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
/// * `index` - Zero-based index of the value to query
///
/// # Returns
///
/// The type of the value as a KoiParamType enum value, or KoiParamType::Invalid
/// if the list pointer is NULL, invalid, or the index is out of bounds.
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
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

/// Get integer value from composite list by index
///
/// This function retrieves an integer value from the list at the specified index.
/// The value at the specified index must be of integer type.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
/// * `index` - Zero-based index of the value to retrieve
/// * `out_value` - Pointer to store the retrieved integer value
///
/// # Returns
///
/// 0 on success, or a non-zero error code on failure:
/// - -1: list pointer is NULL or out_value is NULL
/// - -2: index is out of bounds
/// - -3: value at the specified index is not an integer
/// - -4: list pointer is invalid
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
/// The `out_value` pointer must point to a valid i64 variable.
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

/// Get float value from composite list by index
///
/// This function retrieves a float value from the list at the specified index.
/// The value at the specified index must be of float type.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
/// * `index` - Zero-based index of the value to retrieve
/// * `out_value` - Pointer to store the retrieved float value
///
/// # Returns
///
/// 0 on success, or a non-zero error code on failure:
/// - -1: list pointer is NULL or out_value is NULL
/// - -2: index is out of bounds
/// - -3: value at the specified index is not a float
/// - -4: list pointer is invalid
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
/// The `out_value` pointer must point to a valid f64 variable.
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

/// Get string value from composite list by index
///
/// This function retrieves a string value from the list at the specified index.
/// The value at the specified index must be of string type.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
/// * `index` - Zero-based index of the value to retrieve
/// * `buffer` - Buffer to store the retrieved string value
/// * `buffer_size` - Size of the buffer in bytes
///
/// # Returns
///
/// The number of bytes required for the string including the null terminator.
/// If the buffer is NULL or too small, no data is written and the required size is returned.
/// Returns 0 on error or if the value at the specified index is not a string.
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
/// If `buffer` is not NULL, it must point to a valid memory region of at least `buffer_size` bytes.
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

/// Get string value length from composite list by index
///
/// This function returns the length of a string value at the specified index in the list.
/// The value at the specified index must be of string type.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
/// * `index` - Zero-based index of the value to query
///
/// # Returns
///
/// Required buffer size (including null terminator), or 0 on error or type mismatch.
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
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
/// This function creates a new empty composite list parameter that can be
/// added to a command or used independently.
///
/// # Returns
///
/// Pointer to the newly created composite list parameter, or NULL on failure.
/// The caller is responsible for freeing the list using KoiCompositeList_Del.
///
/// # Safety
///
/// The returned pointer must be freed using KoiCompositeList_Del when no longer needed.
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
/// This function appends an integer value to the end of the list.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
/// * `value` - Integer value to add
///
/// # Returns
///
/// 0 on success, or a non-zero error code on failure:
/// - -1: list pointer is NULL
/// - -3: list pointer is invalid
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object
/// obtained from KoiCommand_GetCompositeList or KoiCompositeList_New.
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
        Parameter::Composite(_, CompositeValue::List(values)) => {
            values.push(Value::Int(value));
            0
        }
        _ => -3,
    }
}

/// Add float value to composite list
///
/// This function appends a float value to the end of the list.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
/// * `value` - Float value to add
///
/// # Returns
///
/// 0 on success, or a non-zero error code on failure:
/// - -1: list pointer is NULL
/// - -3: list pointer is invalid
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object
/// obtained from KoiCommand_GetCompositeList or KoiCompositeList_New.
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
        Parameter::Composite(_, CompositeValue::List(values)) => {
            values.push(Value::Float(value));
            0
        }
        _ => -3,
    }
}

/// Add string value to composite list
///
/// This function appends a string value to the end of the list.
/// The string is copied and managed by the list.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
/// * `value` - String value to add (null-terminated C string)
///
/// # Returns
///
/// 0 on success, or a non-zero error code on failure:
/// - -1: list pointer is NULL or value is NULL
/// - -2: value contains invalid UTF-8
/// - -3: list pointer is invalid
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
/// The `value` pointer must be either NULL or point to a valid null-terminated string.
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
        Parameter::Composite(_, CompositeValue::List(values)) => {
            values.push(Value::String(value_str));
            0
        }
        _ => -3,
    }
}

/// Set integer value in composite list by index
///
/// This function replaces the value at the specified index with an integer value.
/// The index must be within the bounds of the list.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
/// * `index` - Zero-based index of the value to replace
/// * `value` - New integer value
///
/// # Returns
///
/// 0 on success, or a non-zero error code on failure:
/// - -1: list pointer is NULL
/// - -2: index is out of bounds
/// - -3: list pointer is invalid
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
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
        Parameter::Composite(_, CompositeValue::List(values)) => {
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
/// This function replaces the value at the specified index with a float value.
/// The index must be within the bounds of the list.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
/// * `index` - Zero-based index of the value to replace
/// * `value` - New float value
///
/// # Returns
///
/// 0 on success, or a non-zero error code on failure:
/// - -1: list pointer is NULL
/// - -2: index is out of bounds
/// - -3: list pointer is invalid
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
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
        Parameter::Composite(_, CompositeValue::List(values)) => {
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
/// This function replaces the value at the specified index with a string value.
/// The index must be within the bounds of the list.
/// The string is copied and managed by the list.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
/// * `index` - Zero-based index of the value to replace
/// * `value` - New string value (null-terminated C string)
///
/// # Returns
///
/// 0 on success, or a non-zero error code on failure:
/// - -1: list pointer is NULL or value is NULL
/// - -2: value contains invalid UTF-8
/// - -3: list pointer is invalid
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
/// The `value` pointer must be either NULL or point to a valid null-terminated string.
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
        Parameter::Composite(_, CompositeValue::List(values)) => {
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
/// This function removes the value at the specified index from the list.
/// The index must be within the bounds of the list.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
/// * `index` - Zero-based index of the value to remove
///
/// # Returns
///
/// 0 on success, or a non-zero error code on failure:
/// - -1: list pointer is NULL
/// - -2: index is out of bounds
/// - -3: list pointer is invalid
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
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
        Parameter::Composite(_, CompositeValue::List(values)) => {
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
/// This function removes all values from the list, making it empty.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter
///
/// # Returns
///
/// 0 on success, or a non-zero error code on failure:
/// - -1: list pointer is NULL
/// - -3: list pointer is invalid
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_Clear(
    list: *mut KoiCompositeList,
) -> i32 {
    if list.is_null() {
        return -1;
    }
    
    let param = unsafe { &mut *(list as *mut Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::List(values)) => {
            values.clear();
            0
        }
        _ => -3,
    }
}

/// Free composite list parameter
///
/// This function frees the memory used by a composite list parameter.
/// After calling this function, the list pointer becomes invalid and must not be used.
///
/// # Arguments
///
/// * `list` - Pointer to the composite list parameter to delete
///
/// # Safety
///
/// The `list` pointer must be either NULL or point to a valid KoiCompositeList object
/// obtained from KoiCommand_GetCompositeList or KoiCompositeList_New.
/// After this function returns, the pointer is invalid and must not be used.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeList_Del(list: *mut KoiCompositeList) {
    if list.is_null() {
        return;
    }
    
    unsafe { drop(Box::from_raw(list as *mut Parameter)) };
}
