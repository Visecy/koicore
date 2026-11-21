//! Format-preserving patch application for existing KoiLang documents.
//!
//! A [`Patch`] describes whole-line edits against an existing source text. Applying it
//! writes the edited document while copying every untouched line verbatim, so blank
//! lines, indentation, line endings and existing formatting survive.

use std::fmt;
use std::io;

use crate::command::Command;
use crate::parser::ParserConfig;

/// How a patch edit locates its target.
#[derive(Debug, Clone, PartialEq)]
pub enum Anchor {
    /// 1-based physical line number. Must be the first line of a logical line
    /// (a line continued with a trailing backslash cannot be targeted directly).
    ///
    /// `expect` guards against the source having changed since it was inspected.
    Line {
        /// 1-based physical line number.
        lineno: usize,
        /// Optional integrity check.
        expect: Option<Expected>,
    },
    /// 0-based index into the parsed command stream, in the order produced by
    /// [`Parser::next_command_with_source`](crate::parser::Parser::next_command_with_source).
    ///
    /// Requires the source to be parseable when the patch is applied.
    Command {
        /// 0-based command index.
        index: usize,
        /// Optional integrity check against the parsed command.
        expect: Option<Command>,
    },
}

/// Optional integrity check for a patch anchor.
#[derive(Debug, Clone, PartialEq)]
pub enum Expected {
    /// Raw text of the targeted physical line, without its terminator.
    /// A single trailing `\r\n` / `\n` in the expectation is normalized away.
    Text(String),
    /// The targeted logical line must parse to exactly this command.
    Command(Command),
}

/// Whole-line edit operations.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    /// Replace the targeted logical line with these commands (must be non-empty).
    Replace(Vec<Command>),
    /// Insert these commands before the targeted logical line.
    InsertBefore(Vec<Command>),
    /// Insert these commands after the targeted logical line.
    InsertAfter(Vec<Command>),
    /// Delete the targeted logical line.
    Delete,
}

/// One edit in a [`Patch`].
#[derive(Debug, Clone, PartialEq)]
pub struct PatchEdit {
    /// Target anchor.
    pub anchor: Anchor,
    /// Operation to apply.
    pub edit: Edit,
}

/// A format-preserving patch for a KoiLang source document.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Patch {
    edits: Vec<PatchEdit>,
    append: Vec<Command>,
    options: PatchOptions,
}

impl Patch {
    /// Create an empty patch with default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the patch has no edits and no appended commands.
    pub fn is_empty(&self) -> bool {
        self.edits.is_empty() && self.append.is_empty()
    }

    /// Borrow the edit list.
    pub fn edits(&self) -> &[PatchEdit] {
        &self.edits
    }

    /// Borrow the patch options.
    pub fn options(&self) -> &PatchOptions {
        &self.options
    }

    /// Mutably borrow the patch options.
    pub fn options_mut(&mut self) -> &mut PatchOptions {
        &mut self.options
    }

    /// Replace the patch options.
    pub fn with_options(mut self, options: PatchOptions) -> Self {
        self.options = options;
        self
    }

    /// Add a generic edit.
    pub fn edit(&mut self, anchor: Anchor, edit: Edit) -> &mut Self {
        self.edits.push(PatchEdit { anchor, edit });
        self
    }

    /// Append commands after the last logical line of the source.
    /// Repeated calls accumulate in call order.
    pub fn append(&mut self, commands: Vec<Command>) -> &mut Self {
        self.append.extend(commands);
        self
    }

    /// Replace a logical line addressed by 1-based physical line number.
    pub fn replace_line(&mut self, lineno: usize, commands: Vec<Command>) -> &mut Self {
        self.edit(Anchor::Line { lineno, expect: None }, Edit::Replace(commands))
    }

    /// Replace a logical line and verify its raw text first.
    pub fn replace_line_checked(
        &mut self,
        lineno: usize,
        expect: Expected,
        commands: Vec<Command>,
    ) -> &mut Self {
        self.edit(Anchor::Line { lineno, expect: Some(expect) }, Edit::Replace(commands))
    }

    /// Insert a command block before a logical line.
    pub fn insert_before_line(&mut self, lineno: usize, commands: Vec<Command>) -> &mut Self {
        self.edit(Anchor::Line { lineno, expect: None }, Edit::InsertBefore(commands))
    }

    /// Insert a command block after a logical line.
    pub fn insert_after_line(&mut self, lineno: usize, commands: Vec<Command>) -> &mut Self {
        self.edit(Anchor::Line { lineno, expect: None }, Edit::InsertAfter(commands))
    }

    /// Delete a logical line.
    pub fn delete_line(&mut self, lineno: usize) -> &mut Self {
        self.edit(Anchor::Line { lineno, expect: None }, Edit::Delete)
    }

    /// Replace the logical line that produced command `index`.
    pub fn replace_command(&mut self, index: usize, commands: Vec<Command>) -> &mut Self {
        self.edit(Anchor::Command { index, expect: None }, Edit::Replace(commands))
    }

    /// Insert a command block before the logical line that produced command `index`.
    pub fn insert_before_command(&mut self, index: usize, commands: Vec<Command>) -> &mut Self {
        self.edit(Anchor::Command { index, expect: None }, Edit::InsertBefore(commands))
    }

    /// Insert a command block after the logical line that produced command `index`.
    pub fn insert_after_command(&mut self, index: usize, commands: Vec<Command>) -> &mut Self {
        self.edit(Anchor::Command { index, expect: None }, Edit::InsertAfter(commands))
    }

    /// Delete the logical line that produced command `index`.
    pub fn delete_command(&mut self, index: usize) -> &mut Self {
        self.edit(Anchor::Command { index, expect: None }, Edit::Delete)
    }
}

/// Conflict policy and format-preservation knobs.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PatchOptions {
    /// Conflict policy. `Strict` (default) validates everything before writing;
    /// `Lenient` skips invalid or conflicting edits and records warnings.
    pub mode: ApplyMode,
    /// Indentation policy for newly written lines.
    pub indent: IndentPolicy,
    /// Line-ending style for newly written lines.
    pub newline: NewlineStyle,
    /// Parser configuration used to resolve [`Anchor::Command`] and to parse
    /// [`Expected::Command`]. Must match the configuration the source was read with.
    pub parser_config: ParserConfig,
}

/// Conflict policy for patch application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ApplyMode {
    /// Validate every target/expectation/conflict before writing any byte.
    #[default]
    Strict,
    /// Apply what is valid and record skipped edits as warnings.
    Lenient,
}

/// Indentation policy for newly written lines.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum IndentPolicy {
    /// Copy the leading whitespace of the target logical line (default).
    /// For `append`, inherit the last logical line's leading whitespace.
    #[default]
    InheritAnchor,
    /// Emit new lines with no leading whitespace.
    None,
    /// Emit new lines with exactly this prefix.
    Fixed(String),
}

/// Line-ending style for newly written lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NewlineStyle {
    /// Detect from the source's first terminator; default `\n`.
    #[default]
    Auto,
    /// Always write `\n`.
    Lf,
    /// Always write `\r\n`.
    Crlf,
}

/// Result metadata for a successfully applied patch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchReport {
    /// Physical line count of the source (continuation lines included).
    pub lines_before: usize,
    /// Physical line count of the emitted document.
    pub lines_after: usize,
    /// Number of `Replace` edits applied.
    pub replaced: usize,
    /// Number of `InsertBefore` / `InsertAfter` edits applied.
    pub inserted: usize,
    /// Number of `Delete` edits applied.
    pub deleted: usize,
    /// Number of commands appended to end of file.
    pub appended: usize,
    /// Edits skipped in [`ApplyMode::Lenient`].
    pub warnings: Vec<PatchWarning>,
}

/// A lenient-mode edit that was skipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchWarning {
    /// Target line/command index was out of range; edit skipped.
    OutOfRange {
        /// Index of the skipped edit in [`Patch::edits`].
        edit_index: usize,
    },
    /// Anchor expectation did not match; edit skipped.
    AnchorMismatch {
        /// Index of the skipped edit in [`Patch::edits`].
        edit_index: usize,
    },
    /// Another edit already owns this logical line; edit skipped.
    Conflict {
        /// Index of the skipped edit in [`Patch::edits`].
        edit_index: usize,
        /// Index of the edit that owns the target.
        owner_edit_index: usize,
    },
    /// Edit payload was empty; edit skipped.
    EmptyContent {
        /// Index of the skipped edit in [`Patch::edits`].
        edit_index: usize,
    },
    /// Line anchor points into a continuation line; edit skipped.
    InsideContinuation {
        /// Index of the skipped edit in [`Patch::edits`].
        edit_index: usize,
    },
}

/// Errors returned by patch application.
#[derive(Debug)]
pub enum PatchError {
    /// Writing the patched document failed.
    Io(io::Error),
    /// A line anchor is outside the source.
    LineOutOfRange {
        /// Requested 1-based line number.
        lineno: usize,
        /// Physical line count of the source.
        total_lines: usize,
    },
    /// A command anchor is outside the parsed command stream.
    CommandIndexOutOfRange {
        /// Requested command index.
        index: usize,
        /// Number of parsed commands.
        total_commands: usize,
    },
    /// A line anchor points into the middle of a continued logical line.
    TargetInsideContinuation {
        /// Requested 1-based line number.
        lineno: usize,
        /// First physical line of the containing logical line.
        start_lineno: usize,
    },
    /// Parsing the source (or an `Expected::Command` line) failed.
    SourceParse(Box<crate::parser::ParseError>),
    /// A line anchor's expectation did not match.
    AnchorMismatch {
        /// 1-based line number.
        lineno: usize,
        /// The expectation that failed.
        expected: Expected,
        /// Actual raw line text.
        actual: String,
    },
    /// A command anchor's expected command did not match.
    CommandMismatch {
        /// Requested command index.
        index: usize,
        /// Physical line number of the parsed command.
        lineno: usize,
        /// Expected command.
        expected: Command,
        /// Parsed command.
        actual: Command,
    },
    /// Two edits both replace/delete the same logical line.
    ConflictingEdits {
        /// First physical line of the logical line.
        lineno: usize,
        /// Index of the first owning edit.
        first_edit: usize,
        /// Index of the conflicting edit.
        second_edit: usize,
    },
    /// A replace/insert edit carried no commands.
    EmptyContent {
        /// Index of the offending edit in [`Patch::edits`].
        edit_index: usize,
        /// Operation name (`"Replace"`, `"InsertBefore"` or `"InsertAfter"`).
        operation: &'static str,
    },
}

impl fmt::Display for PatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatchError::Io(error) => {
                write!(f, "failed to write patched document: {error}")
            }
            PatchError::LineOutOfRange { lineno, total_lines } => {
                write!(f, "line {lineno} is out of range (source has {total_lines} lines)")
            }
            PatchError::CommandIndexOutOfRange { index, total_commands } => {
                write!(
                    f,
                    "command index {index} is out of range (source has {total_commands} commands)"
                )
            }
            PatchError::TargetInsideContinuation { lineno, start_lineno } => {
                write!(
                    f,
                    "line {lineno} is inside the logical line starting at line {start_lineno}; target the first line instead"
                )
            }
            PatchError::SourceParse(error) => {
                write!(f, "failed to parse source while resolving patch anchors: {error}")
            }
            PatchError::AnchorMismatch { lineno, expected, actual } => {
                write!(
                    f,
                    "line {lineno} does not match the expected anchor ({expected:?}, actual {actual:?})"
                )
            }
            PatchError::CommandMismatch { index, lineno, expected, actual } => {
                write!(
                    f,
                    "command #{index} at line {lineno} does not match the expected command ({expected:?}, actual {actual:?})"
                )
            }
            PatchError::ConflictingEdits { lineno, first_edit, second_edit } => {
                write!(
                    f,
                    "edits {first_edit} and {second_edit} both replace or delete logical line {lineno}"
                )
            }
            PatchError::EmptyContent { edit_index, operation } => {
                write!(
                    f,
                    "{operation} edit {edit_index} has no commands; use Edit::Delete to delete a line"
                )
            }
        }
    }
}

impl std::error::Error for PatchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PatchError::Io(error) => Some(error),
            PatchError::SourceParse(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<io::Error> for PatchError {
    fn from(error: io::Error) -> Self {
        PatchError::Io(error)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;

    fn cmd(name: &str) -> Command {
        Command::new(name, Vec::new())
    }

    #[test]
    fn patch_builder_accumulates_edits_and_append() {
        let mut patch = Patch::new();
        assert!(patch.is_empty());

        patch.replace_line(1, vec![cmd("a")]);
        patch.append(vec![cmd("b")]);

        assert!(!patch.is_empty());
        assert_eq!(patch.edits().len(), 1);
        assert_eq!(patch.append.len(), 1);
    }

    #[test]
    fn checked_replace_records_expectation() {
        let mut patch = Patch::new();
        patch.replace_line_checked(2, Expected::Text("old line".to_string()), vec![cmd("new")]);

        assert!(matches!(
            &patch.edits()[0].anchor,
            Anchor::Line { lineno: 2, expect: Some(Expected::Text(text)) } if text.as_str() == "old line"
        ));
    }

    #[test]
    fn command_anchor_convenience_methods_set_no_expectation() {
        let mut patch = Patch::new();
        patch.delete_command(3);

        assert!(matches!(
            &patch.edits()[0].anchor,
            Anchor::Command { index: 3, expect: None }
        ));
    }

    #[test]
    fn defaults_are_strict_inherit_auto() {
        let options = PatchOptions::default();
        assert_eq!(options.mode, ApplyMode::Strict);
        assert_eq!(options.indent, IndentPolicy::InheritAnchor);
        assert_eq!(options.newline, NewlineStyle::Auto);
        assert_eq!(options.parser_config, ParserConfig::default());
    }

    #[test]
    fn patch_error_display_is_descriptive() {
        let error = PatchError::LineOutOfRange { lineno: 9, total_lines: 3 };
        assert!(error.to_string().contains("line 9"));

        let error = PatchError::EmptyContent { edit_index: 0, operation: "Replace" };
        assert!(error.to_string().contains("Replace"));
    }
}
