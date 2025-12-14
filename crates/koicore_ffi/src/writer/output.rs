use std::ffi::c_void;
use std::io;

/// VTable for custom writer output
#[repr(C)]
pub struct KoiWriterOutputVTable {
    /// Function to write data
    ///
    /// # Arguments
    /// * `user_data` - User-provided data pointer
    /// * `buf` - Pointer to data buffer
    /// * `len` - Length of data buffer
    ///
    /// # Returns
    /// Number of bytes written. 0 implies error if len > 0.
    pub write: extern "C" fn(user_data: *mut c_void, buf: *const u8, len: usize) -> usize,

    /// Function to flush output
    ///
    /// # Arguments
    /// * `user_data` - User-provided data pointer
    ///
    /// # Returns
    /// 0 on success, non-zero on error
    pub flush: extern "C" fn(user_data: *mut c_void) -> i32,
}

pub struct CustomWriterOutput {
    vtable: *const KoiWriterOutputVTable,
    user_data: *mut c_void,
}

impl CustomWriterOutput {
    /// Create a new CustomWriterOutput
    ///
    /// # Safety
    /// vtable must be valid
    pub unsafe fn new(vtable: *const KoiWriterOutputVTable, user_data: *mut c_void) -> Self {
        Self { vtable, user_data }
    }
}

impl io::Write for CustomWriterOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = unsafe { ((*self.vtable).write)(self.user_data, buf.as_ptr(), buf.len()) };
        if written == 0 && !buf.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "failed to write to C callback",
            ));
        }
        Ok(written)
    }

    fn flush(&mut self) -> io::Result<()> {
        let res = unsafe { ((*self.vtable).flush)(self.user_data) };
        if res == 0 {
            Ok(())
        } else {
            Err(io::Error::from_raw_os_error(res))
        }
    }
}

unsafe impl Send for CustomWriterOutput {}

use std::ffi::c_char;
use std::ptr;
use std::sync::{Arc, RwLock};

/// String output buffer that can be shared with C
pub struct KoiStringOutput {
    pub buffer: Arc<RwLock<Vec<u8>>>,
}

/// Create a new String Output.
///
/// The returned object must be freed using `KoiStringOutput_Del`.
///
/// # Returns
///
/// A valid pointer to a `KoiStringOutput`.
#[unsafe(no_mangle)]
pub extern "C" fn KoiStringOutput_New() -> *mut KoiStringOutput {
    let output = KoiStringOutput {
        buffer: Arc::new(RwLock::new(Vec::new())),
    };
    Box::into_raw(Box::new(output))
}

/// Delete String Output.
///
/// Frees the memory allocated for the string output.
///
/// # Safety
///
/// * `output` must be a valid pointer returned by `KoiStringOutput_New`.
/// * After calling this function, `output` is invalid and must not be used.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiStringOutput_Del(output: *mut KoiStringOutput) {
    if !output.is_null() {
        unsafe {
            drop(Box::from_raw(output));
        }
    }
}

/// Get content of the string buffer.
///
/// # Arguments
///
/// * `output` - Pointer to the String Output object
/// * `buffer` - Pointer to destination buffer (can be NULL to query size)
/// * `buffer_len` - Size of the destination buffer
///
/// # Returns
///
/// The length of the string content PLUS null terminator.
/// If `buffer` is NULL or `buffer_len` is too small, returns required size.
/// If successful, copies content to `buffer` (null-terminated) and returns length.
/// Returns 0 if `output` is NULL.
/// 
/// # Safety
/// 
/// * `output` must be a valid pointer returned by `KoiStringOutput_New`.
/// * `buffer` must be a valid pointer to a writable buffer of at least `buffer_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiStringOutput_GetString(
    output: *mut KoiStringOutput,
    buffer: *mut c_char,
    buffer_len: usize,
) -> usize {
    if output.is_null() {
        return 0;
    }

    let output = unsafe { &*output };
    if let Ok(vec) = output.buffer.read() {
        let len = vec.len();
        if buffer.is_null() || buffer_len < len + 1 {
            return len + 1;
        }

        unsafe {
            ptr::copy_nonoverlapping(vec.as_ptr(), buffer as *mut u8, len);
            *buffer.add(len) = 0;
        }
        return len;
    }

    0
}

/// Wrapper for Vec<u8> that allows shared access via Arc<RwLock>
pub struct SharedBufferWriter {
    pub buffer: Arc<RwLock<Vec<u8>>>,
}

impl io::Write for SharedBufferWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut buffer = self
            .buffer
            .write()
            .map_err(|_| io::Error::other("Lock poisoned"))?;
        buffer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

unsafe impl Send for SharedBufferWriter {}
