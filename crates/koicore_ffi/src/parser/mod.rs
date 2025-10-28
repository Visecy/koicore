//! # Parser Module
//!
//! This module provides FFI functions for parsing KoiLang text and managing parser state.
//! It allows C and other foreign languages to parse KoiLang documents and extract commands.
//!
//! ## Main Components
//!
//! - [`KoiParser`]: Main parser structure for parsing KoiLang text
//! - [`KoiParserConfig`]: Configuration options for parser behavior
//! - [`KoiParserError`]: Error handling for parsing operations
//! - [`KoiInputSource`]: Input source abstraction for different data sources
//!
//! ## Usage Pattern
//!
//! 1. Create an input source from a string, file, or custom callback
//! 2. Configure parser options using `KoiParserConfig`
//! 3. Create a parser with `KoiParser_New`
//! 4. Iteratively retrieve commands with `KoiParser_NextCommand`
//! 5. Check for errors with `KoiParser_Error` if needed
//! 6. Clean up resources with the appropriate `_Del` functions
//!
//! ## Thread Safety
//!
//! Each `KoiParser` instance is not thread-safe and should only be used from a single thread.
//! Multiple parser instances can be used concurrently from different threads.

mod error;
mod input;
mod config;

use std::ptr;

use koicore::parser::{TextInputSource, ParseError};
use koicore::Parser;

use crate::command::KoiCommand;
pub use config::KoiParserConfig;
pub use error::KoiParserError;
pub use input::{KoiInputSource, KoiFileInputEncodingStrategy};

/// Opaque handle for KoiLang parser
///
/// This structure represents a parser instance that can parse KoiLang text from various sources.
/// The parser maintains internal state including the current position, error information,
/// and end-of-file status.
#[repr(C)]
pub struct KoiParser {
    inner: Parser<Box<dyn TextInputSource>>,
    last_error: Option<Box<ParseError>>,
    eof: bool,
}

/// Create a new KoiLang parser
///
/// Creates a new parser instance with the provided input source and configuration.
/// This function takes ownership of the input source, which means the caller should
/// not use or free the input source after this call. The parser will manage the input
/// source's lifecycle.
///
/// # Arguments
/// * `input` - Input source pointer (ownership is transferred to the parser)
/// * `config` - Parser configuration pointer
///
/// # Returns
/// Pointer to the new parser instance, or null on error:
/// - null if config is null
/// - null if input is null
///
/// # Safety
/// The input pointer must be a valid KoiInputSource created with one of the
/// KoiInputSource_From* functions. After calling this function, the input pointer
/// becomes invalid and must not be used or freed.
/// The config pointer must be a valid KoiParserConfig created with KoiParserConfig_New.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiParser_New(
    input: *mut KoiInputSource,
    config: *mut KoiParserConfig
) -> *mut KoiParser {
    if config.is_null() || input.is_null() {
        return ptr::null_mut();
    }
    
    let config = unsafe { &*config };
    let input: Box<KoiInputSource> = unsafe {
        Box::from_raw(input)
    };
    let parser = Parser::new(input.inner, config.into());
    Box::into_raw(Box::new(
        KoiParser { inner: parser, last_error: None, eof: false }
    ))
}

/// Delete a KoiLang parser and free its resources
///
/// This function destroys a parser instance and releases all associated memory,
/// including the input source that was transferred to it during creation.
/// After calling this function, the parser pointer becomes invalid and must not
/// be used.
///
/// # Arguments
/// * `parser` - Parser pointer to delete
///
/// # Safety
/// The parser pointer must be a valid KoiParser created with KoiParser_New.
/// After this call, the parser pointer becomes invalid and must not be used.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiParser_Del(parser: *mut KoiParser) {
    if !parser.is_null() {
        drop(unsafe { Box::from_raw(parser)});
    }
}

/// Get the next command from the parser
///
/// Retrieves the next command from the input source. Returns null when the end of
/// the input is reached or when an error occurs. If an error occurs, the error
/// can be retrieved using KoiParser_Error.
///
/// # Arguments
/// * `parser` - Parser pointer
///
/// # Returns
/// Pointer to the next command, or null in these cases:
/// - null if parser is null
/// - null if end of input is reached
/// - null if a parsing error occurred
///
/// # Safety
/// The parser pointer must be a valid KoiParser created with KoiParser_New.
/// The returned command pointer is owned by the caller and must be freed with
/// KoiCommand_Del when no longer needed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiParser_NextCommand(
    parser: *mut KoiParser,
) -> *mut KoiCommand {
    if parser.is_null() {
        return ptr::null_mut();
    }

    let parser = unsafe { &mut *parser };
    if parser.eof {
        return ptr::null_mut();
    }
    let inner = &mut parser.inner;
    let command = inner.next_command();
    match command {
        Ok(Some(command)) => Box::into_raw(Box::new(command)) as *mut KoiCommand,
        Ok(None) => {
            parser.eof = true;
            ptr::null_mut()
        },
        Err(error) => {
            parser.last_error = Some(error);
            ptr::null_mut()
        }
    }
}

/// Get the last parsing error from the parser
///
/// Retrieves the last error that occurred during parsing, if any. This function
/// transfers ownership of the error to the caller, so subsequent calls will
/// return null until another error occurs.
///
/// # Arguments
/// * `parser` - Parser pointer
///
/// # Returns
/// Pointer to the last error, or null in these cases:
/// - null if parser is null
/// - null if no error has occurred
/// - null if the error has already been retrieved
///
/// # Safety
/// The parser pointer must be a valid KoiParser created with KoiParser_New.
/// The returned error pointer is owned by the caller and must be freed with
/// KoiParserError_Del when no longer needed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiParser_Error(parser: *mut KoiParser) -> *mut KoiParserError {
    if parser.is_null() {
        return ptr::null_mut();
    }

    let parser = unsafe { &mut *parser };
    let error = parser.last_error.take();
    if error.is_none() {
        return ptr::null_mut();
    }
    Box::into_raw(error.unwrap()) as *mut KoiParserError
}
