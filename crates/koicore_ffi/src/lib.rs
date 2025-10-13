use std::cell::RefCell;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

use koicore::{
    parser::{
        StringInputSource,
    },
    Parser, ParserConfig, Command, ParseError,
};

thread_local! {
    static LAST_ERROR: RefCell<Option<Box<ParseError>>> = RefCell::new(None);
}

#[repr(C)]
pub struct KoiParser {
    _private: [u8; 0],
}

#[repr(C)]
pub struct KoiCommand {
    _private: [u8; 0],
}

#[repr(C)]
pub struct KoiError {
    pub message: *const c_char,
    pub line: usize,
    pub column: usize,
}

pub(crate) fn set_last_error(error: Box<ParseError>) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(error);
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn koi_parser_new(
    source: *const c_char,
    command_threshold: usize,
) -> *mut KoiParser {
    let source_str = match CStr::from_ptr(source).to_str() {
        Ok(s) => s,
        Err(_) => {
            set_last_error(ParseError::syntax("Invalid UTF-8 in source string".to_string()));
            return std::ptr::null_mut();
        },
    };

    let config = ParserConfig { command_threshold };
    let input = StringInputSource::new(source_str);
    let parser = Box::new(Parser::new(input, config));
    Box::into_raw(parser) as *mut KoiParser
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn koi_parser_free(parser: *mut KoiParser) {
    if !parser.is_null() {
        drop(Box::from_raw(parser as *mut Parser<StringInputSource>));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn koi_parser_next_command(
    parser: *mut KoiParser,
) -> *mut KoiCommand {
    let parser = &mut *(parser as *mut Parser<StringInputSource>);
    match parser.next_command() {
        Ok(Some(cmd)) => {
            koi_clear_last_error();
            Box::into_raw(Box::new(cmd)) as *mut KoiCommand
        },
        Ok(None) => {
            koi_clear_last_error();
            std::ptr::null_mut()
        },
        Err(e) => {
            set_last_error(e);
            std::ptr::null_mut()
        },
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn koi_command_name(cmd: *const KoiCommand) -> *const c_char {
    let cmd = &*(cmd as *const Command);
    CString::new(cmd.name()).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn koi_command_free(cmd: *mut KoiCommand) {
    if !cmd.is_null() {
        drop(Box::from_raw(cmd as *mut Command));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn koi_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn koi_get_last_error() -> *const KoiError {
    LAST_ERROR.with(|e| {
        let error_ref = e.borrow();
        match error_ref.as_ref() {
            Some(err) => {
                let message = CString::new(err.message()).unwrap();
                let (line, column) = err.position().unwrap_or((0, 0));

                let koi_error = Box::new(KoiError {
                    message: message.into_raw(),
                    line,
                    column,
                });

                Box::into_raw(koi_error) as *const KoiError
            }
            None => std::ptr::null()
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn koi_clear_last_error() {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = None;
    });
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn koi_error_free(error: *mut KoiError) {
    if !error.is_null() {
        let err = Box::from_raw(error);
        if !err.message.is_null() {
            drop(CString::from_raw(err.message as *mut c_char));
        }
    }
}