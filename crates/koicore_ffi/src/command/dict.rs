use koicore::{command::{Command, CompositeValue, Parameter}, Value};
use std::{ffi::{c_char, CStr}, ptr, slice};

use crate::command::param::KoiParamType;

use super::command::KoiCommand;

/// Opaque handle for composite dict parameter
/// 
/// This struct represents a dictionary-style composite parameter that stores key-value pairs.
/// The keys are strings, and the values can be integers, floats, or strings.
/// This is an opaque type intended for use through the C FFI API.
#[repr(C)]
pub struct KoiCompositeDict {
    _data: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Create a new composite dict parameter
///
/// Creates an empty dictionary with no key-value pairs. The returned pointer
/// must be freed using KoiCompositeDict_Del when no longer needed to avoid memory leaks.
///
/// # Returns
/// Pointer to the new composite dict parameter, or null on error
///
/// # Safety
/// The returned pointer must not be used after calling KoiCompositeDict_Del on it.
/// The caller is responsible for memory management of the returned object.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_New() -> *mut KoiCompositeDict {
    let param = Parameter::Composite(
        "dict".to_string(),
        CompositeValue::Dict(Vec::new())
    );
    Box::into_raw(Box::new(param)) as *mut KoiCompositeDict
}

/// Get composite dict parameter from command
///
/// Retrieves a dictionary parameter from a command at the specified index.
/// The parameter must be of dictionary type, otherwise null is returned.
///
/// # Ownership and Lifetime
///
/// The returned pointer is a borrowed reference to data owned by the command.
/// It must NOT be freed with KoiCompositeDict_Del. The pointer is only valid
/// as long as the command object exists and is not modified or destroyed.
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index (0-based)
///
/// # Returns
/// Pointer to composite dict parameter, or null on error:
/// - null if command is null
/// - null if index is out of bounds
/// - null if parameter at index is not a dictionary
///
/// # Safety
/// The command pointer must be either null or point to a valid KoiCommand object.
/// The returned pointer must NOT be freed with KoiCompositeDict_Del as it is owned by the command.
/// The returned pointer becomes invalid if the command is destroyed or modified.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetCompositeDict(
    command: *mut KoiCommand,
    index: usize,
) -> *mut KoiCompositeDict {
    if command.is_null() {
        return ptr::null_mut();
    }
    
    let command = unsafe { &*(command as *mut Command) };
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

/// Get number of entries in dict
///
/// Returns the count of key-value pairs currently stored in the dictionary.
/// This is useful for iteration or checking if the dictionary is empty.
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
///
/// # Returns
/// Number of entries in the dict, 0 on error:
/// - 0 if dict is null
/// - 0 if dict is not a valid dictionary
///
/// # Safety
/// The dict pointer must be a valid KoiCompositeDict pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetLength(dict: *mut KoiCompositeDict) -> usize {
    if dict.is_null() {
        return 0;
    }
    
    let param = unsafe { &*(dict as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => entries.len(),
        _ => 0,
    }
}

/// Remove entry from composite dict by key
///
/// Removes a key-value pair from the dictionary based on the provided key.
/// If the key does not exist in the dictionary, the function returns an error.
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key name (null-terminated UTF-8 string)
///
/// # Returns
/// 0 on success, non-zero on error:
/// - -1 if dict or key is null
/// - -2 if key contains invalid UTF-8
/// - -3 if key not found in dictionary
/// - -4 if dict is not a valid dictionary
///
/// # Safety
/// The dict pointer must be a valid KoiCompositeDict pointer.
/// The key pointer must be a valid null-terminated C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_Remove(
    dict: *mut KoiCompositeDict,
    key: *const c_char,
) -> i32 {
    if dict.is_null() || key.is_null() {
        return -1;
    }
    
    let key_str = unsafe { CStr::from_ptr(key) };
    let key_str = match key_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    let param = unsafe { &mut *(dict as *mut Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            let original_len = entries.len();
            entries.retain(|(k, _)| k != key_str);
            
            if entries.len() == original_len {
                // Key not found
                return -3;
            }
            0
        }
        _ => -4,
    }
}

/// Clear all entries from composite dict
///
/// Removes all key-value pairs from the dictionary, making it empty.
/// This operation is irreversible but does not deallocate the dictionary itself.
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
///
/// # Returns
/// 0 on success, non-zero on error:
/// - -1 if dict is null
/// - -3 if dict is not a valid dictionary
///
/// # Safety
/// The dict pointer must be a valid KoiCompositeDict pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_Clear(
    dict: *mut KoiCompositeDict,
) -> i32 {
    if dict.is_null() {
        return -1;
    }
    
    let param = unsafe { &mut *(dict as *mut Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            entries.clear();
            0
        }
        _ => -3,
    }
}

/// Free composite dict parameter
///
/// Deallocates the memory used by the dictionary and all its key-value pairs.
/// After calling this function, the pointer becomes invalid and must not be used.
/// This function should only be called on dictionaries created with KoiCompositeDict_New,
/// not on dictionaries obtained from KoiCommand_GetCompositeDict.
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
///
/// # Safety
/// The dict pointer must be a valid KoiCompositeDict pointer created with KoiCompositeDict_New.
/// Do not call this function on pointers obtained from KoiCommand_GetCompositeDict.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_Del(dict: *mut KoiCompositeDict) {
    if dict.is_null() {
        return;
    }
    
    unsafe { drop(Box::from_raw(dict as *mut Parameter)) };
}

/// Set integer value in composite dict by key
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key name
/// * `value` - Integer value to set
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_SetIntValue(
    dict: *mut KoiCompositeDict,
    key: *const c_char,
    value: i64,
) -> i32 {
    if dict.is_null() || key.is_null() {
        return -1;
    }
    
    let key_str = unsafe { CStr::from_ptr(key) };
    let key_str = match key_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    let param = unsafe { &mut *(dict as *mut Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            if let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == key_str) {
                *v = Value::Int(value);
                0
            } else {
                entries.push((key_str.to_string(), Value::Int(value)));
                0
            }
        }
        _ => -3,
    }
}

/// Set float value in composite dict by key
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key name
/// * `value` - Float value to set
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_SetFloatValue(
    dict: *mut KoiCompositeDict,
    key: *const c_char,
    value: f64,
) -> i32 {
    if dict.is_null() || key.is_null() {
        return -1;
    }
    
    let key_str = unsafe { CStr::from_ptr(key) };
    let key_str = match key_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    let param = unsafe { &mut *(dict as *mut Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            if let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == key_str) {
                *v = Value::Float(value);
                0
            } else {
                entries.push((key_str.to_string(), Value::Float(value)));
                0
            }
        }
        _ => -3,
    }
}

/// Set string value in composite dict by key
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key name
/// * `value` - String value to set
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_SetStringValue(
    dict: *mut KoiCompositeDict,
    key: *const c_char,
    value: *const c_char,
) -> i32 {
    if dict.is_null() || key.is_null() || value.is_null() {
        return -1;
    }
    
    let key_str = unsafe { CStr::from_ptr(key) };
    let key_str = match key_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    let value_str = unsafe { CStr::from_ptr(value) };
    let value_str = match value_str.to_str() {
        Ok(s) => s,
        Err(_) => return -3,
    };
    
    let param = unsafe { &mut *(dict as *mut Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            if let Some((_, v)) = entries.iter_mut().find(|(k, _)| k == key_str) {
                *v = Value::String(value_str.to_string());
                0
            } else {
                entries.push((key_str.to_string(), Value::String(value_str.to_string())));
                0
            }
        }
        _ => -4,
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
pub unsafe extern "C" fn KoiCompositeDict_GetKeybyIndex(
    dict: *mut KoiCompositeDict,
    index: usize,
    buffer: *mut c_char,
    buffer_size: usize,
) -> usize {
    if dict.is_null() {
        return 0;
    }
    
    let param = unsafe { &*(dict as *const Parameter) };
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
            
            let buffer_slice = unsafe { slice::from_raw_parts_mut(buffer as *mut u8, buffer_size) };
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
/// Required buffer size (including null terminator)
/// Returns 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetKeyLenByIndex(
    dict: *mut KoiCompositeDict,
    index: usize,
) -> usize {
    if dict.is_null() {
        return 0;
    }
    
    let param = unsafe { &*(dict as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            if index >= entries.len() {
                return 0;
            }
            
            let key = &entries[index].0;
            key.len() + 1
        }
        _ => 0,
    }
}

/// Get dict value type by index
/// 
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `index` - Entry index
///
/// # Returns
/// Value type as KoiParamType enum value
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetValueTypeByIndex(
    dict: *mut KoiCompositeDict,
    index: usize,
) -> i32 {
    if dict.is_null() {
        return KoiParamType::Invalid as i32;
    }
    
    let param = unsafe { &*(dict as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
           if index >= entries.len() {
                KoiParamType::Invalid as i32
            } else {
                match &entries[index].1 {
                    Value::Int(_) => KoiParamType::BasicInt as i32,
                    Value::Float(_) => KoiParamType::BasicFloat as i32,
                    Value::String(_) => KoiParamType::BasicString as i32,
                }
            }
        }
        _ => KoiParamType::Invalid as i32,
    }
}

/// Get value type from composite dict by key
/// 
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key name
///
/// # Returns
/// Value type as KoiParamType enum value
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetValueType(
    dict: *mut KoiCompositeDict,
    key: *const c_char,
) -> i32 {
    if dict.is_null() || key.is_null() {
        return KoiParamType::Invalid as i32;
    }
    
    let key_str = unsafe { CStr::from_ptr(key) };
    let key_str = match key_str.to_str() {
        Ok(s) => s,
        Err(_) => return KoiParamType::Invalid as i32,
    };
    
    let param = unsafe { &*(dict as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            if let Some((_, value)) = entries.iter().find(|(k, _)| k == key_str) {
                match value {
                    Value::Int(_) => KoiParamType::BasicInt as i32,
                    Value::Float(_) => KoiParamType::BasicFloat as i32,
                    Value::String(_) => KoiParamType::BasicString as i32,
                }
            } else {
                KoiParamType::Invalid as i32
            }
        }
        _ => KoiParamType::Invalid as i32,
    }
}

/// Get integer value from composite dict by key
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key name
/// * `out_value` - Pointer to store integer value
///
/// # Returns
/// 0 on success, non-zero on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetIntValue(
    dict: *mut KoiCompositeDict,
    key: *const c_char,
    out_value: *mut i64,
) -> i32 {
    if dict.is_null() || key.is_null() || out_value.is_null() {
        return -1;
    }
    
    let key_str = unsafe { CStr::from_ptr(key) };
    let key_str = match key_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    let param = unsafe { &*(dict as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            if let Some((_, value)) = entries.iter().find(|(k, _)| k == key_str) {
                match value {
                    Value::Int(v) => {
                        unsafe { *out_value = *v };
                        0
                    }
                    _ => -3,
                }
            } else {
                -2 // Key not found
            }
        }
        _ => -4,
    }
}

/// Get float value from composite dict by key
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key name
/// * `out_value` - Pointer to store float value
///
/// # Returns
/// 0 on success, non-zero on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeDict_GetFloatValue(
    dict: *mut KoiCompositeDict,
    key: *const c_char,
    out_value: *mut f64,
) -> i32 {
    if dict.is_null() || key.is_null() || out_value.is_null() {
        return -1;
    }
    
    let key_str = unsafe { CStr::from_ptr(key) };
    let key_str = match key_str.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };
    
    let param = unsafe { &*(dict as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            if let Some((_, value)) = entries.iter().find(|(k, _)| k == key_str) {
                match value {
                    Value::Float(v) => {
                        unsafe { *out_value = *v };
                        0
                    }
                    _ => -3,
                }
            } else {
                -2 // Key not found
            }
        }
        _ => -4,
    }
}

/// Get string value from composite dict by key into provided buffer
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key name
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
    
    let key_str = unsafe { CStr::from_ptr(key) };
    let key_str = match key_str.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    let param = unsafe { &*(dict as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            if let Some((_, Value::String(v))) = entries.iter().find(|(k, _)| k == key_str) {
                let v_bytes = v.as_bytes();
                let v_len = v_bytes.len();
                let required_size = v_len + 1;
                
                if buffer.is_null() || buffer_size < required_size {
                    return required_size;
                }
                
                let buffer_slice = unsafe { slice::from_raw_parts_mut(buffer as *mut u8, buffer_size) };
                buffer_slice[..v_len].copy_from_slice(v_bytes);
                buffer_slice[v_len] = 0;
                
                required_size
            } else {
                0 // Key not found, return 0 to indicate error
            }
        }
        _ => 0,
    }
}

/// Get string value length from composite dict by key
///
/// # Arguments
/// * `dict` - Composite dict parameter pointer
/// * `key` - Key name
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
    
    let key_str = unsafe { CStr::from_ptr(key) };
    let key_str = match key_str.to_str() {
        Ok(s) => s,
        Err(_) => return 0,
    };
    
    let param = unsafe { &*(dict as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Dict(entries)) => {
            if let Some((_, Value::String(v))) = entries.iter().find(|(k, _)| k == key_str) {
                v.len() + 1
            } else {
                0 // Key not found, return 0 to indicate error
            }
        }
        _ => 0,
    }
}
