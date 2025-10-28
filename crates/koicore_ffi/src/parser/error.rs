use std::{ffi::c_char, slice};

/// Opaque handle for KoiLang parser errors
///
/// This structure represents an error that occurred during parsing.
/// It contains detailed information about the error including the message
/// and position in the source text where the error occurred.
#[repr(C)]
pub struct KoiParserError {
    _data: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

/// Deletes a KoiParserError object and frees its memory
///
/// # Arguments
///
/// * `_this` - Pointer to the KoiParserError to delete. If NULL, the function does nothing.
///
/// # Safety
///
/// The pointer must either be NULL or point to a valid KoiParserError object
/// created by the parser functions. After calling this function, the pointer
/// becomes invalid and must not be used.
#[unsafe(no_mangle)]
pub extern "C" fn KoiParserError_Del(_this: *mut KoiParserError) {
    if _this.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(_this));
    }
}

/// Formats the error message into a buffer
///
/// This function creates a formatted error message including all available
/// error information and writes it to the provided buffer.
///
/// # Arguments
///
/// * `_this` - Pointer to the KoiParserError
/// * `buffer` - Buffer to write the formatted message to. If NULL, no data is written.
/// * `buffer_size` - Size of the buffer in bytes
///
/// # Returns
///
/// The total number of bytes required for the formatted message including the null terminator.
/// If the buffer is NULL or too small, no data is written and the required size is returned.
///
/// # Safety
///
/// The `_this` pointer must be either NULL or point to a valid KoiParserError.
/// If `buffer` is not NULL, it must point to a valid memory region of at least `buffer_size` bytes.
#[unsafe(no_mangle)]
pub extern "C" fn KoiParserError_Format(
    _this: *const KoiParserError,
    buffer: *mut c_char,
    buffer_size: usize,
) -> usize {
    if _this.is_null() {
        return 0;
    }
    let parser_error = unsafe { &*(_this as *const koicore::parser::ParseError) };
    let formatted = format!("{}", parser_error);
    let formatted_bytes = formatted.as_bytes();
    let formatted_len = formatted_bytes.len();
    
    let required_size = formatted_len + 1;
    
    if buffer.is_null() || buffer_size < required_size {
        return required_size;
    }
    
    let buffer_slice = unsafe { slice::from_raw_parts_mut(buffer as *mut u8, buffer_size) };
    buffer_slice[..formatted_len].copy_from_slice(formatted_bytes);
    buffer_slice[formatted_len] = 0;
    
    required_size
}

/// Gets the length of the formatted error message
///
/// This function returns the number of bytes required to store the formatted
/// error message including the null terminator, without actually formatting it.
///
/// # Arguments
///
/// * `_this` - Pointer to the KoiParserError
///
/// # Returns
///
/// The number of bytes required for the formatted message including the null terminator.
///
/// # Safety
///
/// The `_this` pointer must be either NULL or point to a valid KoiParserError.
#[unsafe(no_mangle)]
pub extern "C" fn KoiParserError_FormatLen(_this: *const KoiParserError) -> usize {
    if _this.is_null() {
        return 0;
    }
    let parser_error = unsafe { &*(_this as *const koicore::parser::ParseError) };
    let formatted = format!("{}", parser_error);
    let formatted_bytes = formatted.as_bytes();
    let formatted_len = formatted_bytes.len();

    formatted_len + 1
}

/// Gets the error message text
///
/// This function extracts just the message part of the error (without position
/// information) and writes it to the provided buffer.
///
/// # Arguments
///
/// * `_this` - Pointer to the KoiParserError
/// * `buffer` - Buffer to write the message to. If NULL, no data is written.
/// * `buffer_size` - Size of the buffer in bytes
///
/// # Returns
///
/// The total number of bytes required for the message including the null terminator.
/// If the buffer is NULL or too small, no data is written and the required size is returned.
///
/// # Safety
///
/// The `_this` pointer must be either NULL or point to a valid KoiParserError.
/// If `buffer` is not NULL, it must point to a valid memory region of at least `buffer_size` bytes.
#[unsafe(no_mangle)]
pub extern "C" fn KoiParserError_GetMessage(_this: *const KoiParserError, buffer: *mut c_char, buffer_size: usize) -> usize {
    if _this.is_null() {
        return 0;
    }
    let parser_error = unsafe { &*(_this as *const koicore::parser::ParseError) };
    let message = parser_error.message();
    let value_bytes = message.as_bytes();
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

/// Gets the length of the error message
///
/// This function returns the number of bytes required to store the error
/// message including the null terminator, without actually copying it.
///
/// # Arguments
///
/// * `_this` - Pointer to the KoiParserError
///
/// # Returns
///
/// The number of bytes required for the message including the null terminator.
///
/// # Safety
///
/// The `_this` pointer must be either NULL or point to a valid KoiParserError.
#[unsafe(no_mangle)]
pub extern "C" fn KoiParserError_GetMessageLen(_this: *const KoiParserError) -> usize {
    if _this.is_null() {
        return 0;
    }
    let parser_error = unsafe { &*(_this as *const koicore::parser::ParseError) };
    let message = parser_error.message();
    let value_bytes = message.as_bytes();
    let value_len = value_bytes.len();
    
    value_len + 1
}

/// Gets the position information from the error
///
/// This function extracts the line and column position where the error occurred
/// in the source text.
///
/// # Arguments
///
/// * `_this` - Pointer to the KoiParserError
/// * `lineno` - Pointer to store the line number (1-based). If NULL, the value is not stored.
/// * `column` - Pointer to store the column number (1-based). If NULL, the value is not stored.
///
/// # Returns
///
/// 0 on success, -1 if the error does not contain position information or if `_this` is NULL.
///
/// # Safety
///
/// The `_this` pointer must be either NULL or point to a valid KoiParserError.
/// If `lineno` and `column` are not NULL, they must point to valid memory locations.
#[unsafe(no_mangle)]
pub extern "C" fn KoiParserError_GetTracebackPosition(
    _this: *const KoiParserError,
    lineno: *mut usize,
    column: *mut usize,
) -> i32 {
    if _this.is_null() {
        return -1;
    }
    let parser_error = unsafe { &*(_this as *const koicore::parser::ParseError) };
    if let Some(pos) = parser_error.position() {
        unsafe {
            if !lineno.is_null() {
                *lineno = pos.0;
            }
            if !column.is_null() {
                *column = pos.1;
            }
        }
        0
    } else {
        -1
    }
}
