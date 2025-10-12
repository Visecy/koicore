//! Error types for KoiLang parsing
//! 
//! This module defines the error types used throughout the parsing process.

use std::fmt;
use std::io;

use crate::parser::input::Input;
use crate::parser::traceback::TracebackEntry;
use crate::parser::NomErrorNode;
use crate::parser::TextInputSource;

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Semantic error information without positional data
#[derive(Debug)]
pub enum ErrorInfo {
    /// Syntax error in the input
    SyntaxError {
        /// Error message
        message: String,
    },
    
    // Unexpected input
    UnexpectedInput {
        /// Remaining unparsed input
        remaining: String,
    },
    
    /// Unexpected end of input
    UnexpectedEof {
        /// Expected token or structure
        expected: String,
    },
    
    /// IO error (for file-based parsing)
    IoError {
        /// The underlying IO error
        error: io::Error,
    },
}

/// Information about the source of a parsed line
#[derive(Debug, Clone)]
pub struct ParserLineSource {
    /// Source file path
    pub filename: String,
    /// Line number in the source file
    pub lineno: usize,
    /// The input line content
    pub text: String,
}

/// Combined error type containing both semantic error information and traceback
#[derive(Debug)]
pub struct ParseError {
    /// The semantic error information
    pub error_info: ErrorInfo,
    /// Optional traceback information
    pub traceback: Option<TracebackEntry>,
    pub source: Option<ParserLineSource>,
}

impl ParseError {
    /// Create a new syntax error
    pub fn syntax(message: String) -> Self {
        ParseError {
            error_info: ErrorInfo::SyntaxError {
                message,
            },
            traceback: None,
            source: None,
        }
    }

    /// Create a new syntax error with context
    pub fn syntax_with_context(message: String, line: usize, column: usize, context: String) -> Self {
        ParseError {
            error_info: ErrorInfo::SyntaxError {
                message,
            },
            traceback: Some(TracebackEntry::new(line, (column, column + 1), context)),
        source: None,
        }
    }
    /// Create a new unexpected input error
    pub fn unexpected_input(remaining: String, line: usize, input: String) -> Self {
        let column = input.len() - remaining.len() + 1;
        ParseError {
            traceback: Some(
                TracebackEntry::new(line, (column, column + remaining.len()),
            "".to_string())
            ),
            error_info: ErrorInfo::UnexpectedInput {
                remaining,
            },
            source: None,
        }
    }

    /// Create a new unexpected EOF error
    pub fn unexpected_eof(expected: String, line: usize) -> Self {
        ParseError {
            error_info: ErrorInfo::UnexpectedEof {
                expected,
            },
            traceback: Some(TracebackEntry::new(line, (0, 0), "".to_string())),
            source: None,
        }
    }

    /// Create a new IO error from an io::Error
    pub fn io(error: io::Error) -> Self {
        ParseError {
            error_info: ErrorInfo::IoError {
                error,
            },
            traceback: None,
            source: None,
        }
    }

    /// Create a syntax error from nom error
    pub(super) fn from_nom_error<I: core::ops::Deref<Target = str> + nom::Input>(
        message: String,
        original_input: I,
        lineno: usize,
        nom_error: NomErrorNode<I>,
    ) -> Self {
        let traceback = TracebackEntry::build_error_trace(original_input, lineno, &nom_error);
        ParseError {
            error_info: ErrorInfo::SyntaxError {
                message,
            },
            traceback: Some(traceback),
            source: None,
        }
    }

    pub(crate) fn with_source<T: TextInputSource>(mut self, input: &Input<T>, lineno: usize, text: String) -> Self {
        self.source = Some(ParserLineSource {
            filename: input.as_ref().source_name().to_string(),
            lineno,
            text,
        });
        self
    }

    /// Get the position (line, column) associated with this error, if any
    pub fn position(&self) -> Option<(usize, usize)> {
        self.traceback.as_ref().map(|tb| {
            (tb.lineno, tb.column_range.0)
        })
    }

    /// Get the line number associated with this error, if any
    pub fn line(&self) -> Option<usize> {
        self.traceback.as_ref().map(|tb| {
            tb.lineno
        })
    }

    /// Get the error message
    pub fn message(&self) -> String {
        match &self.error_info {
            ErrorInfo::SyntaxError { message, .. } => message.clone(),
            ErrorInfo::UnexpectedInput { remaining, .. } => format!("Unexpected input: '{}'", remaining),
            ErrorInfo::UnexpectedEof { expected, .. } => format!("Unexpected end of input, expected {}", expected),
            ErrorInfo::IoError { error, .. } => error.to_string(),
        }
    }
}

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
        if let Some(source) = &self.source {
            if let Some(traceback) = &self.traceback {
                let (start, end) = traceback.column_range;
                
                // Display source location
                write!(f, "\n   --> {}:{}:{}", source.filename, source.lineno, start)?;
            
            // Display the code line with visual indicators
            write!(f, "\n    │")?;
                
                // Display line number and content
                write!(f, "\n{: ^4}│   {}", source.lineno, &source.text)?;
                
                // Show arrow pointing to error location
                let arrow = " ".repeat(start + 3) + &"^".repeat(end - start);
                write!(f, "\n    │{}", arrow)?;
            }
        }

        write!(f, "\n")?;
        // Display the traceback tree
        if let Some(traceback) = &self.traceback {
            traceback.write_tree(f, "    ", false)?;
        }
        
        Ok(())
    }
}

impl std::error::Error for ParseError {}
