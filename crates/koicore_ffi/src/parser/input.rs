use std::ffi::{c_char, CStr};
use std::ptr;

use encoding_rs::Encoding;
use koicore::parser::input::{StringInputSource, FileInputSource, EncodingErrorStrategy};
use koicore::parser::TextInputSource;

#[repr(C)]
pub struct KoiInputSource {
    pub(super) inner: Box<dyn TextInputSource>,
}

#[repr(C)]
pub enum KoiFileInputEncodingStrategy {
    /// Strict encoding error strategy, panics on invalid sequences
    Strict = 0,
    /// Replace invalid sequences with the replacement character (U+FFFD)
    Replace = 1,
    /// Ignore invalid sequences
    Ignore = 2,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiInputSource_FromString(
    source: *const c_char,
) -> *mut KoiInputSource {
    if source.is_null() {
        return ptr::null_mut();
    }
    
    let source_str = match unsafe { CStr::from_ptr(source) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };
    
    let input = StringInputSource::new(&source_str);
    let input_source = Box::new(input);
    let source_wrapper = Box::new(KoiInputSource { inner: input_source });
    Box::into_raw(source_wrapper) as *mut KoiInputSource
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiInputSource_FromFile(
    path: *const c_char,
) -> *mut KoiInputSource {
    if path.is_null() {
        return ptr::null_mut();
    }
    
    let source_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let input = FileInputSource::new(source_str);
    if input.is_err() {
        return ptr::null_mut();
    }
    
    let input_source = Box::new(input.unwrap());
    let source_wrapper = Box::new(KoiInputSource { inner: input_source });
    Box::into_raw(source_wrapper) as *mut KoiInputSource
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiInputSource_FromFileAndEncoding(
    path: *const c_char,
    encoding: *const c_char,
    encoding_strategy: KoiFileInputEncodingStrategy,
) -> *mut KoiInputSource {
    if path.is_null() || encoding.is_null() {
        return ptr::null_mut();
    }

    let path_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let encoding_str = match unsafe { CStr::from_ptr(encoding) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return ptr::null_mut(),
    };

    let encoding = Encoding::for_label(encoding_str.as_bytes());
    if encoding.is_none() {
        return ptr::null_mut();
    }

    let strategy = match encoding_strategy {
        KoiFileInputEncodingStrategy::Strict => EncodingErrorStrategy::Strict,
        KoiFileInputEncodingStrategy::Replace => EncodingErrorStrategy::Replace,
        KoiFileInputEncodingStrategy::Ignore => EncodingErrorStrategy::Ignore,
    };

    let input = FileInputSource::with_encoding(
        path_str,
        encoding,
        strategy,
    );
    if input.is_err() {
        return ptr::null_mut();
    }
    
    let input_source = Box::new(input.unwrap());
    let source_wrapper = Box::new(KoiInputSource { inner: input_source });
    Box::into_raw(source_wrapper) as *mut KoiInputSource
}
