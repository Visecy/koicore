mod error;
mod input;
mod config;

use std::ptr;

use koicore::parser::{TextInputSource, ParseError};
use koicore::Parser;

use error::KoiParserError;
use input::KoiInputSource;
use config::KoiParserConfig;
use crate::command::KoiCommand;

#[repr(C)]
pub struct KoiParser {
    inner: Parser<Box<dyn TextInputSource>>,
    last_error: Option<Box<ParseError>>,
    eof: bool,
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiParser_New(
    input: *mut KoiInputSource,
    config: *mut KoiParserConfig
) -> *mut KoiParser {
    if config.is_null() || input.is_null() {
        return ptr::null_mut();
    }
    
    let config = unsafe { &*(config as *mut KoiParserConfig) };
    let input: Box<KoiInputSource> = unsafe {
        Box::from_raw(input)
    };
    let parser = Parser::new(input.inner, config.clone().into());
    Box::into_raw(Box::new(parser)) as *mut KoiParser
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiParser_Del(parser: *mut KoiParser) {
    if !parser.is_null() {
        drop(unsafe { Box::from_raw(parser as *mut Parser<Box<dyn TextInputSource>>) });
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn KoiParser_NextCommand(
    parser: *mut KoiParser,
) -> *mut KoiCommand {
    if parser.is_null() {
        return ptr::null_mut();
    }

    let parser = unsafe { &mut *parser };
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
