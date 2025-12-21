//! Error traceback handling for the parser.
//!
//! This module defines the error traceback type.

use std::fmt;

use nom::Input;
use nom::Offset;
use nom::error::ContextError;
use nom::error::ErrorKind;
use nom::error::FromExternalError;
use nom::error::ParseError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Error context for `VerboseError`
enum NomErrorKind {
    /// Static string added by the `context` function
    Context(&'static str),
    /// Indicates which character was expected by the `char` function
    Char(char),
    /// Error kind given by various nom parsers
    Nom(ErrorKind),
}

#[derive(Clone, Debug, Eq, PartialEq)]
/// Node in the nom error tree
pub(super) struct NomErrorNode<I> {
    /// Input position where the error occurred
    input: I,
    /// Nom error kind
    kind: NomErrorKind,
    /// Child error nodes
    children: Vec<NomErrorNode<I>>,
}

impl<I: Input> ParseError<I> for NomErrorNode<I> {
    fn from_error_kind(input: I, kind: ErrorKind) -> Self {
        NomErrorNode {
            input,
            kind: NomErrorKind::Nom(kind),
            children: vec![],
        }
    }

    fn append(input: I, kind: ErrorKind, mut other: Self) -> Self {
        match (kind, other.kind) {
            (ErrorKind::Alt, NomErrorKind::Nom(ErrorKind::Alt)) => {
                // Both are Alt, just return the existing node
                other.input = input;
                other
            }
            _ => {
                other
                    .children
                    .push(NomErrorNode::from_error_kind(input, kind));
                other
            }
        }
    }

    fn from_char(input: I, c: char) -> Self {
        NomErrorNode {
            input,
            kind: NomErrorKind::Char(c),
            children: vec![],
        }
    }

    fn or(mut self, mut other: Self) -> Self {
        match (self.kind, other.kind) {
            (NomErrorKind::Nom(ErrorKind::Alt), NomErrorKind::Nom(ErrorKind::Alt)) => {
                self.children.extend(other.children);
                self
            }
            (NomErrorKind::Nom(ErrorKind::Alt), _) => {
                self.children.push(other);
                self
            }
            (_, NomErrorKind::Nom(ErrorKind::Alt)) => {
                other.children.push(self);
                other
            }
            _ => {
                // Neither is an Alt, create a new Alt node
                NomErrorNode {
                    input: self.input.clone(), // or other.input.clone(), they should be the same
                    kind: NomErrorKind::Nom(ErrorKind::Alt),
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
            kind: NomErrorKind::Context(ctx),
            children: vec![other],
        }
    }
}

impl<I: Input, E> FromExternalError<I, E> for NomErrorNode<I> {
    /// Create a new error from an input position and an external error
    fn from_external_error(input: I, kind: ErrorKind, _e: E) -> Self {
        Self::from_error_kind(input, kind)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TracebackEntry {
    /// Line number where this traceback point occurred
    pub lineno: usize,
    /// Column number where this traceback point occurred
    pub column_range: (usize, usize),
    /// Context description (from nom error kind or parsing context)
    pub context: String,
    /// Traceback children (for nested contexts)
    pub children: Vec<TracebackEntry>,
}

impl TracebackEntry {
    pub fn new(lineno: usize, column_range: (usize, usize), context: String) -> Self {
        Self {
            lineno,
            column_range,
            context,
            children: vec![],
        }
    }

    pub(super) fn build_error_trace<I: core::ops::Deref<Target = str> + Input>(
        input: I,
        line: usize,
        column_offset: usize,
        error: &NomErrorNode<I>,
    ) -> Self {
        let line_index = LineIndex::new(&input);
        Self::build_error_trace_recursive(&input, &line_index, line, column_offset, error)
    }

    fn build_error_trace_recursive<I: core::ops::Deref<Target = str> + Input>(
        input: &I,
        line_index: &LineIndex,
        line_base: usize,
        column_offset: usize,
        error: &NomErrorNode<I>,
    ) -> Self {
        let (rel_line, rel_column) = line_index.get_location(input, &error.input);
        let context = Self::format_error_kind(&error.kind);

        let children = error
            .children
            .iter()
            .map(|child| {
                Self::build_error_trace_recursive(
                    input,
                    line_index,
                    line_base,
                    column_offset,
                    child,
                )
            })
            .collect();

        TracebackEntry {
            lineno: line_base + rel_line - 1,
            column_range: (
                rel_column + column_offset,
                rel_column + error.input.len() + column_offset,
            ),
            context,
            children,
        }
    }

    /// Format nom error kind into human-readable context
    #[inline]
    fn format_error_kind(error_kind: &NomErrorKind) -> String {
        match error_kind {
            NomErrorKind::Context(ctx) => format!("koicore.{}", ctx),
            NomErrorKind::Char(c) => format!("nom.char<'{}'>", c),
            NomErrorKind::Nom(kind) => format!("nom.{:?}", kind),
        }
    }

    pub(super) fn write_tree(
        &self,
        f: &mut fmt::Formatter<'_>,
        prefix: &str,
        is_last: bool,
    ) -> fmt::Result {
        let connector = if is_last { "└─ " } else { "├─ " };
        let (start, end) = self.column_range;
        writeln!(
            f,
            "{}{}{} ({}–{})",
            prefix, connector, self.context, start, end
        )?;

        let child_prefix = if is_last { "   " } else { "│  " };
        let total_children = self.children.len();
        for (i, child) in self.children.iter().enumerate() {
            let is_last_child = i == total_children - 1;
            let new_prefix = format!("{}{}", prefix, child_prefix);
            child.write_tree(f, &new_prefix, is_last_child)?;
        }

        Ok(())
    }
}

impl fmt::Display for TracebackEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_tree(f, "", false)
    }
}

/// Helper structure for efficient line number lookups
struct LineIndex {
    /// Start offsets of newlines
    newlines: Vec<usize>,
}

impl LineIndex {
    fn new(input: &str) -> Self {
        let newlines = input
            .bytes()
            .enumerate()
            .filter_map(|(i, b)| if b == b'\n' { Some(i) } else { None })
            .collect();
        Self { newlines }
    }

    /// Get valid (line_number, column_number) for a substring
    /// line_number is 1-based, column_number is 1-based
    fn get_location<I: core::ops::Deref<Target = str>>(
        &self,
        full_input: &I,
        substring: &I,
    ) -> (usize, usize) {
        let offset = full_input.offset(substring);

        if full_input.is_empty() {
            return (1, 1);
        }

        let line_number = match self.newlines.binary_search(&offset) {
            Ok(idx) => idx + 1,
            Err(idx) => idx + 1,
        };

        // Determine column
        let line_start = if line_number == 1 {
            0
        } else {
            self.newlines[line_number - 2] + 1
        };

        let column_number = offset - line_start + 1;

        (line_number, column_number)
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
        let entry = TracebackEntry::build_error_trace(input, 1, 0, &error);
        assert_eq!(entry.lineno, 1);
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
        match node {
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                let traceback = TracebackEntry::build_error_trace(input, 1, 0, &e);
                println!("Converted traceback: {:#?}", traceback);
            }
            _ => unreachable!(),
        }

        let input = "error e() 1";
        let result = command_parser::parse_command_line::<'_, NomErrorNode<&str>>(input);
        assert!(result.is_err());
        let node = result.unwrap_err();
        println!("Parser error traceback: {:#?}", node);
        match node {
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                let traceback = TracebackEntry::build_error_trace(input, 1, 0, &e);
                println!("Converted traceback: {:#?}", traceback);
            }
            _ => unreachable!(),
        }

        let input = "error e(1, 2 3)";
        let result = command_parser::parse_command_line::<'_, NomErrorNode<&str>>(input);
        assert!(result.is_err());
        let node = result.unwrap_err();
        println!("Parser error traceback: {:#?}", node);
        match node {
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                let traceback = TracebackEntry::build_error_trace(input, 1, 0, &e);
                println!("Converted traceback: {:#?}", traceback);
            }
            _ => unreachable!(),
        }
    }
}
