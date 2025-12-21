use koicore::command::{Command, CompositeValue, Parameter, Value};
use std::ffi::CStr;
use std::ffi::c_char;
use std::slice;

use super::command::KoiCommand;

/// Unified parameter type enumeration
///
/// This enumeration represents all possible parameter types in KoiLang commands.
/// It includes both basic types (int, float, string) and composite types (single, list, dict).
#[repr(C)]
pub enum KoiParamType {
    /// 64-bit signed integer value
    BasicInt = 0,
    /// 64-bit floating point value
    BasicFloat = 1,
    /// UTF-8 string value (merged Literal and String into single String type)
    BasicString = 2,
    /// Single composite value
    CompositeSingle = 3,
    /// List composite value
    CompositeList = 4,
    /// Dictionary composite value
    CompositeDict = 5,
    /// Invalid or unknown type
    Invalid = -1,
    /// Boolean value
    BasicBool = 6,
}

/// Get number of parameters in command
///
/// # Arguments
/// * `command` - Command object pointer
///
/// # Returns
/// Number of parameters, or 0 if command is null
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetParamCount(command: *mut KoiCommand) -> usize {
    if command.is_null() {
        return 0;
    }

    let command = unsafe { &*(command as *mut Command) };
    command.params().len()
}

/// Get parameter type (unified enum for both basic and composite types)
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
///
/// # Returns
/// Parameter type, or KoiParamType::Invalid on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetParamType(command: *mut KoiCommand, index: usize) -> i32 {
    if command.is_null() {
        return KoiParamType::Invalid as i32;
    }

    let command = unsafe { &*(command as *mut Command) };
    let params = command.params();

    if index >= params.len() {
        return KoiParamType::Invalid as i32;
    }

    match &params[index] {
        Parameter::Basic(value) => match value {
            Value::Int(_) => KoiParamType::BasicInt as i32,
            Value::Float(_) => KoiParamType::BasicFloat as i32,
            Value::String(_) => KoiParamType::BasicString as i32,
            Value::Bool(_) => KoiParamType::BasicBool as i32,
        },
        Parameter::Composite(_, composite) => match composite {
            CompositeValue::Single(_) => KoiParamType::CompositeSingle as i32,
            CompositeValue::List(_) => KoiParamType::CompositeList as i32,
            CompositeValue::Dict(_) => KoiParamType::CompositeDict as i32,
        },
    }
}

/// Get integer value from basic parameter
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
/// * `out_value` - Pointer to store integer value
///
/// # Returns
/// 0 on success, non-zero on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetIntParam(
    command: *mut KoiCommand,
    index: usize,
    out_value: *mut i64,
) -> i32 {
    if command.is_null() || out_value.is_null() {
        return -1;
    }

    let command = unsafe { &*(command as *mut Command) };
    let params = command.params();

    if index >= params.len() {
        return -2;
    }

    match &params[index] {
        Parameter::Basic(Value::Int(value)) => {
            unsafe {
                *out_value = *value;
            }
            0
        }
        _ => -3,
    }
}

/// Get float value from basic parameter
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
/// * `out_value` - Pointer to store float value
///
/// # Returns
/// 0 on success, non-zero on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetFloatParam(
    command: *mut KoiCommand,
    index: usize,
    out_value: *mut f64,
) -> i32 {
    if command.is_null() || out_value.is_null() {
        return -1;
    }

    let command = unsafe { &*(command as *mut Command) };
    let params = command.params();

    if index >= params.len() {
        return -2;
    }

    match &params[index] {
        Parameter::Basic(Value::Float(value)) => {
            unsafe {
                *out_value = *value;
            }
            0
        }
        _ => -3,
    }
}

/// Get string value from basic parameter into provided buffer
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
/// * `buffer` - Buffer for string output
/// * `buffer_size` - Buffer size
///
/// # Returns
/// Actual string length (excluding null terminator), or required buffer size if insufficient
/// Returns 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetStringParam(
    command: *mut KoiCommand,
    index: usize,
    buffer: *mut c_char,
    buffer_size: usize,
) -> usize {
    if command.is_null() {
        return 0;
    }

    let command = unsafe { &*(command as *mut Command) };
    let params = command.params();

    if index >= params.len() {
        return 0;
    }

    let value_str = match &params[index] {
        Parameter::Basic(Value::String(value)) => value,
        _ => return 0,
    };

    let value_bytes = value_str.as_bytes();
    let value_len = value_bytes.len();
    let required_size = value_len + 1;

    if buffer.is_null() || buffer_size < required_size {
        return required_size;
    }

    let buffer_slice = unsafe { slice::from_raw_parts_mut(buffer as *mut u8, buffer_size) };
    buffer_slice[..value_len].copy_from_slice(value_bytes);
    buffer_slice[value_len] = 0;

    required_size
}

/// Get string parameter length
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
///
/// # Returns
/// Required buffer size (including null terminator), or 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetStringParamLen(
    command: *mut KoiCommand,
    index: usize,
) -> usize {
    if command.is_null() {
        return 0;
    }

    let command = unsafe { &*(command as *mut Command) };
    let params = command.params();

    if index >= params.len() {
        return 0;
    }

    match &params[index] {
        Parameter::Basic(Value::String(value)) => value.len() + 1,
        _ => 0,
    }
}

/// Get composite parameter name into provided buffer
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
/// * `buffer` - Buffer for name output
/// * `buffer_size` - Buffer size
///
/// # Returns
/// Actual name length (excluding null terminator), or required buffer size if insufficient
/// Returns 0 on error or if parameter is not composite
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetCompositeParamName(
    command: *mut KoiCommand,
    index: usize,
    buffer: *mut c_char,
    buffer_size: usize,
) -> usize {
    if command.is_null() {
        return 0;
    }

    let command = unsafe { &*(command as *mut Command) };
    let params = command.params();

    if index >= params.len() {
        return 0;
    }

    let name = match &params[index] {
        Parameter::Composite(name, _) => name,
        _ => return 0,
    };

    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len();
    let required_size = name_len + 1;

    if buffer.is_null() || buffer_size < required_size {
        return required_size;
    }

    let buffer_slice = unsafe { slice::from_raw_parts_mut(buffer as *mut u8, buffer_size) };
    buffer_slice[..name_len].copy_from_slice(name_bytes);
    buffer_slice[name_len] = 0;

    required_size
}

/// Get composite parameter name length
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
///
/// # Returns
/// Required buffer size (including null terminator), or 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetCompositeParamNameLen(
    command: *mut KoiCommand,
    index: usize,
) -> usize {
    if command.is_null() {
        return 0;
    }

    let command = unsafe { &*(command as *mut Command) };
    let params = command.params();

    if index >= params.len() {
        return 0;
    }

    match &params[index] {
        Parameter::Composite(name, _) => name.len() + 1,
        _ => 0,
    }
}

/// Check if command is a text command (@text)
///
/// # Arguments
/// * `command` - Command object pointer
///
/// # Returns
/// 1 if text command, 0 otherwise or on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_IsTextCommand(command: *mut KoiCommand) -> i32 {
    if command.is_null() {
        return 0;
    }

    let command = unsafe { &*(command as *mut Command) };
    (command.name() == "@text") as i32
}

/// Check if command is an annotation command (@annotation)
///
/// # Arguments
/// * `command` - Command object pointer
///
/// # Returns
/// 1 if annotation command, 0 otherwise or on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_IsAnnotationCommand(command: *mut KoiCommand) -> i32 {
    if command.is_null() {
        return 0;
    }

    let command = unsafe { &*(command as *mut Command) };
    (command.name() == "@annotation") as i32
}

/// Check if command is a number command (@number)
///
/// # Arguments
/// * `command` - Command object pointer
///
/// # Returns
/// 1 if number command, 0 otherwise or on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_IsNumberCommand(command: *mut KoiCommand) -> i32 {
    if command.is_null() {
        return 0;
    }

    let command = unsafe { &*(command as *mut Command) };
    (command.name() == "@number") as i32
}

/// Add a new integer parameter to command
///
/// # Arguments
/// * `command` - Command object pointer
/// * `value` - Integer value
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_AddIntParameter(command: *mut KoiCommand, value: i64) -> i32 {
    if command.is_null() {
        return -1;
    }

    let command = unsafe { &mut *(command as *mut Command) };
    command.params.push(value.into());
    0
}

/// Add a new float parameter to command
///
/// # Arguments
/// * `command` - Command object pointer
/// * `value` - Float value
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_AddFloatParameter(command: *mut KoiCommand, value: f64) -> i32 {
    if command.is_null() {
        return -1;
    }

    let command = unsafe { &mut *(command as *mut Command) };
    command.params.push(value.into());
    0
}

/// Add a new string parameter to command
///
/// # Arguments
/// * `command` - Command object pointer
/// * `value` - String value (null-terminated C string)
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_AddStringParameter(
    command: *mut KoiCommand,
    value: *const c_char,
) -> i32 {
    if command.is_null() || value.is_null() {
        return -1;
    }

    let value_str = match unsafe { CStr::from_ptr(value) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -2,
    };

    let command = unsafe { &mut *(command as *mut Command) };
    command.params.push(Value::String(value_str).into());
    0
}

/// Remove parameter from command by index
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index to remove
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_RemoveParameter(command: *mut KoiCommand, index: usize) -> i32 {
    if command.is_null() {
        return -1;
    }

    let command = unsafe { &mut *(command as *mut Command) };
    command.params.remove(index);
    0
}

/// Clear all parameters from command
///
/// # Arguments
/// * `command` - Command object pointer
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_ClearParameters(command: *mut KoiCommand) -> i32 {
    if command.is_null() {
        return -1;
    }

    let command = unsafe { &mut *(command as *mut Command) };
    command.params.clear();
    0
}

/// Modify integer parameter value
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
/// * `value` - New integer value
///
/// # Returns
/// 0 on success, non-zero on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_SetIntParameter(
    command: *mut KoiCommand,
    index: usize,
    value: i64,
) -> i32 {
    if command.is_null() {
        return -1;
    }

    let command = unsafe { &mut *(command as *mut Command) };
    let params = &mut command.params;

    if index >= params.len() {
        return -2;
    }

    match &mut params[index] {
        Parameter::Basic(Value::Int(old_value)) => {
            *old_value = value;
            0
        }
        _ => -3,
    }
}

/// Modify float parameter value
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
/// * `value` - New float value
///
/// # Returns
/// 0 on success, non-zero on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_SetFloatParameter(
    command: *mut KoiCommand,
    index: usize,
    value: f64,
) -> i32 {
    if command.is_null() {
        return -1;
    }

    let command = unsafe { &mut *(command as *mut Command) };
    let params = &mut command.params;

    if index >= params.len() {
        return -2;
    }

    match &mut params[index] {
        Parameter::Basic(Value::Float(old_value)) => {
            *old_value = value;
            0
        }
        _ => -3,
    }
}

/// Modify string parameter value
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
/// * `value` - New string value (null-terminated C string)
///
/// # Returns
/// 0 on success, non-zero on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_SetStringParameter(
    command: *mut KoiCommand,
    index: usize,
    value: *const c_char,
) -> i32 {
    if command.is_null() || value.is_null() {
        return -1;
    }

    let value_str = match unsafe { CStr::from_ptr(value) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -2,
    };

    let command = unsafe { &mut *(command as *mut Command) };
    let params = &mut command.params;

    if index >= params.len() {
        return -3;
    }

    match &mut params[index] {
        Parameter::Basic(Value::String(old_value)) => {
            *old_value = value_str;
            0
        }
        _ => -4,
    }
}

/// Get boolean value from basic parameter
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
/// * `out_value` - Pointer to store boolean value (1 for true, 0 for false)
///
/// # Returns
/// 0 on success, non-zero on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetBoolParam(
    command: *mut KoiCommand,
    index: usize,
    out_value: *mut i32,
) -> i32 {
    if command.is_null() || out_value.is_null() {
        return -1;
    }

    let command = unsafe { &*(command as *mut Command) };
    let params = command.params();

    if index >= params.len() {
        return -2;
    }

    match &params[index] {
        Parameter::Basic(Value::Bool(value)) => {
            unsafe { *out_value = if *value { 1 } else { 0 } };
            0
        }
        _ => -3,
    }
}

/// Add a new boolean parameter to command
///
/// # Arguments
/// * `command` - Command object pointer
/// * `value` - Boolean value (non-zero for true, 0 for false)
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_AddBoolParameter(command: *mut KoiCommand, value: i32) -> i32 {
    if command.is_null() {
        return -1;
    }

    let command = unsafe { &mut *(command as *mut Command) };
    command.params.push(Value::Bool(value != 0).into());
    0
}

/// Modify boolean parameter value
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
/// * `value` - New boolean value (non-zero for true, 0 for false)
///
/// # Returns
/// 0 on success, non-zero on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_SetBoolParameter(
    command: *mut KoiCommand,
    index: usize,
    value: i32,
) -> i32 {
    if command.is_null() {
        return -1;
    }

    let command = unsafe { &mut *(command as *mut Command) };
    let params = &mut command.params;

    if index >= params.len() {
        return -2;
    }

    match &mut params[index] {
        Parameter::Basic(Value::Bool(old_value)) => {
            *old_value = value != 0;
            0
        }
        _ => -3,
    }
}
