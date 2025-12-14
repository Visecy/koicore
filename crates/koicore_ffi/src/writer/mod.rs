use koicore::WriterConfig;
use koicore::command::Command;
use koicore::writer::{FormatterOptions, ParamFormatSelector, Writer};
use std::collections::HashMap;
use std::ffi::{CStr, c_char, c_void};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ptr;

pub use self::config::{
    KoiCommandOption, KoiFormatterOptions, KoiNumberFormat, KoiParamFormatSelector, KoiParamOption,
    KoiWriterConfig,
};
use self::output::{
    CustomWriterOutput, KoiStringOutput, KoiWriterOutputVTable, SharedBufferWriter,
};
use crate::command::KoiCommand;

pub mod config;
pub mod output;

/// Opaque handle for KoiWriter
#[repr(C)]
pub struct KoiWriter {
    inner: Writer<Box<dyn Write + Send>>,
}

/// Create a new Writer with custom output VTable.
///
/// # Safety
///
/// * `vtable` must be a valid pointer to a `KoiWriterOutputVTable`.
/// * `config` must be a valid pointer to a `KoiWriterConfig`.
/// * The returned pointer must be freed using `KoiWriter_Del`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiWriter_NewFromVTable(
    vtable: *const KoiWriterOutputVTable,
    user_data: *mut c_void,
    config: *const KoiWriterConfig,
) -> *mut KoiWriter {
    if vtable.is_null() || config.is_null() {
        return ptr::null_mut();
    }

    let output = unsafe { CustomWriterOutput::new(vtable, user_data) };
    let config = unsafe { WriterConfig::from(&*config) };

    let boxed_output: Box<dyn Write + Send> = Box::new(output);
    let writer = Writer::new(boxed_output, config);

    Box::into_raw(Box::new(KoiWriter { inner: writer }))
}

/// Create a new Writer that writes to a file.
///
/// # Safety
///
/// * `path` must be a valid null-terminated C string.
/// * `config` must be a valid pointer to a `KoiWriterConfig`.
/// * The returned pointer must be freed using `KoiWriter_Del`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiWriter_NewFromFile(
    path: *const c_char,
    config: *const KoiWriterConfig,
) -> *mut KoiWriter {
    if path.is_null() || config.is_null() {
        return ptr::null_mut();
    }

    let path_str = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let file = match File::create(path_str) {
        Ok(f) => f,
        Err(_) => return ptr::null_mut(),
    };

    let config = unsafe { WriterConfig::from(&*config) };
    // Use BufWriter for performance
    let boxed_output: Box<dyn Write + Send> = Box::new(BufWriter::new(file));
    let writer = Writer::new(boxed_output, config);

    Box::into_raw(Box::new(KoiWriter { inner: writer }))
}

/// Create a new Writer that writes to a string output.
///
/// # Safety
///
/// * `output` must be a valid pointer to a `KoiStringOutput`.
/// * `config` must be a valid pointer to a `KoiWriterConfig`.
/// * The returned pointer must be freed using `KoiWriter_Del`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiWriter_NewFromStringOutput(
    output: *mut KoiStringOutput,
    config: *const KoiWriterConfig,
) -> *mut KoiWriter {
    if output.is_null() || config.is_null() {
        return ptr::null_mut();
    }

    let output_obj = unsafe { &*output };
    let buffer_writer = SharedBufferWriter {
        buffer: output_obj.buffer.clone(),
    };

    let config = unsafe { WriterConfig::from(&*config) };
    let boxed_output: Box<dyn Write + Send> = Box::new(buffer_writer);
    let writer = Writer::new(boxed_output, config);

    Box::into_raw(Box::new(KoiWriter { inner: writer }))
}

/// Delete Writer.
///
/// Frees the memory allocated for the writer.
///
/// # Safety
///
/// * `writer` must be a valid pointer returned by one of the `KoiWriter_New*` functions.
/// * After calling this function, `writer` is invalid and must not be used.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiWriter_Del(writer: *mut KoiWriter) {
    if !writer.is_null() {
        unsafe {
            drop(Box::from_raw(writer));
        }
    }
}

/// Write a command.
///
/// # Safety
///
/// * `writer` must be a valid pointer to a `KoiWriter`.
/// * `command` must be a valid pointer to a `KoiCommand`.
///
/// # Returns
///
/// * 0 on success
/// * -1 if arguments are null
/// * -2 if writing fails
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiWriter_WriteCommand(
    writer: *mut KoiWriter,
    command: *const KoiCommand,
) -> i32 {
    if writer.is_null() || command.is_null() {
        return -1;
    }

    let writer = unsafe { &mut *writer };
    let command = unsafe { &*(command as *const Command) };

    match writer.inner.write_command(command) {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

/// Helper to convert param options
unsafe fn parse_param_options(
    ptr: *const KoiParamOption,
) -> HashMap<ParamFormatSelector, FormatterOptions> {
    unsafe {
        let mut map = HashMap::new();
        if ptr.is_null() {
            return map;
        }

        let mut current = ptr;
        // Terminate when selector.name is NULL AND selector.is_position is false
        // since is_position is bool, we check if it is explicitly false. Actually,
        // safe bet is to require name to be NULL, and we ignore if is_position was set true
        // but name NULL is weird for named usage. Let's strictly follow protocol:
        // terminator: name == NULL. If is_position=true, name ignored anyway?
        // Actually the struct definition has name as *const char.
        // If is_position is true, we use position.
        // So terminator must be distinguishable.
        // Let's rely on name being NULL effectively meaning "End" IF is_position is also false.
        // Or just name being NULL is enough? If is_position is true, name is ignored.
        // But for terminator, we usually set everything to 0/NULL.
        while !(*current).selector.name.is_null() || (*current).selector.is_position {
            // If name is NULL but is_position is true, it is a VALID entry (positional).
            // So loop condition: !(name is NULL AND !is_position)
            // Checks:
            // if name == NULL && !is_position => break

            let sel = (*current).selector;
            if sel.name.is_null() && !sel.is_position {
                break;
            }

            let selector = if sel.is_position {
                ParamFormatSelector::Position(sel.position)
            } else {
                let name_str = CStr::from_ptr(sel.name).to_string_lossy().into_owned();
                ParamFormatSelector::Name(name_str)
            };

            map.insert(selector, (*current).options.into());
            current = current.add(1);
        }
        map
    }
}

/// Write a command with custom options.
///
/// # Safety
///
/// * `writer` must be a valid pointer to a `KoiWriter`.
/// * `command` must be a valid pointer to a `KoiCommand`.
/// * `options` can be null (uses defaults).
/// * `param_options` can be null (uses defaults).
///
/// # Returns
///
/// * 0 on success
/// * -1 if writer or command are null
/// * -2 if writing fails
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiWriter_WriteCommandWithOptions(
    writer: *mut KoiWriter,
    command: *const KoiCommand,
    options: *const KoiFormatterOptions,
    param_options: *const KoiParamOption,
) -> i32 {
    if writer.is_null() || command.is_null() {
        return -1;
    }

    let writer = unsafe { &mut *writer };
    let command = unsafe { &*(command as *const Command) };

    let options: Option<FormatterOptions> = if options.is_null() {
        None
    } else {
        Some(unsafe { (*options).into() })
    };

    let param_options_map = if param_options.is_null() {
        None
    } else {
        Some(unsafe { parse_param_options(param_options) })
    };

    // We need to pass reference to options
    let options_ref = options.as_ref();
    let param_options_ref = param_options_map.as_ref();

    match writer
        .inner
        .write_command_with_options(command, options_ref, param_options_ref)
    {
        Ok(_) => 0,
        Err(_) => -2,
    }
}

/// Increase indentation.
///
/// # Safety
///
/// * `writer` must be a valid pointer to a `KoiWriter`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiWriter_IncIndent(writer: *mut KoiWriter) {
    if !writer.is_null() {
        let writer = unsafe { &mut *writer };
        writer.inner.inc_indent();
    }
}

/// Decrease indentation.
///
/// # Safety
///
/// * `writer` must be a valid pointer to a `KoiWriter`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiWriter_DecIndent(writer: *mut KoiWriter) {
    if !writer.is_null() {
        let writer = unsafe { &mut *writer };
        writer.inner.dec_indent();
    }
}

/// Get current indentation.
///
/// # Safety
///
/// * `writer` must be a valid pointer to a `KoiWriter`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiWriter_GetIndent(writer: *const KoiWriter) -> usize {
    if !writer.is_null() {
        let writer = unsafe { &*writer };
        writer.inner.get_indent()
    } else {
        0
    }
}

/// Write a newline.
///
/// # Safety
///
/// * `writer` must be a valid pointer to a `KoiWriter`.
///
/// # Returns
///
/// * 0 on success
/// * -1 if writer is null
/// * -2 if writing fails
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiWriter_Newline(writer: *mut KoiWriter) -> i32 {
    if !writer.is_null() {
        let writer = unsafe { &mut *writer };
        match writer.inner.newline() {
            Ok(_) => 0,
            Err(_) => -2,
        }
    } else {
        -1
    }
}
