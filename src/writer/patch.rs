//! Format-preserving patch application for existing KoiLang documents.
//!
//! A [`Patch`] describes whole-line edits against an existing source text. Applying it
//! writes the edited document while copying every untouched line verbatim, so blank
//! lines, indentation, line endings and existing formatting survive.

use std::fmt;
use std::io;

use crate::command::Command;
use crate::parser::ParserConfig;

use super::WriterConfig;
use super::generators::Generators;

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

fn leading_whitespace(text: &str) -> &str {
    let end = text
        .char_indices()
        .find(|&(_, c)| c != ' ' && c != '\t')
        .map(|(index, _)| index)
        .unwrap_or(text.len());
    &text[..end]
}

#[derive(Debug)]
struct PhysicalLine<'a> {
    text: &'a str,
    terminator: &'a str,
    lineno: usize,
    start: usize,
}

#[derive(Debug, Clone, Copy)]
struct LogicalLine {
    start_line: usize,
    end_line: usize,
    start_lineno: usize,
}

#[derive(Debug)]
struct SourceLines<'a> {
    rest: &'a str,
    bom: Option<&'a str>,
    lines: Vec<PhysicalLine<'a>>,
    groups: Vec<LogicalLine>,
    line_to_group: Vec<usize>,
    detected_newline: &'static str,
    ends_with_terminator: bool,
}

impl<'a> SourceLines<'a> {
    fn parse(source: &'a str) -> Self {
        let (bom, rest) = match source.strip_prefix('\u{feff}') {
            Some(rest) => (Some("\u{feff}"), rest),
            None => (None, source),
        };

        let mut lines = Vec::new();
        let mut start = 0usize;
        let mut lineno = 1usize;
        while start < rest.len() {
            match rest[start..].find('\n') {
                Some(relative_end) => {
                    let newline = start + relative_end;
                    let (text_end, terminator_end) =
                        if newline > start && rest.as_bytes()[newline - 1] == b'\r' {
                            (newline - 1, newline + 1)
                        } else {
                            (newline, newline + 1)
                        };
                    lines.push(PhysicalLine {
                        text: &rest[start..text_end],
                        terminator: &rest[text_end..terminator_end],
                        lineno,
                        start,
                    });
                    lineno += 1;
                    start = terminator_end;
                }
                None => {
                    lines.push(PhysicalLine {
                        text: &rest[start..],
                        terminator: "",
                        lineno,
                        start,
                    });
                    start = rest.len();
                }
            }
        }

        let detected_newline = lines
            .iter()
            .find(|line| !line.terminator.is_empty())
            .map(|line| if line.terminator == "\r\n" { "\r\n" } else { "\n" })
            .unwrap_or("\n");

        let ends_with_terminator = lines
            .last()
            .map(|line| !line.terminator.is_empty())
            .unwrap_or(false);

        let mut groups = Vec::new();
        let mut line_to_group = vec![0usize; lines.len()];
        let mut index = 0usize;
        while index < lines.len() {
            let group_index = groups.len();
            let start_line = index;
            loop {
                line_to_group[index] = group_index;
                let continues = index + 1 < lines.len()
                    && !lines[index].terminator.is_empty()
                    && lines[index].text.ends_with('\\');
                index += 1;
                if !continues {
                    break;
                }
            }
            groups.push(LogicalLine {
                start_line,
                end_line: index,
                start_lineno: lines[start_line].lineno,
            });
        }

        Self {
            rest,
            bom,
            lines,
            groups,
            line_to_group,
            detected_newline,
            ends_with_terminator,
        }
    }

    fn group_end_offset(&self, group: usize) -> usize {
        let last = &self.lines[self.groups[group].end_line - 1];
        last.start + last.text.len() + last.terminator.len()
    }

    fn group_raw(&self, group: usize) -> &'a str {
        let first = &self.lines[self.groups[group].start_line];
        &self.rest[first.start..self.group_end_offset(group)]
    }

    fn group_raw_without_final_terminator(&self, group: usize) -> &'a str {
        let first = &self.lines[self.groups[group].start_line];
        let last = &self.lines[self.groups[group].end_line - 1];
        &self.rest[first.start..last.start + last.text.len()]
    }

    fn group_ends_with_terminator(&self, group: usize) -> bool {
        !self.lines[self.groups[group].end_line - 1]
            .terminator
            .is_empty()
    }

    fn group_first_line_indent(&self, group: usize) -> &'a str {
        leading_whitespace(self.lines[self.groups[group].start_line].text)
    }

    fn last_group_indent(&self) -> &'a str {
        match self.groups.last() {
            Some(group) => leading_whitespace(self.lines[group.start_line].text),
            None => "",
        }
    }

    fn resolved_newline(&self, style: NewlineStyle) -> &'static str {
        match style {
            NewlineStyle::Lf => "\n",
            NewlineStyle::Crlf => "\r\n",
            NewlineStyle::Auto => self.detected_newline,
        }
    }
}

/// Render one command as patch line content: no leading indentation, no trailing
/// newline. Line-breaking formatter options are disabled so one command always
/// becomes one logical line.
pub(crate) fn render_command(config: &WriterConfig, command: &Command) -> io::Result<String> {
    let mut options = Generators::get_effective_options(&command.name, None, config);
    options.newline_before = false;
    options.newline_after = false;
    options.newline_before_param = false;
    options.newline_after_param = false;

    let mut buffer = Vec::new();
    Generators::write_command_with_param_options(&mut buffer, command, config, &options, None, 0)?;
    String::from_utf8(buffer).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "writer produced non-UTF-8 output")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Command;
    use crate::command::Parameter;
    use crate::writer::{FormatterOptions, NumberFormat};

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

    #[test]
    fn splits_physical_lines_lf() {
        let source = SourceLines::parse("#a 1\n#b 2");
        assert_eq!(source.lines.len(), 2);
        assert_eq!(source.lines[0].text, "#a 1");
        assert_eq!(source.lines[0].terminator, "\n");
        assert_eq!(source.lines[0].lineno, 1);
        assert_eq!(source.lines[1].text, "#b 2");
        assert_eq!(source.lines[1].terminator, "");
        assert_eq!(source.detected_newline, "\n");
        assert!(!source.ends_with_terminator);
    }

    #[test]
    fn splits_physical_lines_crlf() {
        let source = SourceLines::parse("#a 1\r\n#b 2\r\n");
        assert_eq!(source.lines.len(), 2);
        assert_eq!(source.lines[0].text, "#a 1");
        assert_eq!(source.lines[0].terminator, "\r\n");
        assert_eq!(source.lines[1].terminator, "\r\n");
        assert_eq!(source.detected_newline, "\r\n");
        assert!(source.ends_with_terminator);
    }

    #[test]
    fn empty_source_has_no_lines_or_groups() {
        let source = SourceLines::parse("");
        assert!(source.lines.is_empty());
        assert!(source.groups.is_empty());
        assert!(!source.ends_with_terminator);
        assert!(source.bom.is_none());
    }

    #[test]
    fn strips_utf8_bom() {
        let source = SourceLines::parse("\u{feff}#a\n");
        assert_eq!(source.bom, Some("\u{feff}"));
        assert_eq!(source.rest, "#a\n");
        assert_eq!(source.lines[0].text, "#a");
    }

    #[test]
    fn groups_continuation_lines() {
        let source = SourceLines::parse("Hello\\\nworld\nnext\n");
        assert_eq!(source.lines.len(), 3);
        assert_eq!(source.groups.len(), 2);
        assert_eq!(source.groups[0].start_line, 0);
        assert_eq!(source.groups[0].end_line, 2);
        assert_eq!(source.groups[0].start_lineno, 1);
        assert_eq!(source.groups[1].start_line, 2);
        assert_eq!(source.group_raw(0), "Hello\\\nworld\n");
        assert_eq!(source.line_to_group, vec![0, 0, 1]);
    }

    #[test]
    fn trailing_continuation_backslash_is_a_single_group() {
        let source = SourceLines::parse("Hello\\\n");
        assert_eq!(source.lines.len(), 1);
        assert_eq!(source.groups.len(), 1);
        assert_eq!(source.groups[0].end_line, 1);
    }

    #[test]
    fn raw_group_accessors_strip_only_the_final_terminator() {
        let source = SourceLines::parse("a\\\nb\n");
        assert_eq!(source.group_raw(0), "a\\\nb\n");
        assert_eq!(source.group_raw_without_final_terminator(0), "a\\\nb");
        assert!(source.group_ends_with_terminator(0));
    }

    #[test]
    fn indentation_helpers_read_leading_whitespace() {
        let source = SourceLines::parse("  #a\n\t#b\n");
        assert_eq!(source.group_first_line_indent(0), "  ");
        assert_eq!(source.group_first_line_indent(1), "\t");
        assert_eq!(source.last_group_indent(), "\t");
    }

    #[test]
    fn resolves_newline_style() {
        let source = SourceLines::parse("a\r\nb\n");
        assert_eq!(source.resolved_newline(NewlineStyle::Auto), "\r\n");
        assert_eq!(source.resolved_newline(NewlineStyle::Lf), "\n");
        assert_eq!(source.resolved_newline(NewlineStyle::Crlf), "\r\n");
    }

    #[test]
    fn renders_regular_command_inline() {
        let command = Command::new(
            "character",
            vec![Parameter::from("Alice"), Parameter::from("Hello, world!")],
        );
        let rendered = render_command(&WriterConfig::default(), &command).unwrap();
        assert_eq!(rendered, "#character Alice \"Hello, world!\"");
    }

    #[test]
    fn renders_raw_text_with_escaped_newlines() {
        let command = Command::new_text("Hello\nWorld");
        let rendered = render_command(&WriterConfig::default(), &command).unwrap();
        assert_eq!(rendered, "Hello\\\nWorld");
    }

    #[test]
    fn renders_annotation_and_number_commands() {
        let config = WriterConfig::default();
        assert_eq!(
            render_command(&config, &Command::new_annotation("note")).unwrap(),
            "## note"
        );
        assert_eq!(
            render_command(&config, &Command::new_number(42, Vec::new())).unwrap(),
            "#42"
        );
    }

    #[test]
    fn honors_number_format_configuration() {
        let mut command_options = std::collections::HashMap::new();
        command_options.insert(
            "hex".to_string(),
            FormatterOptions {
                number_format: NumberFormat::Hex,
                ..Default::default()
            },
        );
        let config = WriterConfig {
            command_options,
            ..Default::default()
        };
        let command = Command::new("hex", vec![Parameter::from(42)]);
        assert_eq!(render_command(&config, &command).unwrap(), "#hex 0x2a");
    }

    #[test]
    fn ignores_line_breaking_options() {
        let mut command_options = std::collections::HashMap::new();
        command_options.insert(
            "split".to_string(),
            FormatterOptions {
                newline_after_param: true,
                newline_before_param: true,
                ..Default::default()
            },
        );
        let config = WriterConfig {
            command_options,
            ..Default::default()
        };
        let command = Command::new("split", vec![Parameter::from("a"), Parameter::from("b")]);
        let rendered = render_command(&config, &command).unwrap();
        assert_eq!(rendered, "#split a b");
        assert!(!rendered.contains('\n'));
    }
}
