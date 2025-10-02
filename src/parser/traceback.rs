//! Error traceback handling for the parser.
//!
//! This module defines the error traceback type.

use std::fmt;
use std::io;
use nom::error::ContextError;
use nom::error::ErrorKind;
use nom::error::FromExternalError;
use nom::error::ParseError;
use nom::Offset;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Error context for `VerboseError`
pub(crate) enum VerboseErrorKind {
    /// Static string added by the `context` function
    Context(&'static str),
    /// Indicates which character was expected by the `char` function
    Char(char),
    /// Error kind given by various nom parsers
    Nom(ErrorKind),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Node in the nom error tree
pub(crate) struct NomErrorNode<I> {
    /// Input position where the error occurred
    pub input: I,
    /// Nom error kind
    pub kind: VerboseErrorKind,
    /// Child error nodes
    pub children: Vec<NomErrorNode<I>>,
}

impl<I> ParseError<I> for NomErrorNode<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        NomErrorNode {
            input,
            kind: VerboseErrorKind::Nom(kind),
            children: vec![],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        other.children.push(NomErrorNode::from_error_kind(input, kind));
        other
    }

    fn from_char(input: I, c: char) -> Self {
        NomErrorNode {
            input,
            kind: VerboseErrorKind::Char(c),
            children: vec![],
        }
    }

    fn or(mut self, mut other: Self) -> Self {
        match (self.kind, other.kind)  {
            (VerboseErrorKind::Context(_), _) => {
                self.children.push(other);
                self
            }
            (_, VerboseErrorKind::Context(_)) => {
                other.children.push(self);
                other
            }
            (VerboseErrorKind::Nom(ErrorKind::Alt), VerboseErrorKind::Nom(ErrorKind::Alt)) => {
                self.children.extend(other.children);
                self
            }
            (VerboseErrorKind::Nom(ErrorKind::Alt), _) => {
                self.children.push(other);
                self
            }
            (_, VerboseErrorKind::Nom(ErrorKind::Alt)) => {
                other.children.push(self);
                other
            }
            _ => {
                // Neither is an Alt, create a new Alt node
                NomErrorNode {
                    input: self.input.clone(), // or other.input.clone(), they should be the same
                    kind: VerboseErrorKind::Nom(ErrorKind::Alt),
                    children: vec![self, other],
                }
            }
        }
    }
}

impl<I> ContextError<I> for NomErrorNode<I> {
    fn add_context(input: I, ctx: &'static str, other: Self) -> Self {
        NomErrorNode {
            input,
            kind: VerboseErrorKind::Context(ctx),
            children: vec![other],
        }
    }
}

impl<I, E> FromExternalError<I, E> for NomErrorNode<I> {
    /// Create a new error from an input position and an external error
    fn from_external_error(input: I, kind: ErrorKind, _e: E) -> Self {
        Self::from_error_kind(input, kind)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TracebackEntry {
    /// Line number where this traceback point occurred
    pub line: usize,
    /// Column number where this traceback point occurred
    pub column_range: (usize, usize),
    /// Context description (from nom error kind or parsing context)
    pub context: String,
    /// Traceback children (for nested contexts)
    pub children: Vec<TracebackEntry>,
}

impl TracebackEntry {
    pub(crate) fn convert_error<I: core::ops::Deref<Target = str> + Clone>(input: I, line: usize, error: &NomErrorNode<I>) -> Self {
        let (line_offset, column, _) = Self::extract_position_info(&input, &error.input);
        let context = Self::format_error_kind(&error.kind);

        let children = error.children.iter().map(|child| Self::convert_error(input.clone(), line, child)).collect();

        TracebackEntry {
            line : line + line_offset - 1,
            column_range: (column, column + error.input.len()),
            context,
            children,
        }
    }
    
    /// Extract line, column, and line content from input substrings
    #[inline]
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
    #[inline]
    fn format_error_kind(error_kind: &VerboseErrorKind) -> String {
        match error_kind {
            VerboseErrorKind::Context(ctx) => format!("koicore.{}", ctx),
            VerboseErrorKind::Char(c) => format!("nom.char<'{}'>", c),
            VerboseErrorKind::Nom(kind) => format!("nom.{:?}", kind),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Error traceback for parse errors
pub struct Traceback {
    /// List of traceback entries
    pub entry: TracebackEntry,
    pub input: String,
    pub input_line: usize,
}

impl Traceback {
    pub fn new(entry: TracebackEntry, input: String, input_line: usize) -> Self {
        Traceback {
            entry,
            input,
            input_line,
        }
    }

    /// Create a traceback from a nom verbose error
    pub(crate) fn from_nom_error<I: core::ops::Deref<Target = str> + Clone>(
        input: I,
        line_number: usize,
        nom_error: &NomErrorNode<I>,
    ) -> Self {
        let entry = TracebackEntry::convert_error(input.clone(), line_number, nom_error);
        Self::new(entry, input.to_string(), line_number)
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::command_parser;

    use super::*;

    #[test]
    fn test_traceback_entry_convert_error() {
        let input = "line1\nline2\nline3";
        let error = NomErrorNode::from_char(input, 'a');
        let entry = TracebackEntry::convert_error(input, 1, &error);
        assert_eq!(entry.line, 1);
        assert_eq!(entry.column_range, (1, input.len() + 1));
        assert_eq!(entry.context, "nom.char<'a'>");
        assert!(entry.children.is_empty());
    }

    #[test]
    fn test_parser_error_traceback() {
        let input = "error e(";
        let result = command_parser::parse_command_line::<'_, NomErrorNode<&str>>(input);
        assert!(result.is_err());
        let node = result.unwrap_err();
        println!("Parser error traceback: {:#?}", node);
    }
}
