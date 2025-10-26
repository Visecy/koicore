use std::{ffi::c_char, slice};

#[repr(C)]
pub struct KoiParserError {
    _data: (),
    _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
}

#[unsafe(no_mangle)]
pub extern "C" fn KoiParserError_Del(_this: *mut KoiParserError) {
    if _this.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(_this));
    }
}

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
