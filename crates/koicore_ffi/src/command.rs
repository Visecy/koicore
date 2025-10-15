use koicore::command::{Command, Parameter, Value, CompositeValue};
use std::ffi::{c_char, CStr};
use std::slice;
use std::ptr;

/// Opaque handle for KoiLang command
#[repr(C)]
pub struct KoiCommand {
    _data: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Opaque handle for composite list parameter
#[repr(C)]
pub struct KoiCompositeList {
    _data: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Opaque handle for composite dict parameter
#[repr(C)]
pub struct KoiCompositeDict {
    _data: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

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

/// Get command name, caller provides buffer
///
/// # Arguments
/// * `command` - Command object pointer
/// * `buffer` - Buffer pointer provided by C caller
/// * `buffer_size` - Buffer size
///
/// # Returns
/// Returns actual length of command name (excluding null terminator)
/// If buffer_size is insufficient, returns required buffer size (including null terminator)
/// Returns 0 if parameters are invalid
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetName(
    command: *mut KoiCommand,
    buffer: *mut c_char,
    buffer_size: usize,
) -> usize {
    if command.is_null() {
        return 0;
    }
    
    let command = &*(command as *mut Command);
    let name = command.name();
    let name_bytes = name.as_bytes();
    let name_len = name_bytes.len();
    
    // Calculate required buffer size (including null terminator)
    let required_size = name_len + 1;
    
    // If buffer is null or size is insufficient, return required size
    if buffer.is_null() || buffer_size < required_size {
        return required_size;
    }
    
    // Copy name to buffer
    let buffer_slice = slice::from_raw_parts_mut(buffer as *mut u8, buffer_size);
    buffer_slice[..name_len].copy_from_slice(name_bytes);
    buffer_slice[name_len] = 0; // null terminator
    
    required_size
}

/// Get command name length, always returns required size
/// Caller can call this first to get size, then allocate buffer and call full version
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetNameLen(command: *mut KoiCommand) -> usize {
    if command.is_null() {
        return 0;
    }
    
    let command = &*(command as *mut Command);
    command.name().len() + 1 // including null terminator
}

/// Create a new command with specified name and parameters
///
/// # Arguments
/// * `name` - Command name (null-terminated C string)
/// * `param_count` - Number of parameters
/// * `params` - Array of parameter pointers
///
/// # Returns
/// Pointer to new command object, or null pointer on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_Create(
    name: *const c_char,
    param_count: usize,
    params: *const *mut KoiCommand,
) -> *mut KoiCommand {
    if name.is_null() {
        return ptr::null_mut();
    }
    
    let name_str = match CStr::from_ptr(name).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    
    let mut rust_params = Vec::with_capacity(param_count);
    
    for i in 0..param_count {
        let param_ptr = *params.add(i);
        if param_ptr.is_null() {
            return ptr::null_mut();
        }
        let param = Box::from_raw(param_ptr as *mut Parameter);
        rust_params.push(*param);
    }
    
    let command = Command::new(name_str, rust_params);
    Box::into_raw(Box::new(command)) as *mut KoiCommand
}

/// Create a text command (@text)
///
/// # Arguments
/// * `content` - Text content (null-terminated C string)
///
/// # Returns
/// Pointer to new text command object
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_CreateText(content: *const c_char) -> *mut KoiCommand {
    if content.is_null() {
        return ptr::null_mut();
    }
    
    let content_str = match CStr::from_ptr(content).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    
    let command = Command::new_text(content_str);
    Box::into_raw(Box::new(command)) as *mut KoiCommand
}

/// Create an annotation command (@annotation)
///
/// # Arguments
/// * `content` - Annotation content (null-terminated C string)
///
/// # Returns
/// Pointer to new annotation command object
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_CreateAnnotation(content: *const c_char) -> *mut KoiCommand {
    if content.is_null() {
        return ptr::null_mut();
    }
    
    let content_str = match CStr::from_ptr(content).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    
    let command = Command::new_annotation(content_str);
    Box::into_raw(Box::new(command)) as *mut KoiCommand
}

/// Create a number command (@number)
///
/// # Arguments
/// * `value` - Numeric value
/// * `param_count` - Number of additional parameters
/// * `params` - Array of additional parameter pointers
///
/// # Returns
/// Pointer to new number command object, or null pointer on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_CreateNumber(
    value: i64,
    param_count: usize,
    params: *const *mut KoiCommand,
) -> *mut KoiCommand {
    let mut rust_params = Vec::with_capacity(param_count);
    
    for i in 0..param_count {
        let param_ptr = *params.add(i);
        if param_ptr.is_null() {
            return ptr::null_mut();
        }
        let param = Box::from_raw(param_ptr as *mut Parameter);
        rust_params.push(*param);
    }
    
    let command = Command::new_number(value, rust_params);
    Box::into_raw(Box::new(command)) as *mut KoiCommand
}

/// Free a command object
///
/// # Arguments
/// * `command` - Command object pointer to free
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_Free(command: *mut KoiCommand) {
    if !command.is_null() {
        drop(Box::from_raw(command as *mut Command));
    }
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
        Parameter::Composite(_, CompositeValue::List(_)) => {
            // Cast the parameter reference to the opaque list type
            &params[index] as *const Parameter as *mut KoiCompositeList
        }
        _ => ptr::null_mut()
    }
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
        Parameter::Composite(_, CompositeValue::Dict(_)) => {
            // Cast the parameter reference to the opaque dict type
            &params[index] as *const Parameter as *mut KoiCompositeDict
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
/// Number of elements in the list, or 0 on error
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
