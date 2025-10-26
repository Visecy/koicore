//! Error types for KoiLang parsing
//! 
//! This module defines the error types used throughout the parsing process.
//! It provides a comprehensive error handling system that includes:
//! 
//! - Error classification through the `ErrorInfo` enum
//! - Source location tracking via `ParserLineSource`
//! - Detailed traceback information with `TracebackEntry`
//! - User-friendly error display with the `Display` implementation
//! 
//! The main error type is `ParseError`, which combines semantic error information
//! with optional traceback and source location data. This allows for precise
//! error reporting that includes file names, line numbers, column positions,
//! and visual indicators of where in the source code the error occurred.

use std::fmt;
use std::io;

use crate::parser::input::Input;
use crate::parser::traceback::TracebackEntry;
use crate::parser::NomErrorNode;
use crate::parser::TextInputSource;

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, Box<ParseError>>;

/// Semantic error information without positional data
/// 
/// This enum represents the different types of errors that can occur during parsing,
/// without any information about where in the input the error occurred.
/// Position information is handled separately in the ParseError struct.
#[derive(Debug)]
pub enum ErrorInfo {
    /// Syntax error in the input
    /// 
    /// Used when the input doesn't conform to the expected syntax rules.
    SyntaxError {
        /// Error message describing the syntax issue
        message: String,
    },
    
    /// Unexpected input
    /// 
    /// Used when the parser encounters input that doesn't match the expected pattern.
    UnexpectedInput {
        /// Remaining unparsed input that caused the error
        remaining: String,
    },
    
    /// Unexpected end of input
    /// 
    /// Used when the parser reaches the end of input before finding the expected token or structure.
    UnexpectedEof {
        /// Description of what was expected when EOF was encountered
        expected: String,
    },
    
    /// IO error (for file-based parsing)
    /// 
    /// Used when file operations or other IO operations fail during parsing.
    IoError {
        /// The underlying IO error that occurred
        error: io::Error,
    },
}

/// Information about the source of a parsed line
/// 
/// This struct contains metadata about where a line of code originated from,
/// including the file path, line number, and the actual text content.
/// This information is used for error reporting and debugging.
#[derive(Debug, Clone)]
pub struct ParserLineSource {
    /// Source file path
    /// 
    /// The path to the file where the line originated from.
    /// This could be a file path, URL, or any other identifier for the source.
    pub filename: String,
    
    /// Line number in the source file
    /// 
    /// The line number where the error occurred, starting from 1.
    pub lineno: usize,
    
    /// The input line content
    /// 
    /// The actual text content of the line where the error occurred.
    /// This is used to display the problematic code in error messages.
    pub text: String,
}

/// Combined error type containing both semantic error information and traceback
#[derive(Debug)]
pub struct ParseError {
    /// The semantic error information
    pub error_info: ErrorInfo,
    /// Optional traceback information showing the parsing context
    pub traceback: Option<TracebackEntry>,
    /// Optional source information including filename, line number, and text content
    pub source: Option<ParserLineSource>,
}

impl ParseError {
    /// Create a new syntax error
    /// 
    /// This error is used when there's a syntax issue in the input without additional context.
    /// 
    /// # Arguments
    /// * `message` - Error message describing the syntax issue
    /// 
    /// # Returns
    /// A boxed ParseError with syntax error information
    pub fn syntax(message: String) -> Box<Self> {
        Box::new(ParseError {
            error_info: ErrorInfo::SyntaxError {
                message,
            },
            traceback: None,
            source: None,
        })
    }

    /// Create a new syntax error with context
    /// 
    /// This error is used when there's a syntax issue and additional context is available.
    /// 
    /// # Arguments
    /// * `message` - Error message describing the syntax issue
    /// * `line` - The line number where the error occurred
    /// * `column` - The column number where the error occurred
    /// * `context` - Additional context information about the error
    /// 
    /// # Returns
    /// A boxed ParseError with syntax error information and traceback context
    pub fn syntax_with_context(message: String, line: usize, column: usize, context: String) -> Box<Self> {
        Box::new(ParseError {
            error_info: ErrorInfo::SyntaxError {
                message,
            },
            traceback: Some(TracebackEntry::new(line, (column, column + 1), context)),
            source: None,
        })
    }
    /// Create a new unexpected input error
    /// 
    /// This error is used when the parser encounters input that doesn't match the expected pattern.
    /// 
    /// # Arguments
    /// * `remaining` - The remaining unparsed input that caused the error
    /// * `line` - The line number where the error occurred
    /// * `column_offset` - The column offset from the beginning of the line
    /// * `input` - The complete input string for calculating the column position
    /// 
    /// # Returns
    /// A boxed ParseError with unexpected input information and traceback
    pub fn unexpected_input(remaining: String, line: usize, column_offset: usize, input: String) -> Box<Self> {
        let column = input.len() - remaining.len() + 1 + column_offset;
        Box::new(ParseError {
            traceback: Some(
                TracebackEntry::new(line, (column, column + remaining.len()),
            "".to_string())
            ),
            error_info: ErrorInfo::UnexpectedInput {
                remaining,
            },
            source: None,
        })
    }

    /// Create a new unexpected EOF error
    /// 
    /// This error is used when the parser reaches the end of input before finding the expected token or structure.
    /// 
    /// # Arguments
    /// * `expected` - Description of what was expected when EOF was encountered
    /// * `line` - The line number where the error occurred
    /// * `column_offset` - The column offset from the beginning of the line
    /// 
    /// # Returns
    /// A boxed ParseError with unexpected EOF information and traceback
    pub fn unexpected_eof(expected: String, line: usize, column_offset: usize) -> Box<Self> {
        Box::new(ParseError {
            error_info: ErrorInfo::UnexpectedEof {
                expected,
            },
            traceback: Some(TracebackEntry::new(line, (column_offset, column_offset), "".to_string())),
            source: None,
        })
    }

    /// Create a new IO error from an io::Error
    /// 
    /// This error is used when file operations or other IO operations fail during parsing.
    /// 
    /// # Arguments
    /// * `error` - The underlying IO error that occurred
    /// 
    /// # Returns
    /// A boxed ParseError with IO error information
    pub fn io(error: io::Error) -> Box<Self> {
        Box::new(ParseError {
            error_info: ErrorInfo::IoError {
                error,
            },
            traceback: None,
            source: None,
        })
    }

    /// Create a syntax error from nom error
    /// 
    /// # Arguments
    /// * `message` - Custom error message describing the syntax issue
    /// * `original_input` - The complete input that was being parsed
    /// * `lineno` - The line number where the error occurred
    /// * `column` - The column number where the error occurred
    /// * `nom_error` - The nom error node containing detailed parsing information
    /// 
    /// # Returns
    /// A boxed ParseError with syntax error information and traceback
    pub(super) fn from_nom_error<I: core::ops::Deref<Target = str> + nom::Input>(
        message: String,
        original_input: I,
        lineno: usize,
        column: usize,
        nom_error: NomErrorNode<I>,
    ) -> Box<Self> {
        let traceback = TracebackEntry::build_error_trace(original_input, lineno, column, &nom_error);
        Box::new(ParseError {
            error_info: ErrorInfo::SyntaxError {
                message,
            },
            traceback: Some(traceback),
            source: None,
        })
    }

    /// Attach source information to this error
    /// 
    /// # Arguments
    /// * `input` - The input source containing the error
    /// * `lineno` - The line number where the error occurred
    /// * `text` - The text content of the line where the error occurred
    /// 
    /// # Returns
    /// The error with source information attached
    pub(crate) fn with_source<T: TextInputSource>(mut self: Box<Self>, input: &Input<T>, lineno: usize, text: String) -> Box<Self> {
        self.source = Some(ParserLineSource {
            filename: input.as_ref().source_name().to_string(),
            lineno,
            text,
        });
        self
    }

    /// Get the position (line, column) associated with this error, if any
    /// 
    /// # Returns
    /// An Option containing a tuple of (line, column) if position information is available,
    /// or None if no position information is associated with this error.
    pub fn position(&self) -> Option<(usize, usize)> {
        self.traceback.as_ref().map(|tb| {
            (tb.lineno, tb.column_range.0)
        })
    }

    /// Get the line number associated with this error, if any
    /// 
    /// # Returns
    /// An Option containing the line number if available,
    /// or None if no line information is associated with this error.
    pub fn line(&self) -> Option<usize> {
        self.traceback.as_ref().map(|tb| {
            tb.lineno
        })
    }

    /// Get the error message
    /// 
    /// Extracts the error message from the underlying ErrorInfo enum.
    /// The message format depends on the error type:
    /// - SyntaxError: Returns the original message
    /// - UnexpectedInput: Returns "Unexpected input: '<remaining>'"
    /// - UnexpectedEof: Returns "Unexpected end of input, expected <expected>"
    /// - IoError: Returns the IO error message
    /// 
    /// # Returns
    /// A String containing the formatted error message
    pub fn message(&self) -> String {
        match &self.error_info {
            ErrorInfo::SyntaxError { message, .. } => message.clone(),
            ErrorInfo::UnexpectedInput { remaining, .. } => format!("Unexpected input: '{}'", remaining),
            ErrorInfo::UnexpectedEof { expected, .. } => format!("Unexpected end of input, expected {}", expected),
            ErrorInfo::IoError { error, .. } => error.to_string(),
        }
    }
}

/// Implementation for displaying ParseError in a user-friendly format
/// 
/// This implementation provides a formatted error output that includes:
/// - The error type and message
/// - Source file location (filename, line number, column) if available
/// - The line of code where the error occurred with visual indicators
/// - A traceback tree showing the parsing context
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display error type and message based on error_info
        match &self.error_info {
            ErrorInfo::SyntaxError { message } => {
                write!(f, "SyntaxError: {}", message)?;
            }
            ErrorInfo::UnexpectedInput { remaining, .. } => {
                write!(f, "UnexpectedInputError: '{}'", remaining)?;
            }
            ErrorInfo::UnexpectedEof { expected } => {
                write!(f, "UnexpectedEofError: '{}'", expected)?;
            }
            ErrorInfo::IoError { error } => {
                write!(f, "IOError: {}", error)?;
            }
        }
        
        // Display file location and line information if available
        if let Some(source) = &self.source
            && let Some(traceback) = &self.traceback {
                let (start, end) = traceback.column_range;
                
                // Display source location
                write!(f, "\n  -->   {}:{}:{}", source.filename, source.lineno, start)?;
            
            // Display the code line with visual indicators
            write!(f, "\n    │")?;
                
                // Display line number and content
                write!(f, "\n{: ^4}│    {}", source.lineno, &source.text)?;
                
                // Show arrow pointing to error location
                let arrow = " ".repeat(start + 3) + &"^".repeat(end - start);
                write!(f, "\n    │{}", arrow)?;
            }

        writeln!(f)?;
        // Display the traceback tree
        if let Some(traceback) = &self.traceback {
            if !traceback.context.is_empty() {
                traceback.write_tree(f, "    ", false)?;
            }
        }
        
        Ok(())
    }
}

/// Implementation of the standard Error trait for ParseError
/// 
/// This allows ParseError to be used with the standard error handling mechanisms in Rust.
impl std::error::Error for ParseError {}
