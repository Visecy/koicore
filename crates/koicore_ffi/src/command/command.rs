use koicore::command::Command;
use std::ffi::{c_char, CStr};
use std::slice;
use std::ptr;

/// Opaque handle for KoiLang command
#[repr(C)]
pub struct KoiCommand {
    _data: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
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
    
    let command = unsafe { &*(command as *mut Command) };
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
    let buffer_slice = unsafe { slice::from_raw_parts_mut(buffer as *mut u8, buffer_size) };
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
    
    let command = unsafe { &*(command as *mut Command) };
    command.name().len() + 1 // including null terminator
}

/// Create a new command with specified name and parameters
///
/// # Arguments
/// * `name` - Command name (null-terminated C string)
///
/// # Returns
/// Pointer to new command object, or null pointer on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_New(
    name: *const c_char,
) -> *mut KoiCommand {
    if name.is_null() {
        return ptr::null_mut();
    }
    
    let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    
    let command = Command::new(name_str, Vec::new());
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
pub unsafe extern "C" fn KoiCommand_NewText(content: *const c_char) -> *mut KoiCommand {
    if content.is_null() {
        return ptr::null_mut();
    }
    
    let content_str = match unsafe { CStr::from_ptr(content) }.to_str() {
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
pub unsafe extern "C" fn KoiCommand_NewAnnotation(content: *const c_char) -> *mut KoiCommand {
    if content.is_null() {
        return ptr::null_mut();
    }
    
    let content_str = match unsafe { CStr::from_ptr(content) }.to_str() {
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
///
/// # Returns
/// Pointer to new number command object, or null pointer on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_NewNumber(
    value: i64,
) -> *mut KoiCommand {
    let command = Command::new_number(value, Vec::new());
    Box::into_raw(Box::new(command)) as *mut KoiCommand
}

/// Free a command object
///
/// # Arguments
/// * `command` - Command object pointer to free
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_Free(command: *mut KoiCommand) {
    if !command.is_null() {
        drop(unsafe { Box::from_raw(command as *mut Command) });
    }
}

/// Set command name
///
/// # Arguments
/// * `command` - Command object pointer
/// * `name` - New command name (null-terminated C string)
///
/// # Returns
/// 0 on success, non-zero on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_SetName(
    command: *mut KoiCommand,
    name: *const c_char,
) -> i32 {
    if command.is_null() || name.is_null() {
        return -1;
    }
    
    let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -1,
    };
    
    let command = unsafe { &mut *(command as *mut Command) };
    command.name = name_str;
    0
}

/// Clone a command object
///
/// # Arguments
/// * `command` - Command object pointer to clone
///
/// # Returns
/// Pointer to new command object, or null on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_Clone(command: *const KoiCommand) -> *mut KoiCommand {
    if command.is_null() {
        return ptr::null_mut();
    }
    
    let command = unsafe { &*(command as *const Command) };
    let cloned = command.clone();
    Box::into_raw(Box::new(cloned)) as *mut KoiCommand
}

/// Compare two command objects for equality
///
/// # Arguments
/// * `command1` - First command object pointer
/// * `command2` - Second command object pointer
///
/// # Returns
/// 1 if commands are equal, 0 if not equal or on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_Compare(
    command1: *const KoiCommand,
    command2: *const KoiCommand,
) -> i32 {
    if command1.is_null() || command2.is_null() {
        return 0;
    }
    
    let cmd1 = unsafe { &*(command1 as *const Command) };
    let cmd2 = unsafe { &*(command2 as *const Command) };
    (cmd1 == cmd2) as i32
}
