use std::ffi::{ c_char, c_void, CStr };
use std::ptr;
use std::io;

use encoding_rs::Encoding;
use koicore::parser::input::{ StringInputSource, FileInputSource, EncodingErrorStrategy };
use koicore::parser::TextInputSource;

/// Opaque handle for KoiLang input sources
///
/// This structure represents an input source that provides text to the parser.
/// It can be created from strings, files, or custom callback functions.
#[repr(C)]
pub struct KoiInputSource {
    pub(super) inner: Box<dyn TextInputSource>,
}

/// Strategy for handling encoding errors when reading files
///
/// This enum determines how the parser handles invalid byte sequences
/// when reading files with a specific encoding.
#[repr(C)]
pub enum KoiFileInputEncodingStrategy {
    /// Strict encoding error strategy, panics on invalid sequences
    Strict = 0,
    /// Replace invalid sequences with the replacement character (U+FFFD)
    Replace = 1,
    /// Ignore invalid sequences
    Ignore = 2,
}

/// VTable for custom text input sources
///
/// This structure provides function pointers for implementing custom input sources
/// in C or other languages. The user must provide implementations of these functions.
#[repr(C)]
pub struct KoiTextInputSourceVTable {
    /// Function to get the next line of text
    /// 
    /// # Arguments
    /// 
    /// * `user_data` - User-provided data pointer
    /// 
    /// # Returns
    /// 
    /// Pointer to a null-terminated C string containing the next line,
    /// or NULL if there are no more lines or an error occurred.
    /// The string must remain valid until the next call or until the input source is destroyed.
    next_line: extern "C" fn(user_data: *mut c_void) -> *mut c_char,
    
    /// Function to get the name of the input source
    /// 
    /// # Arguments
    /// 
    /// * `user_data` - User-provided data pointer
    /// 
    /// # Returns
    /// 
    /// Pointer to a null-terminated C string containing the source name,
    /// or NULL if no name is available.
    source_name: extern "C" fn(user_data: *mut c_void) -> *const c_char,
}

struct CustomTextInputSource {
    vtable: *const KoiTextInputSourceVTable,
    user_data: *mut c_void,
}

impl TextInputSource for CustomTextInputSource {
    fn next_line(&mut self) -> io::Result<Option<String>> {
        let line_ptr = unsafe { ((*self.vtable).next_line)(self.user_data) };

        if !line_ptr.is_null() {
            let c_str = unsafe { CStr::from_ptr(line_ptr) };
            let line = c_str.to_string_lossy().into_owned();
            return Ok(Some(line));
        }

        let errno = errno::errno().into();
        if errno == 0 {
            Ok(None)
        } else {
            Err(io::Error::from_raw_os_error(errno))
        }
    }

    fn source_name(&self) -> &str {
        let name_ptr = unsafe { ((*self.vtable).source_name)(self.user_data) };
        if name_ptr.is_null() {
            "<string>"
        } else {
            let c_str = unsafe { CStr::from_ptr(name_ptr) };
            c_str.to_str().unwrap_or("<string>")
        }
    }
}

/// Creates a new input source from a custom VTable implementation
///
/// This function allows creating custom input sources by providing function pointers
/// for reading lines and getting the source name. This is useful for integrating
/// with existing C code or implementing special input handling.
///
/// # Arguments
///
/// * `vtable` - Pointer to a KoiTextInputSourceVTable structure containing function pointers
/// * `user_data` - User-provided data pointer that will be passed to the VTable functions
///
/// # Returns
///
/// Pointer to the created KoiInputSource, or NULL if vtable is NULL.
///
/// # Safety
///
/// The `vtable` pointer must be valid and remain valid until the input source is destroyed.
/// The function pointers in the vtable must follow the specified contracts.
/// The `user_data` pointer is passed directly to the VTable functions and is not managed by this library.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiInputSource_FromVTable(
    vtable: *const KoiTextInputSourceVTable,
    user_data: *mut c_void
) -> *mut KoiInputSource {
    if vtable.is_null() {
        return ptr::null_mut();
    }

    let source = Box::new(CustomTextInputSource {
        vtable,
        user_data,
    });

    let wrapper = Box::new(KoiInputSource {
        inner: source as Box<dyn TextInputSource>,
    });

    Box::into_raw(wrapper)
}

/// Creates a new input source from a null-terminated C string
///
/// This function creates an input source that will provide the specified string
/// to the parser. The string is copied and can be safely freed after this call.
///
/// # Arguments
///
/// * `source` - Pointer to a null-terminated UTF-8 C string containing the text to parse
///
/// # Returns
///
/// Pointer to the created KoiInputSource, or NULL if source is NULL or contains invalid UTF-8.
///
/// # Safety
///
/// The `source` pointer must be either NULL or point to a valid null-terminated C string.
/// The string should contain valid UTF-8, but invalid sequences will be replaced with the Unicode replacement character.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiInputSource_FromString(source: *const c_char) -> *mut KoiInputSource {
    if source.is_null() {
        return ptr::null_mut();
    }

    let source_str = match (unsafe { CStr::from_ptr(source) }).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            return ptr::null_mut();
        }
    };

    let input = StringInputSource::new(&source_str);
    let input_source = Box::new(input);
    let source_wrapper = Box::new(KoiInputSource { inner: input_source });
    Box::into_raw(source_wrapper)
}

/// Creates a new input source from a file path
///
/// This function creates an input source that will read from the specified file.
/// The file is opened with UTF-8 encoding and strict error handling.
///
/// # Arguments
///
/// * `path` - Pointer to a null-terminated C string containing the file path
///
/// # Returns
///
/// Pointer to the created KoiInputSource, or NULL if path is NULL, contains invalid UTF-8,
/// or the file cannot be opened.
///
/// # Safety
///
/// The `path` pointer must be either NULL or point to a valid null-terminated C string.
/// The file must exist and be readable with UTF-8 encoding.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiInputSource_FromFile(path: *const c_char) -> *mut KoiInputSource {
    if path.is_null() {
        return ptr::null_mut();
    }

    let source_str = match (unsafe { CStr::from_ptr(path) }).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            return ptr::null_mut();
        }
    };

    let input = FileInputSource::new(source_str);
    if input.is_err() {
        return ptr::null_mut();
    }

    let input_source = Box::new(input.unwrap());
    let source_wrapper = Box::new(KoiInputSource { inner: input_source });
    Box::into_raw(source_wrapper)
}

/// Creates a new input source from a file path with specific encoding
///
/// This function creates an input source that will read from the specified file
/// using the specified text encoding and error handling strategy.
///
/// # Arguments
///
/// * `path` - Pointer to a null-terminated C string containing the file path
/// * `encoding` - Pointer to a null-terminated C string containing the encoding name
/// * `encoding_strategy` - Strategy for handling encoding errors
///
/// # Returns
///
/// Pointer to the created KoiInputSource, or NULL if path or encoding is NULL,
/// contains invalid UTF-8, the encoding is not recognized, or the file cannot be opened.
///
/// # Safety
///
/// The `path` and `encoding` pointers must be either NULL or point to valid null-terminated C strings.
/// The file must exist and be readable with the specified encoding.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiInputSource_FromFileAndEncoding(
    path: *const c_char,
    encoding: *const c_char,
    encoding_strategy: KoiFileInputEncodingStrategy
) -> *mut KoiInputSource {
    if path.is_null() || encoding.is_null() {
        return ptr::null_mut();
    }

    let path_str = match (unsafe { CStr::from_ptr(path) }).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            return ptr::null_mut();
        }
    };

    let encoding_str = match (unsafe { CStr::from_ptr(encoding) }).to_str() {
        Ok(s) => s.to_string(),
        Err(_) => {
            return ptr::null_mut();
        }
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

    let input = FileInputSource::with_encoding(path_str, encoding, strategy);
    if input.is_err() {
        return ptr::null_mut();
    }

    let input_source = Box::new(input.unwrap());
    let source_wrapper = Box::new(KoiInputSource { inner: input_source });
    Box::into_raw(source_wrapper)
}

/// Deletes a KoiInputSource object and frees its memory
///
/// # Arguments
///
/// * `input` - Pointer to the KoiInputSource to delete. If NULL, the function does nothing.
///
/// # Safety
///
/// The pointer must either be NULL or point to a valid KoiInputSource object
/// created by one of the KoiInputSource_From* functions. After calling this function,
/// the pointer becomes invalid and must not be used.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiInputSource_Del(input: *mut KoiInputSource) {
    if input.is_null() {
        return;
    }
    unsafe { drop(Box::from_raw(input)); }
}
