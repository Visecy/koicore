use koicore::{
    Value,
    command::{Command, CompositeValue, Parameter},
};
use std::{
    ffi::{CStr, c_char},
    ptr,
};

use super::command::KoiCommand;
use crate::command::param::KoiParamType;

/// Opaque handle for composite single parameter
///
/// Represents a named parameter with a single value, e.g., name(value).
#[repr(C)]
pub struct KoiCompositeSingle {
    _data: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Create a new composite single parameter
///
/// # Arguments
/// * `name` - The name of the composite parameter
/// * `value` - Initial integer value (dummy, will be overwritten by Set functions)
///
/// # Returns
/// Pointer to the new composite single parameter, or NULL on error
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeSingle_New(name: *const c_char) -> *mut KoiCompositeSingle {
    if name.is_null() {
        return ptr::null_mut();
    }

    let name_str = match unsafe { CStr::from_ptr(name) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let param = Parameter::Composite(name_str, CompositeValue::Single(Value::Int(0))); 
    Box::into_raw(Box::new(param)) as *mut KoiCompositeSingle
}

/// Borrow composite single parameter from command
///
/// Retrieves a borrowed reference to a single-value composite parameter from a command.
/// The parameter must be of single type, otherwise NULL is returned.
///
/// # Ownership and Lifetime
///
/// The returned pointer is a borrowed reference to data owned by the command.
/// It must NOT be freed with KoiCompositeSingle_Del. The pointer is only valid
/// as long as the command object exists and is not modified or destroyed.
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index (0-based)
///
/// # Returns
/// Pointer to composite single parameter, or NULL on error:
/// - NULL if command is NULL
/// - NULL if index is out of bounds
/// - NULL if parameter at index is not a single-value composite
///
/// # Safety
/// The command pointer must be either NULL or point to a valid KoiCommand object.
/// The returned pointer must NOT be freed with KoiCompositeSingle_Del as it is owned by the command.
/// The returned pointer becomes invalid if the command is destroyed or modified.
///
/// # Deprecated
///
/// This function is deprecated in favor of `KoiCommand_BorrowCompositeSingle` which has
/// a clearer name indicating the borrowing semantics. Use `KoiCommand_BorrowCompositeSingle`
/// for new code.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetCompositeSingle(
    command: *mut KoiCommand,
    index: usize,
) -> *mut KoiCompositeSingle {
    unsafe { KoiCommand_BorrowCompositeSingle(command, index) }
}

/// Borrow composite single parameter from command
///
/// Retrieves a borrowed reference to a single-value composite parameter from a command.
/// The parameter must be of single type, otherwise NULL is returned.
///
/// # Ownership and Lifetime
///
/// The returned pointer is a borrowed reference to data owned by the command.
/// It must NOT be freed with KoiCompositeSingle_Del. The pointer is only valid
/// as long as the command object exists and is not modified or destroyed.
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index (0-based)
///
/// # Returns
/// Pointer to composite single parameter, or NULL on error:
/// - NULL if command is NULL
/// - NULL if index is out of bounds
/// - NULL if parameter at index is not a single-value composite
///
/// # Safety
/// The command pointer must be either NULL or point to a valid KoiCommand object.
/// The returned pointer must NOT be freed with KoiCompositeSingle_Del as it is owned by the command.
/// The returned pointer becomes invalid if the command is destroyed or modified.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_BorrowCompositeSingle(
    command: *mut KoiCommand,
    index: usize,
) -> *mut KoiCompositeSingle {
    if command.is_null() {
        return ptr::null_mut();
    }

    let command = unsafe { &*(command as *mut Command) };
    let params = command.params();

    if index >= params.len() {
        return ptr::null_mut();
    }

    match &params[index] {
        p @ &Parameter::Composite(_, CompositeValue::Single(_)) => {
            p as *const Parameter as *mut KoiCompositeSingle
        }
        _ => ptr::null_mut(),
    }
}

/// Clone composite single parameter from command
///
/// Creates a new copy of a single-value composite parameter from a command.
/// The returned pointer is owned by the caller and must be freed with
/// `KoiCompositeSingle_Del` when no longer needed.
///
/// # Arguments
/// * `command` - Command object pointer
/// * `index` - Parameter index (0-based)
///
/// # Returns
/// Pointer to a new composite single parameter, or NULL on error:
/// - NULL if command is NULL
/// - NULL if index is out of bounds
/// - NULL if parameter at index is not a single-value composite
///
/// # Safety
/// The command pointer must be either NULL or point to a valid KoiCommand object.
/// The returned pointer must be freed with `KoiCompositeSingle_Del` when no longer needed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_CloneCompositeSingle(
    command: *mut KoiCommand,
    index: usize,
) -> *mut KoiCompositeSingle {
    if command.is_null() {
        return ptr::null_mut();
    }

    let command = unsafe { &*(command as *mut Command) };
    let params = command.params();

    if index >= params.len() {
        return ptr::null_mut();
    }

    match &params[index] {
        Parameter::Composite(name, CompositeValue::Single(value)) => {
            let cloned = Parameter::Composite(
                name.clone(),
                CompositeValue::Single(value.clone()),
            );
            Box::into_raw(Box::new(cloned)) as *mut KoiCompositeSingle
        }
        _ => ptr::null_mut(),
    }
}

/// Free composite single parameter
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeSingle_Del(single: *mut KoiCompositeSingle) {
    if single.is_null() {
        return;
    }

    unsafe { drop(Box::from_raw(single as *mut Parameter)) };
}

/// Set integer value in composite single
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeSingle_SetIntValue(
    single: *mut KoiCompositeSingle,
    value: i64,
) -> i32 {
    if single.is_null() {
        return -1;
    }

    let param = unsafe { &mut *(single as *mut Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Single(v)) => {
            *v = Value::Int(value);
            0
        }
        _ => -3,
    }
}

/// Get integer value from composite single
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeSingle_GetIntValue(
    single: *mut KoiCompositeSingle,
    out_value: *mut i64,
) -> i32 {
    if single.is_null() || out_value.is_null() {
        return -1;
    }

    let param = unsafe { &*(single as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Single(Value::Int(v))) => {
            unsafe { *out_value = *v };
            0
        }
        _ => -3,
    }
}

/// Set float value in composite single
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeSingle_SetFloatValue(
    single: *mut KoiCompositeSingle,
    value: f64,
) -> i32 {
    if single.is_null() {
        return -1;
    }

    let param = unsafe { &mut *(single as *mut Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Single(v)) => {
            *v = Value::Float(value);
            0
        }
        _ => -3,
    }
}

/// Get float value from composite single
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeSingle_GetFloatValue(
    single: *mut KoiCompositeSingle,
    out_value: *mut f64,
) -> i32 {
    if single.is_null() || out_value.is_null() {
        return -1;
    }

    let param = unsafe { &*(single as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Single(Value::Float(v))) => {
            unsafe { *out_value = *v };
            0
        }
        _ => -3,
    }
}

/// Set string value in composite single
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeSingle_SetStringValue(
    single: *mut KoiCompositeSingle,
    value: *const c_char,
) -> i32 {
    if single.is_null() || value.is_null() {
        return -1;
    }

    let value_str = match unsafe { CStr::from_ptr(value) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -2,
    };

    let param = unsafe { &mut *(single as *mut Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Single(v)) => {
            *v = Value::String(value_str);
            0
        }
        _ => -3,
    }
}

/// Set boolean value in composite single
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeSingle_SetBoolValue(
    single: *mut KoiCompositeSingle,
    value: i32,
) -> i32 {
    if single.is_null() {
        return -1;
    }

    let param = unsafe { &mut *(single as *mut Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Single(v)) => {
            *v = Value::Bool(value != 0);
            0
        }
        _ => -3,
    }
}

/// Get boolean value from composite single
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeSingle_GetBoolValue(
    single: *mut KoiCompositeSingle,
    out_value: *mut i32,
) -> i32 {
    if single.is_null() || out_value.is_null() {
        return -1;
    }

    let param = unsafe { &*(single as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Single(Value::Bool(v))) => {
            unsafe { *out_value = if *v { 1 } else { 0 } };
            0
        }
        _ => -3,
    }
}

/// Add composite single to command
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_AddCompositeSingle(
    command: *mut KoiCommand,
    single: *mut KoiCompositeSingle,
) -> i32 {
    if command.is_null() || single.is_null() {
        return -1;
    }

    let command = unsafe { &mut *(command as *mut Command) };
    let single_param = unsafe { Box::from_raw(single as *mut Parameter) };

    command.params.push(*single_param);
    0
}

/// Get value type from composite single
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCompositeSingle_GetValueType(single: *mut KoiCompositeSingle) -> i32 {
    if single.is_null() {
        return KoiParamType::Invalid as i32;
    }

    let param = unsafe { &*(single as *const Parameter) };
    match param {
        Parameter::Composite(_, CompositeValue::Single(v)) => match v {
            Value::Int(_) => KoiParamType::BasicInt as i32,
            Value::Float(_) => KoiParamType::BasicFloat as i32,
            Value::String(_) => KoiParamType::BasicString as i32,
            Value::Bool(_) => KoiParamType::BasicBool as i32,
        },
        _ => KoiParamType::Invalid as i32,
    }
}
