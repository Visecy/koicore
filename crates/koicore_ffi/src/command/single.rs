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

/// Get composite single parameter from command
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiCommand_GetCompositeSingle(
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
