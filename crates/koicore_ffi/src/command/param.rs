use koicore::command::{Command, Parameter, Value, CompositeValue};
use std::ffi::c_char;
use std::ffi::CStr;
use std::slice;
use std::ptr;

use super::command::KoiCommand;

/// Unified parameter type enumeration
#[repr(C)]
pub enum KoiParamType {
    BasicInt = 0,
    BasicFloat = 1,
    BasicLiteral = 2,
    BasicString = 3,
    CompositeSingle = 4,
    CompositeList = 5,
    CompositeDict = 6,
    Invalid = -1,
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
    
    let command = &*(command as *mut Command);
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
pub unsafe extern "C" fn KoiCommand_GetParamType(
    command: *mut KoiCommand,
    index: usize,
) -> i32 {
    if command.is_null() {
        return KoiParamType::Invalid as i32;
    }
    
    let command = &*(command as *mut Command);
    let params = command.params();
    
    if index >= params.len() {
        return KoiParamType::Invalid as i32;
    }
    
    match &params[index] {
        Parameter::Basic(value) => match value {
            Value::Int(_) => KoiParamType::BasicInt as i32,
            Value::Float(_) => KoiParamType::BasicFloat as i32,
            Value::Literal(_) => KoiParamType::BasicLiteral as i32,
            Value::String(_) => KoiParamType::BasicString as i32,
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
/// 1 on success, 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetIntParam(
    command: *mut KoiCommand,
    index: usize,
    out_value: *mut i64,
) -> i32 {
    if command.is_null() || out_value.is_null() {
        return 0;
    }
    
    let command = &*(command as *mut Command);
    let params = command.params();
    
    if index >= params.len() {
        return 0;
    }
    
    match &params[index] {
        Parameter::Basic(Value::Int(value)) => {
            *out_value = *value;
            1
        }
        _ => 0,
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
/// 1 on success, 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetFloatParam(
    command: *mut KoiCommand,
    index: usize,
    out_value: *mut f64,
) -> i32 {
    if command.is_null() || out_value.is_null() {
        return 0;
    }
    
    let command = &*(command as *mut Command);
    let params = command.params();
    
    if index >= params.len() {
        return 0;
    }
    
    match &params[index] {
        Parameter::Basic(Value::Float(value)) => {
            *out_value = *value;
            1
        }
        _ => 0,
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
    
    let command = &*(command as *mut Command);
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
    
    let buffer_slice = slice::from_raw_parts_mut(buffer as *mut u8, buffer_size);
    buffer_slice[..value_len].copy_from_slice(value_bytes);
    buffer_slice[value_len] = 0;
    
    required_size
}

/// Get literal value from basic parameter into provided buffer
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
/// * `buffer` - Buffer for literal output
/// * `buffer_size` - Buffer size
///
/// # Returns
/// Actual literal length (excluding null terminator), or required buffer size if insufficient
/// Returns 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetLiteralParam(
    command: *mut KoiCommand,
    index: usize,
    buffer: *mut c_char,
    buffer_size: usize,
) -> usize {
    if command.is_null() {
        return 0;
    }
    
    let command = &*(command as *mut Command);
    let params = command.params();
    
    if index >= params.len() {
        return 0;
    }
    
    let value_str = match &params[index] {
        Parameter::Basic(Value::Literal(value)) => value,
        _ => return 0,
    };
    
    let value_bytes = value_str.as_bytes();
    let value_len = value_bytes.len();
    let required_size = value_len + 1;
    
    if buffer.is_null() || buffer_size < required_size {
        return required_size;
    }
    
    let buffer_slice = slice::from_raw_parts_mut(buffer as *mut u8, buffer_size);
    buffer_slice[..value_len].copy_from_slice(value_bytes);
    buffer_slice[value_len] = 0;
    
    required_size
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
    
    let command = &*(command as *mut Command);
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
    
    let buffer_slice = slice::from_raw_parts_mut(buffer as *mut u8, buffer_size);
    buffer_slice[..name_len].copy_from_slice(name_bytes);
    buffer_slice[name_len] = 0;
    
    required_size
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
    
    let command = &*(command as *mut Command);
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
    
    let command = &*(command as *mut Command);
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
    
    let command = &*(command as *mut Command);
    (command.name() == "@number") as i32
}

/// Add a new integer parameter to command
///
/// # Arguments
/// * `command` - Command object pointer
/// * `value` - Integer value
///
/// # Returns
/// 1 on success, 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_AddIntParameter(
    command: *mut KoiCommand,
    value: i64,
) -> i32 {
    if command.is_null() {
        return 0;
    }
    
    let command = &mut *(command as *mut Command);
    command.params.push(value.into());
    1
}

/// Add a new float parameter to command
///
/// # Arguments
/// * `command` - Command object pointer
/// * `value` - Float value
///
/// # Returns
/// 1 on success, 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_AddFloatParameter(
    command: *mut KoiCommand,
    value: f64,
) -> i32 {
    if command.is_null() {
        return 0;
    }
    
    let command = &mut *(command as *mut Command);
    command.params.push(value.into());
    1
}

/// Add a new string parameter to command
///
/// # Arguments
/// * `command` - Command object pointer
/// * `value` - String value (null-terminated C string)
///
/// # Returns
/// 1 on success, 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_AddStringParameter(
    command: *mut KoiCommand,
    value: *const c_char,
) -> i32 {
    if command.is_null() || value.is_null() {
        return 0;
    }
    
    let value_str = match CStr::from_ptr(value).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return 0,
    };
    
    let command = &mut *(command as *mut Command);
    command.params.push(Value::String(value_str).into());
    1
}

/// Add a new literal parameter to command
///
/// # Arguments
/// * `command` - Command object pointer
/// * `value` - Literal value (null-terminated C string)
///
/// # Returns
/// 1 on success, 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_AddLiteralParameter(
    command: *mut KoiCommand,
    value: *const c_char,
) -> i32 {
    if command.is_null() || value.is_null() {
        return 0;
    }
    
    let value_str = match CStr::from_ptr(value).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return 0,
    };
    
    let command = &mut *(command as *mut Command);
    command.params.push(Value::Literal(value_str).into());
    1
}

/// Remove parameter from command by index
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index to remove
///
/// # Returns
/// 1 on success, 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_RemoveParameter(
    command: *mut KoiCommand,
    index: usize,
) -> i32 {
    if command.is_null() {
        return 0;
    }
    
    let command = &mut *(command as *mut Command);
    command.params.remove(index);
    1
}

/// Clear all parameters from command
///
/// # Arguments
/// * `command` - Command object pointer
///
/// # Returns
/// 1 on success, 0 on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_ClearParameters(command: *mut KoiCommand) -> i32 {
    if command.is_null() {
        return 0;
    }
    
    let command = &mut *(command as *mut Command);
    command.params.clear();
    1
}

/// Modify integer parameter value
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
/// * `value` - New integer value
///
/// # Returns
/// 1 on success, 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_ModifyIntParameter(
    command: *mut KoiCommand,
    index: usize,
    value: i64,
) -> i32 {
    if command.is_null() {
        return 0;
    }
    
    let command = &mut *(command as *mut Command);
    let params = &mut command.params;
    
    if index >= params.len() {
        return 0;
    }
    
    match &mut params[index] {
        Parameter::Basic(Value::Int(old_value)) => {
            *old_value = value;
            1
        }
        _ => 0,
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
/// 1 on success, 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_ModifyFloatParameter(
    command: *mut KoiCommand,
    index: usize,
    value: f64,
) -> i32 {
    if command.is_null() {
        return 0;
    }
    
    let command = &mut *(command as *mut Command);
    let params = &mut command.params;
    
    if index >= params.len() {
        return 0;
    }
    
    match &mut params[index] {
        Parameter::Basic(Value::Float(old_value)) => {
            *old_value = value;
            1
        }
        _ => 0,
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
/// 1 on success, 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_ModifyStringParameter(
    command: *mut KoiCommand,
    index: usize,
    value: *const c_char,
) -> i32 {
    if command.is_null() || value.is_null() {
        return 0;
    }
    
    let value_str = match CStr::from_ptr(value).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return 0,
    };
    
    let command = &mut *(command as *mut Command);
    let params = &mut command.params;
    
    if index >= params.len() {
        return 0;
    }
    
    match &mut params[index] {
        Parameter::Basic(Value::String(old_value)) => {
            *old_value = value_str;
            1
        }
        _ => 0,
    }
}

/// Modify literal parameter value
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index
/// * `value` - New literal value (null-terminated C string)
///
/// # Returns
/// 1 on success, 0 on error or type mismatch
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_ModifyLiteralParameter(
    command: *mut KoiCommand,
    index: usize,
    value: *const c_char,
) -> i32 {
    if command.is_null() || value.is_null() {
        return 0;
    }
    
    let value_str = match CStr::from_ptr(value).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return 0,
    };
    
    let command = &mut *(command as *mut Command);
    let params = &mut command.params;
    
    if index >= params.len() {
        return 0;
    }
    
    match &mut params[index] {
        Parameter::Basic(Value::Literal(old_value)) => {
            *old_value = value_str;
            1
        }
        _ => 0,
    }
}
