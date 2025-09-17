//! Error types for KoiLang parsing
//! 
//! This module defines the error types used throughout the parsing process.

use std::fmt;
use std::io;

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Error types that can occur during KoiLang parsing
#[derive(Debug)]
pub enum ParseError {
    /// Syntax error in the input
    SyntaxError {
        /// Error message
        message: String,
        /// Line number where the error occurred
        line: usize,
        /// Column number where the error occurred
        column: usize,
    },
    
    /// Unexpected end of input
    UnexpectedEof {
        /// Expected token or structure
        expected: String,
        /// Line number where the error occurred
        line: usize,
    },
    
    /// IO error (for file-based parsing)
    IoError {
        /// The underlying IO error
        error: io::Error,
    },
}

impl ParseError {
    /// Create a new syntax error
    pub fn syntax(message: String, line: usize, column: usize) -> Self {
        ParseError::SyntaxError { message, line, column }
    }

    /// Create a new unexpected EOF error
    pub fn unexpected_eof(expected: String, line: usize) -> Self {
        ParseError::UnexpectedEof { expected, line }
    }

    /// Create a new IO error from an io::Error
    pub fn io(error: io::Error) -> Self {
        ParseError::IoError { error }
    }

    /// Get the line number associated with this error, if any
    pub fn line(&self) -> Option<usize> {
        match self {
            ParseError::SyntaxError { line, .. } => Some(*line),
            ParseError::UnexpectedEof { line, .. } => Some(*line),
            ParseError::IoError { .. } => None,
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::SyntaxError { message, line, column } => {
                write!(f, "Syntax error at line {}, column {}: {}", line, column, message)
            }
            ParseError::UnexpectedEof { expected, line } => {
                write!(f, "Unexpected end of input at line {}, expected {}", line, expected)
            }
            ParseError::IoError { error } => {
                write!(f, "IO error: {}", error)
            }
        }
    }
}

impl std::error::Error for ParseError {}
