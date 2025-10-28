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
