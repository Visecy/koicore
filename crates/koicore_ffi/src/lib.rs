//! # KoiCore FFI
//!
//! This crate provides a C-compatible foreign function interface (FFI) for the KoiCore library,
//! enabling C and other languages to interact with KoiLang parsing functionality.
//!
//! ## Features
//!
//! - Parse KoiLang text from various input sources (strings, files, custom callbacks)
//! - Access and manipulate KoiLang commands and parameters
//! - Handle composite data structures (lists and dictionaries)
//! - Comprehensive error handling with detailed error information
//!
//! ## Modules
//!
//! - [`command`]: Functions for creating and manipulating KoiLang commands
//! - [`parser`]: Functions for parsing KoiLang text and managing parser state
//!
//! ## Safety
//!
//! This FFI uses raw pointers and requires careful memory management. Users must:
//! - Always check for null pointers before dereferencing
//! - Properly free allocated objects using the provided `_Del` functions
//! - Ensure thread safety when using the same parser from multiple threads
//! - Follow the documentation for each function regarding parameter validation
//!
//! ## Example
//!
//! ```c
//! #include "koicore.h"
//!
//! // Create a parser from a string
//! const char* text = "#command param1 param2";
//! struct KoiInputSource* input = KoiInputSource_FromString(text);
//! struct KoiParserConfig* config = malloc(sizeof(struct KoiParserConfig));
//! KoiParserConfig_Init(config);
//!
//! struct KoiParser* parser = KoiParser_New(input, config);
//!
//! // Parse commands
//! struct KoiCommand* cmd = KoiParser_NextCommand(parser);
//! if (cmd) {
//!     // Process command
//!     KoiCommand_Del(cmd);
//! }
//!
//! // Clean up
//! KoiParser_Del(parser);
//! KoiInputSource_Del(input);
//! free(config);
//! ```

pub mod command;
pub mod parser;
pub mod writer;

#[cfg(test)]
mod tests {
    use crate::command::command::*;
    use crate::command::dict::*;
    use crate::command::list::*;
    use crate::command::single::*;
    use koicore::command::{Command, CompositeValue, Parameter, Value};
    use std::ffi::CString;

    #[test]
    fn test_ffi_composite_list() {
        unsafe {
            let cmd_name = CString::new("test_cmd").unwrap();
            let cmd = KoiCommand_New(cmd_name.as_ptr());

            let list_name = CString::new("my_list").unwrap();
            let list = KoiCompositeList_New(list_name.as_ptr());

            KoiCompositeList_AddIntValue(list, 42);
            KoiCommand_AddCompositeList(cmd, list);

            let command = &*(cmd as *mut Command);
            assert_eq!(command.name, "test_cmd");
            assert_eq!(command.params.len(), 1);

            if let Parameter::Composite(name, CompositeValue::List(values)) = &command.params[0] {
                assert_eq!(name, "my_list");
                assert_eq!(values.len(), 1);
                assert_eq!(values[0], Value::Int(42));
            } else {
                panic!("Expected composite list parameter");
            }

            KoiCommand_Del(cmd);
        }
    }

    #[test]
    fn test_ffi_composite_dict() {
        unsafe {
            let cmd_name = CString::new("test_cmd").unwrap();
            let cmd = KoiCommand_New(cmd_name.as_ptr());

            let dict_name = CString::new("my_dict").unwrap();
            let dict = KoiCompositeDict_New(dict_name.as_ptr());

            let key = CString::new("key").unwrap();
            KoiCompositeDict_SetIntValue(dict, key.as_ptr(), 123);
            KoiCommand_AddCompositeDict(cmd, dict);

            let command = &*(cmd as *mut Command);
            assert_eq!(command.params.len(), 1);

            if let Parameter::Composite(name, CompositeValue::Dict(entries)) = &command.params[0] {
                assert_eq!(name, "my_dict");
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, "key");
                assert_eq!(entries[0].1, Value::Int(123));
            } else {
                panic!("Expected composite dict parameter");
            }

            KoiCommand_Del(cmd);
        }
    }

    #[test]
    fn test_ffi_composite_single() {
        unsafe {
            let cmd_name = CString::new("test_cmd").unwrap();
            let cmd = KoiCommand_New(cmd_name.as_ptr());

            let single_name = CString::new("my_single").unwrap();
            let single = KoiCompositeSingle_New(single_name.as_ptr());

            KoiCompositeSingle_SetIntValue(single, 114);
            KoiCommand_AddCompositeSingle(cmd, single);

            let command = &*(cmd as *mut Command);
            assert_eq!(command.params.len(), 1);

            if let Parameter::Composite(name, CompositeValue::Single(value)) = &command.params[0] {
                assert_eq!(name, "my_single");
                assert_eq!(*value, Value::Int(114));
            } else {
                panic!("Expected composite single parameter");
            }

            KoiCommand_Del(cmd);
        }
    }
}
