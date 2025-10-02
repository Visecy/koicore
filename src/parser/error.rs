//! Error types for KoiLang parsing
//! 
//! This module defines the error types used throughout the parsing process.

use std::fmt;
use std::io;
use nom_language::error::VerboseError;
use nom::Offset;

/// Result type for parsing operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Individual traceback entry containing parsing context information
#[derive(Debug, Clone)]
pub struct TracebackEntry {
    /// Line number where this traceback point occurred
    pub line: usize,
    /// Column number where this traceback point occurred
    pub column: usize,
    /// Context description (from nom error kind or parsing context)
    pub context: String,
    /// Input text at this position
    pub input: String,
}

/// Traceback information - a collection of traceback entries
pub type Traceback = Vec<TracebackEntry>;

/// Semantic error information without positional data
#[derive(Debug)]
pub enum ErrorInfo {
    /// Syntax error in the input
    SyntaxError {
        /// Error message
        message: String,
        /// Optional detailed information
        details: Option<String>,
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

/// Combined error type containing both semantic error information and traceback
#[derive(Debug)]
pub struct ParseError {
    /// The semantic error information
    pub error_info: ErrorInfo,
    /// Optional traceback information
    pub traceback: Option<Traceback>,
}

impl ParseError {
    /// Create a new syntax error
    pub fn syntax(message: String, line: usize, column: usize) -> Self {
        ParseError {
            error_info: ErrorInfo::SyntaxError {
                message,
                details: None,
            },
            traceback: Some(vec![TracebackEntry {
                line,
                column,
                context: String::new(),
                input: String::new(),
            }]),
        }
    }

    /// Create a new syntax error with context
    pub fn syntax_with_context(message: String, line: usize, column: usize, context: String) -> Self {
        ParseError {
            error_info: ErrorInfo::SyntaxError {
                message,
                details: None,
            },
            traceback: Some(vec![TracebackEntry {
                line,
                column,
                context,
                input: String::new(),
            }]),
        }
    }
    /// Create a new unexpected input error
    pub fn unexpected_input(remaining: String, line: usize, input: String) -> Self {
        let column = input.len() - remaining.len() + 1;
        ParseError {
            error_info: ErrorInfo::UnexpectedInput {
                remaining,
            },
            traceback: Some(vec![TracebackEntry {
                line,
                column,
                context: "<string>".to_string(),
                input,
            }]),
        }
    }

    /// Create a new unexpected EOF error
    pub fn unexpected_eof(expected: String, line: usize) -> Self {
        ParseError {
            error_info: ErrorInfo::UnexpectedEof {
                expected,
                source: None,
            },
            traceback: Some(vec![TracebackEntry {
                line,
                column: 0,
                context: "<string>".to_string(),
                input: String::new(),
            }]),
        }
    }

    /// Create a new IO error from an io::Error
    pub fn io(error: io::Error) -> Self {
        ParseError {
            error_info: ErrorInfo::IoError {
                error,
                source: None,
            },
            traceback: None,
        }
    }

    /// Create a syntax error from nom error
    pub fn from_nom_error<I: core::ops::Deref<Target = str>>(
        message: String,
        original_input: I,
        nom_error: VerboseError<I>,
    ) -> Self {
        let traceback = Self::parse_nom_traceback(original_input, &nom_error);
        ParseError {
            error_info: ErrorInfo::SyntaxError {
                message,
                details: None,
            },
            traceback: Some(traceback),
        }
    }

    /// Parse nom VerboseError into Traceback entries
    fn parse_nom_traceback<I: core::ops::Deref<Target = str>>(
        original_input: I,
        nom_error: &VerboseError<I>,
    ) -> Traceback {
        let mut traceback = Vec::new();
        
        for (substring, error_kind) in &nom_error.errors {
            let (line, column, line_content) = Self::extract_position_info(&original_input, substring);
            let context = Self::format_error_kind(error_kind);
            
            traceback.push(TracebackEntry {
                line,
                column,
                context,
                input: line_content,
            });
        }
        
        traceback
    }

    /// Extract line, column, and line content from input substrings
    fn extract_position_info<I: core::ops::Deref<Target = str>>(
        input: &I,
        substring: &I,
    ) -> (usize, usize, String) {
        let offset = input.offset(substring);

        if input.is_empty() {
            return (1, 1, String::new());
        }

        let prefix = &input.as_bytes()[..offset];

        // Count the number of newlines in the first `offset` bytes of input
        let line_number = prefix.iter().filter(|&&b| b == b'\n').count() + 1;

        // Find the line that includes the subslice:
        // Find the *last* newline before the substring starts
        let line_begin = prefix
            .iter()
            .rev()
            .position(|&b| b == b'\n')
            .map(|pos| offset - pos)
            .unwrap_or(0);

        // Find the full line after that newline
        let line = input[line_begin..]
            .lines()
            .next()
            .unwrap_or(&input[line_begin..])
            .trim_end();

        // The (1-indexed) column number is the offset of our substring into that line
        let column_number = line.offset(substring) + 1;

        (line_number, column_number, line.to_string())
    }

    /// Format nom error kind into human-readable context
    fn format_error_kind(error_kind: &nom_language::error::VerboseErrorKind) -> String {
        use nom_language::error::VerboseErrorKind;
        match error_kind {
            VerboseErrorKind::Context(ctx) => format!("koicore.{}", ctx),
            VerboseErrorKind::Char(c) => format!("nom.char<'{}'>", c),
            VerboseErrorKind::Nom(kind) => format!("nom.{:?}", kind),
        }
    }

    /// Get the position (line, column) associated with this error, if any
    pub fn position(&self) -> Option<(usize, usize)> {
        self.traceback.as_ref().and_then(|tb| {
            tb.first().map(|entry| (entry.line, entry.column))
        })
    }

    /// Get the line number associated with this error, if any
    pub fn line(&self) -> Option<usize> {
        self.traceback.as_ref().and_then(|tb| {
            tb.first().map(|entry| entry.line)
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
        match &self.error_info {
            ErrorInfo::SyntaxError { message, details } => {
                write!(f, "Syntax error: {}", message)?;
                if let Some(detail_info) = details {
                    write!(f, "\nDetails: {}", detail_info)?;
                }
            }
            ErrorInfo::UnexpectedInput { remaining, .. } => {
                write!(f, "Unexpected input: '{}'", remaining)?;
            }
            ErrorInfo::UnexpectedEof { expected, source } => {
                write!(f, "Unexpected end of input, expected {}", expected)?;
                if let Some(src) = source {
                    write!(f, " in {}", src)?;
                }
            }
            ErrorInfo::IoError { error, source } => {
                write!(f, "IO error: {}", error)?;
                if let Some(src) = source {
                    write!(f, " in {}", src)?;
                }
            }
        }
        
        // Display traceback information
        if let Some(traceback) = &self.traceback {
            for entry in traceback {
                write!(f, "\nAt line {}, column {}: {}", entry.line, entry.column, entry.context)?;
                if !entry.input.is_empty() {
                    write!(f, "\nInput: {}", entry.input)?;
                }
            }
        }
        
        Ok(())
    }
}

impl std::error::Error for ParseError {}
