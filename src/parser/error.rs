//! Error types for KoiLang parsing
//! 
//! This module defines the error types used throughout the parsing process.

use std::fmt;
use std::io;
use nom_language::error::VerboseError;

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
        /// Context text around the error location
        context: String,
        /// Optional detailed information
        details: Option<String>,
        /// Original nom VerboseError (if from nom parsing error)
        nom_error: Option<VerboseError<String>>,
    },
    
    /// Unexpected end of input
    UnexpectedEof {
        /// Expected token or structure
        expected: String,
        /// Line number where the error occurred
        line: usize,
        /// Source file or input information
        source: Option<String>,
    },
    
    /// IO error (for file-based parsing)
    IoError {
        /// The underlying IO error
        error: io::Error,
        /// Source file or input information
        source: Option<String>,
    },
}

impl ParseError {
    /// Create a new syntax error
    pub fn syntax(message: String, line: usize, column: usize) -> Self {
        ParseError::SyntaxError {
            message,
            line,
            column,
            context: String::new(),
            details: None,
            nom_error: None,
        }
    }

    /// Create a new syntax error with context
    pub fn syntax_with_context(message: String, line: usize, column: usize, context: String) -> Self {
        ParseError::SyntaxError {
            message,
            line,
            column,
            context,
            details: None,
            nom_error: None,
        }
    }

    /// Create a new unexpected EOF error
    pub fn unexpected_eof(expected: String, line: usize) -> Self {
        ParseError::UnexpectedEof {
            expected,
            line,
            source: None,
        }
    }

    /// Create a new IO error from an io::Error
    pub fn io(error: io::Error) -> Self {
        ParseError::IoError {
            error,
            source: None,
        }
    }

    /// Create a syntax error from nom error
    pub fn from_nom_error(
        message: String,
        line: usize,
        column: usize,
        context: String,
        nom_error: VerboseError<String>,
    ) -> Self {
        ParseError::SyntaxError {
            message,
            line,
            column,
            context,
            details: None,
            nom_error: Some(nom_error),
        }
    }


    /// Get the position (line, column) associated with this error, if any
    pub fn position(&self) -> Option<(usize, usize)> {
        match self {
            ParseError::SyntaxError { line, column, .. } => Some((*line, *column)),
            ParseError::UnexpectedEof { line, .. } => Some((*line, 0)),
            ParseError::IoError { .. } => None,
        }
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
            ParseError::SyntaxError { message, line, column, context, details, nom_error } => {
                write!(f, "Syntax error at line {}, column {}: {}", line, column, message)?;
                if !context.is_empty() {
                    write!(f, "\nContext: {}", context)?;
                }
                if let Some(detail_info) = details {
                    write!(f, "\nDetails: {}", detail_info)?;
                }
                if let Some(nom_err) = nom_error {
                    write!(f, "\nParse details: {:?}", nom_err)?;
                }
                Ok(())
            }
            ParseError::UnexpectedEof { expected, line, source } => {
                write!(f, "Unexpected end of input at line {}, expected {}", line, expected)?;
                if let Some(src) = source {
                    write!(f, " in {}", src)?;
                }
                Ok(())
            }
            ParseError::IoError { error, source } => {
                write!(f, "IO error: {}", error)?;
                if let Some(src) = source {
                    write!(f, " in {}", src)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ParseError {}
