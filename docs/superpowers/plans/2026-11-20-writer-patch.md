# Writer Patch Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add format-preserving patch application to `koicore::writer` so callers can replace/insert/delete whole lines in an existing KoiLang document (addressed by physical line number or parsed command index) while every untouched line stays byte-for-byte identical.

**Architecture:** A new `src/writer/patch.rs` module owns the public `Patch`/`Anchor`/`Edit` model plus the private source-line model (physical → logical line groups), validation, and emission. `Writer::write_patched` validates everything first (strict atomic mode) and then streams the patched document to its output; `apply_patch` / `apply_patch_with_config` are string-returning convenience wrappers. New commands are rendered through the existing `Generators` formatting pipeline with line-breaking options disabled, so one command always maps to one logical line.

**Tech Stack:** Rust 2024 edition, `koicore` crate, `nom`-based parser (already present), Cargo test/doc-test, no new dependencies.

**Spec:** `docs/superpowers/specs/2026-11-18-writer-patch-design.md` (committed as `bfc17a8` + `3eab952`).

## Global Constraints

- Work in the `koicore` repository (`cd koicore` from the workspace root). Create the implementation branch with `git switch -c feat/writer-patch` before Task 1.
- Rust 2024 edition (let-chains are allowed).
- No `unsafe`, no new dependencies, no `#[allow(...)]` suppressions.
- Existing `Writer` behavior must not change: every pre-existing test (`cargo test`) must keep passing, and `src/writer/mod.rs` changes are limited to `mod patch;` + re-exports.
- No changes to `crates/koicore_ffi` or to `koilang-py`.
- No version bump in this change (release is a separate commit).
- All public items need doc comments (per `AGENTS.md`).
- Each task ends with its own test run and commit; commit only the files touched by that task.
- Final verification (Task 8) must run: `cargo build --release --workspace`, `cargo test`, `cargo clippy --workspace`, `cargo doc --workspace --no-deps`, `make ffi-test`.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `src/writer/patch.rs` | **Create** | All patch types, source-line model, validation, emission, entry points, unit tests. |
| `src/writer/mod.rs` | Modify | `mod patch;` + re-exports of the public patch API. |
| `src/lib.rs` | Modify | Crate-root re-exports (`Patch`, `Anchor`, `apply_patch`, ...). |
| `tests/test_patch.rs` | **Create** | Public-API integration tests (write → patch → re-parse round trips). |
| `README.md`, `README_cn.md` | Modify | Add a "patch existing documents" feature bullet. |

All patch logic lives in `src/writer/patch.rs`; it is the only file that grows substantially in this change.

---

### Task 1: Public patch types, options, errors, and builder

**Files:**
- Create: `src/writer/patch.rs`
- Modify: `src/writer/mod.rs`

**Interfaces:**
- Consumes: `crate::command::Command`, `crate::parser::ParserConfig`.
- Produces: `Anchor`, `Expected`, `Edit`, `PatchEdit`, `Patch` (with builders), `PatchOptions`, `ApplyMode`, `IndentPolicy`, `NewlineStyle`, `PatchReport`, `PatchWarning`, `PatchError` (+ `Display` / `Error` / `From<io::Error>`). Later tasks add private helpers and public entry points to this same file.

- [ ] **Step 1: Wire the module and write the failing type tests**

In `src/writer/mod.rs`, add `mod patch;` next to the other internal modules:

```rust
// Internal modules
mod config;
mod formatters;
mod generators;
mod patch;
```

And extend the re-export block below the existing `pub use self::config::{...};` line:

```rust
pub use self::patch::{
    Anchor, ApplyMode, Edit, Expected, IndentPolicy, NewlineStyle, Patch, PatchEdit, PatchError,
    PatchOptions, PatchReport, PatchWarning,
};
```

Create `src/writer/patch.rs` with only the module doc comment and the failing tests:

```rust
//! Format-preserving patch application for existing KoiLang documents.
//!
//! A [`Patch`] describes whole-line edits against an existing source text. Applying it
//! writes the edited document while copying every untouched line verbatim, so blank
//! lines, indentation, line endings and existing formatting survive.

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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p koicore --lib writer::patch`

Expected: FAIL — compile errors such as `cannot find type Patch in this scope`, `cannot find type PatchOptions`, `cannot find type PatchError`.

- [ ] **Step 3: Add the public types and implementations**

Append the following below the module doc comment (above the `#[cfg(test)] mod tests` block). Note: `std::io` is imported here for `PatchError::Io` only.

```rust
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
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p koicore --lib writer::patch`

Expected: PASS — `test result: ok.` with 5 tests in the `writer::patch::tests` module.

Then run the pre-existing suite to confirm nothing changed: `cargo test -p koicore`.

- [ ] **Step 5: Commit**

```bash
git add src/writer/patch.rs src/writer/mod.rs
git commit -m "feat(writer): add patch data model and builder"
```

---

### Task 2: Source line model (physical lines, logical line groups, BOM, EOL)

**Files:**
- Modify: `src/writer/patch.rs`

**Interfaces:**
- Consumes: nothing new.
- Produces: private `SourceLines<'a>`, `PhysicalLine<'a>`, `LogicalLine`; helpers used by validation/emission in Tasks 4–6:
  - `SourceLines::parse(source: &'a str) -> SourceLines<'a>`
  - `SourceLines::group_raw(group: usize) -> &'a str` (raw bytes including final terminator)
  - `SourceLines::group_raw_without_final_terminator(group: usize) -> &'a str`
  - `SourceLines::group_ends_with_terminator(group: usize) -> bool`
  - `SourceLines::group_first_line_indent(group: usize) -> &'a str`
  - `SourceLines::last_group_indent(&self) -> &'a str`
  - `SourceLines::resolved_newline(&self, style: NewlineStyle) -> &'static str`
  - Fields: `rest`, `bom`, `lines`, `groups`, `line_to_group`, `detected_newline`, `ends_with_terminator`.

- [ ] **Step 1: Write the failing line-model tests**

Inside the existing `#[cfg(test)] mod tests` block in `src/writer/patch.rs`, append:

```rust
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p koicore --lib writer::patch`

Expected: FAIL — `cannot find type SourceLines in this scope`, `cannot find type NewlineStyle` (if not already in scope), `no method named group_raw`.

- [ ] **Step 3: Add the line model**

Append below the `impl From<io::Error> for PatchError` block and above `#[cfg(test)] mod tests`:

```rust
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
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p koicore --lib writer::patch`

Expected: PASS — all Task 1 + Task 2 tests pass (`test result: ok.`).

- [ ] **Step 5: Commit**

```bash
git add src/writer/patch.rs
git commit -m "feat(writer): add source line model for patch application"
```

---

### Task 3: Patch renderer (`render_command`)

**Files:**
- Modify: `src/writer/patch.rs`

**Interfaces:**
- Consumes: `super::generators::Generators`, `super::WriterConfig`.
- Produces: `pub(crate) fn render_command(config: &WriterConfig, command: &Command) -> io::Result<String>` — one command rendered as a single patch line body with no indentation and no trailing newline; line-breaking formatter options are forced off.

- [ ] **Step 1: Write the failing renderer tests**

Add the needed imports at the top of the `#[cfg(test)] mod tests` block (below the existing `use crate::command::Command;`):

```rust
    use crate::command::Parameter;
    use crate::writer::{FormatterOptions, NumberFormat};
```

Append to the tests module:

```rust
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
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p koicore --lib writer::patch`

Expected: FAIL — `cannot find function render_command in this scope` and `cannot find type WriterConfig in this scope`.

- [ ] **Step 3: Add the renderer**

At the top of the file, replace the current imports block:

```rust
use std::fmt;
use std::io;

use crate::command::Command;
use crate::parser::ParserConfig;

use super::WriterConfig;
use super::generators::Generators;
```

(Keep the existing imports and add the two `use super::...` lines.)

Append below the `SourceLines` impl block:

```rust
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
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p koicore --lib writer::patch`

Expected: PASS — Task 1–3 tests all pass.

Also verify the pre-existing writer tests still pass untouched: `cargo test -p koicore --lib writer`.

- [ ] **Step 5: Commit**

```bash
git add src/writer/patch.rs
git commit -m "feat(writer): render patch commands without line-breaking options"
```

---

### Task 4: Anchor resolution and expectation validation (strict + lenient)

**Files:**
- Modify: `src/writer/patch.rs`

**Interfaces:**
- Consumes: `Parser`, `StringInputSource`, `ParseError`, `SourceLines`, `render_command` (not used yet).
- Produces:
  - `struct ParsedCommand { command: Command, start_lineno: usize }`
  - `fn parse_commands(source: &str, config: &ParserConfig) -> Result<Vec<ParsedCommand>, Box<ParseError>>`
  - `fn parse_group_command(raw: &str, config: &ParserConfig) -> Result<Option<Command>, Box<ParseError>>`
  - `fn check_line_expectation(source: &SourceLines<'_>, line_index: usize, group: usize, expected: &Expected, parser_config: &ParserConfig) -> Result<(), ExpectationError>`
  - `struct EmissionPlan { resolved, primary, before, after, warnings }` with `EmissionPlan::new(edit_count, group_count)`
  - `fn validate(source: &SourceLines<'_>, patch: &Patch) -> Result<EmissionPlan, PatchError>` (Targets 4-5; Task 5 adds conflicts)

- [ ] **Step 1: Write the failing validation tests**

Add this import to the top of the `#[cfg(test)] mod tests` block:

```rust
    use crate::parser::{Parser, StringInputSource};
```

Append to the tests module:

```rust
    #[test]
    fn line_out_of_range_is_rejected() {
        let source = SourceLines::parse("#a\n");
        let mut patch = Patch::new();
        patch.replace_line(2, vec![cmd("b")]);

        let error = validate(&source, &patch).unwrap_err();
        assert!(matches!(
            error,
            PatchError::LineOutOfRange { lineno: 2, total_lines: 1 }
        ));
    }

    #[test]
    fn continuation_line_target_is_rejected() {
        let source = SourceLines::parse("Hello\\\nworld\n");
        let mut patch = Patch::new();
        patch.replace_line(2, vec![cmd("b")]);

        let error = validate(&source, &patch).unwrap_err();
        assert!(matches!(
            error,
            PatchError::TargetInsideContinuation { lineno: 2, start_lineno: 1 }
        ));
    }

    #[test]
    fn expected_text_matches_and_mismatches() {
        let source = SourceLines::parse("#cmd a\n");

        let mut good = Patch::new();
        good.edit(
            Anchor::Line {
                lineno: 1,
                expect: Some(Expected::Text("#cmd a".to_string())),
            },
            Edit::Delete,
        );
        assert!(validate(&source, &good).is_ok());

        let mut bad = Patch::new();
        bad.edit(
            Anchor::Line {
                lineno: 1,
                expect: Some(Expected::Text("#cmd b".to_string())),
            },
            Edit::Delete,
        );
        let error = validate(&source, &bad).unwrap_err();
        assert!(matches!(error, PatchError::AnchorMismatch { lineno: 1, .. }));
    }

    #[test]
    fn expected_text_normalizes_one_trailing_newline() {
        let source = SourceLines::parse("#cmd a\r\n");
        let mut patch = Patch::new();
        patch.edit(
            Anchor::Line {
                lineno: 1,
                expect: Some(Expected::Text("#cmd a\n".to_string())),
            },
            Edit::Delete,
        );
        assert!(validate(&source, &patch).is_ok());
    }

    #[test]
    fn expected_command_matches_and_mismatches() {
        let source = SourceLines::parse("#cmd a\n");
        let expected = Command::new("cmd", vec![Parameter::from("a")]);

        let mut good = Patch::new();
        good.edit(
            Anchor::Line {
                lineno: 1,
                expect: Some(Expected::Command(expected.clone())),
            },
            Edit::Delete,
        );
        assert!(validate(&source, &good).is_ok());

        let mut bad = Patch::new();
        bad.edit(
            Anchor::Line {
                lineno: 1,
                expect: Some(Expected::Command(Command::new("cmd", vec![Parameter::from("b")]))),
            },
            Edit::Delete,
        );
        let error = validate(&source, &bad).unwrap_err();
        assert!(matches!(error, PatchError::AnchorMismatch { lineno: 1, .. }));
    }

    #[test]
    fn expected_command_parses_a_continuation_group() {
        let source = SourceLines::parse("Hello\\\nworld\n");
        let mut patch = Patch::new();
        patch.edit(
            Anchor::Line {
                lineno: 1,
                expect: Some(Expected::Command(Command::new_text("Hello\nworld"))),
            },
            Edit::Delete,
        );
        assert!(validate(&source, &patch).is_ok());

        let crlf_source = SourceLines::parse("Hello\\\r\nworld\r\n");
        assert!(validate(&crlf_source, &patch).is_ok());
    }

    #[test]
    fn lenient_mode_skips_line_expectation_that_fails_to_parse() {
        let source = SourceLines::parse("#cmd \"unclosed string\n");
        let mut patch = Patch::new();
        patch.options_mut().mode = ApplyMode::Lenient;
        patch.edit(
            Anchor::Line {
                lineno: 1,
                expect: Some(Expected::Command(Command::new("a", Vec::new()))),
            },
            Edit::Delete,
        );

        let plan = validate(&source, &patch).unwrap();
        assert_eq!(
            plan.warnings,
            vec![PatchWarning::AnchorMismatch { edit_index: 0 }]
        );
        assert_eq!(plan.resolved, vec![None]);
    }

    #[test]
    fn payload_validation_precedes_source_parsing() {
        let source = SourceLines::parse("#cmd \"unclosed string\n");
        let mut patch = Patch::new();
        patch.replace_command(0, Vec::new());

        let error = validate(&source, &patch).unwrap_err();
        assert!(matches!(
            error,
            PatchError::EmptyContent { edit_index: 0, operation: "Replace" }
        ));
    }

    #[test]
    fn command_index_out_of_range_is_rejected() {
        let source = SourceLines::parse("#a\n");
        let mut patch = Patch::new();
        patch.delete_command(5);

        let error = validate(&source, &patch).unwrap_err();
        assert!(matches!(
            error,
            PatchError::CommandIndexOutOfRange { index: 5, total_commands: 1 }
        ));
    }

    #[test]
    fn command_index_resolves_to_the_logical_line() {
        let source = SourceLines::parse("#a\n#b\n");
        let mut patch = Patch::new();
        patch.delete_command(1);

        let plan = validate(&source, &patch).unwrap();
        assert_eq!(plan.resolved, vec![Some(1)]);
    }

    #[test]
    fn command_index_does_not_count_continuation_lines_as_commands() {
        let source = SourceLines::parse("Hello\\\r\nworld\r\n");
        let mut patch = Patch::new();
        patch.delete_command(1);

        let error = validate(&source, &patch).unwrap_err();
        assert!(matches!(
            error,
            PatchError::CommandIndexOutOfRange { index: 1, total_commands: 1 }
        ));
    }

    #[test]
    fn command_anchor_expectation_mismatch_is_reported() {
        let source = SourceLines::parse("#a\n");
        let mut patch = Patch::new();
        patch.edit(
            Anchor::Command {
                index: 0,
                expect: Some(Command::new("other", Vec::new())),
            },
            Edit::Delete,
        );

        let error = validate(&source, &patch).unwrap_err();
        assert!(matches!(error, PatchError::CommandMismatch { index: 0, .. }));
    }

    #[test]
    fn command_anchor_requires_parseable_source() {
        let source = SourceLines::parse("#cmd \"unclosed string\n");
        let mut patch = Patch::new();
        patch.delete_command(0);

        let error = validate(&source, &patch).unwrap_err();
        assert!(matches!(error, PatchError::SourceParse(_)));
    }

    #[test]
    fn lenient_mode_source_parse_error_is_still_fatal() {
        let source = SourceLines::parse("#cmd \"unclosed string\n");
        let mut patch = Patch::new();
        patch.options_mut().mode = ApplyMode::Lenient;
        patch.delete_command(0);

        let error = validate(&source, &patch).unwrap_err();
        assert!(matches!(error, PatchError::SourceParse(_)));
    }

    #[test]
    fn empty_replace_payload_is_rejected() {
        let source = SourceLines::parse("#a\n");
        let mut patch = Patch::new();
        patch.replace_line(1, Vec::new());

        let error = validate(&source, &patch).unwrap_err();
        assert!(matches!(
            error,
            PatchError::EmptyContent { edit_index: 0, operation: "Replace" }
        ));
    }

    #[test]
    fn lenient_mode_skips_out_of_range_edit_with_warning() {
        let source = SourceLines::parse("#a\n");
        let mut patch = Patch::new();
        patch.options_mut().mode = ApplyMode::Lenient;
        patch.replace_line(9, vec![cmd("b")]);

        let plan = validate(&source, &patch).unwrap();
        assert_eq!(plan.warnings, vec![PatchWarning::OutOfRange { edit_index: 0 }]);
        assert_eq!(plan.resolved, vec![None]);
    }

    #[test]
    fn line_anchors_do_not_require_parsing_the_source() {
        let source = SourceLines::parse("#cmd \"unclosed string\n");
        let mut patch = Patch::new();
        patch.delete_line(1);
        assert!(validate(&source, &patch).is_ok());
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p koicore --lib writer::patch`

Expected: FAIL — `cannot find function validate`, `cannot find struct EmissionPlan`.

- [ ] **Step 3: Add parser helpers, expectation checking, and `validate`**

Update the imports at the top of the file to:

```rust
use std::fmt;
use std::io;

use crate::command::Command;
use crate::parser::{ParseError, Parser, ParserConfig, StringInputSource};

use super::WriterConfig;
use super::generators::Generators;
```

Append below the `render_command` function:

```rust
#[derive(Debug, Clone)]
struct ParsedCommand {
    command: Command,
    start_lineno: usize,
}

fn parse_commands(
    source: &str,
    config: &ParserConfig,
) -> Result<Vec<ParsedCommand>, Box<ParseError>> {
    let normalized = source.replace("\r\n", "\n");
    let mut parser = Parser::new(StringInputSource::new(&normalized), config.clone());
    let mut commands = Vec::new();
    while let Some((command, line_source)) = parser.next_command_with_source()? {
        commands.push(ParsedCommand {
            command,
            start_lineno: line_source.lineno,
        });
    }
    Ok(commands)
}

fn parse_group_command(
    raw: &str,
    config: &ParserConfig,
) -> Result<Option<Command>, Box<ParseError>> {
    let normalized = raw.replace("\r\n", "\n");
    let mut parser = Parser::new(StringInputSource::new(&normalized), config.clone());
    let first = parser.next_command_with_source()?;
    Ok(first.map(|(command, _)| command))
}

impl From<Box<ParseError>> for PatchError {
    fn from(error: Box<ParseError>) -> Self {
        PatchError::SourceParse(error)
    }
}

fn normalize_expected_text(expected: &str) -> &str {
    if let Some(stripped) = expected.strip_suffix("\r\n") {
        stripped
    } else if let Some(stripped) = expected.strip_suffix('\n') {
        stripped
    } else {
        expected
    }
}

enum ExpectationError {
    Mismatch { expected: Expected, actual: String },
    Parse(Box<ParseError>),
}

fn check_line_expectation(
    source: &SourceLines<'_>,
    line_index: usize,
    group: usize,
    expected: &Expected,
    parser_config: &ParserConfig,
) -> Result<(), ExpectationError> {
    match expected {
        Expected::Text(expected_text) => {
            let normalized = normalize_expected_text(expected_text);
            let actual = source.lines[line_index].text;
            if normalized == actual {
                Ok(())
            } else {
                Err(ExpectationError::Mismatch {
                    expected: Expected::Text(normalized.to_string()),
                    actual: actual.to_string(),
                })
            }
        }
        Expected::Command(expected_command) => {
            let parsed = parse_group_command(source.group_raw(group), parser_config)
                .map_err(ExpectationError::Parse)?;
            match parsed {
                Some(actual) if actual == *expected_command => Ok(()),
                _ => Err(ExpectationError::Mismatch {
                    expected: Expected::Command(expected_command.clone()),
                    actual: source
                        .group_raw_without_final_terminator(group)
                        .to_string(),
                }),
            }
        }
    }
}

fn empty_content_operation(edit: &Edit) -> Option<&'static str> {
    let (commands, operation) = match edit {
        Edit::Replace(commands) => (commands, "Replace"),
        Edit::InsertBefore(commands) => (commands, "InsertBefore"),
        Edit::InsertAfter(commands) => (commands, "InsertAfter"),
        Edit::Delete => return None,
    };
    if commands.is_empty() { Some(operation) } else { None }
}

struct EmissionPlan {
    resolved: Vec<Option<usize>>,
    primary: Vec<Option<usize>>,
    before: Vec<Vec<usize>>,
    after: Vec<Vec<usize>>,
    warnings: Vec<PatchWarning>,
}

impl EmissionPlan {
    fn new(edit_count: usize, group_count: usize) -> Self {
        Self {
            resolved: vec![None; edit_count],
            primary: vec![None; group_count],
            before: vec![Vec::new(); group_count],
            after: vec![Vec::new(); group_count],
            warnings: Vec::new(),
        }
    }
}

fn validate(source: &SourceLines<'_>, patch: &Patch) -> Result<EmissionPlan, PatchError> {
    let mode = patch.options.mode;
    let mut plan = EmissionPlan::new(patch.edits.len(), source.groups.len());

    // Payload validation runs before any source parsing so that an empty
    // replace/insert payload wins over a source parse error.
    for (edit_index, patch_edit) in patch.edits.iter().enumerate() {
        if let Some(operation) = empty_content_operation(&patch_edit.edit) {
            if mode == ApplyMode::Strict {
                return Err(PatchError::EmptyContent { edit_index, operation });
            }
            plan.warnings.push(PatchWarning::EmptyContent { edit_index });
        }
    }

    let has_command_anchor = patch
        .edits
        .iter()
        .any(|edit| matches!(&edit.anchor, Anchor::Command { .. }));
    let parsed_commands = if has_command_anchor {
        Some(parse_commands(source.rest, &patch.options.parser_config)?)
    } else {
        None
    };

    for (edit_index, patch_edit) in patch.edits.iter().enumerate() {
        if empty_content_operation(&patch_edit.edit).is_some() {
            continue;
        }

        let group = match &patch_edit.anchor {
            Anchor::Line { lineno, expect } => {
                let total_lines = source.lines.len();
                if *lineno == 0 || *lineno > total_lines {
                    if mode == ApplyMode::Strict {
                        return Err(PatchError::LineOutOfRange { lineno: *lineno, total_lines });
                    }
                    plan.warnings.push(PatchWarning::OutOfRange { edit_index });
                    continue;
                }

                let line_index = lineno - 1;
                let group = source.line_to_group[line_index];
                let group_start_lineno = source.groups[group].start_lineno;
                if source.groups[group].start_line != line_index {
                    if mode == ApplyMode::Strict {
                        return Err(PatchError::TargetInsideContinuation {
                            lineno: *lineno,
                            start_lineno: group_start_lineno,
                        });
                    }
                    plan.warnings.push(PatchWarning::InsideContinuation { edit_index });
                    continue;
                }

                if let Some(expected) = expect {
                    match check_line_expectation(
                        source,
                        line_index,
                        group,
                        expected,
                        &patch.options.parser_config,
                    ) {
                        Ok(()) => {}
                        Err(ExpectationError::Mismatch { expected, actual }) => {
                            if mode == ApplyMode::Strict {
                                return Err(PatchError::AnchorMismatch {
                                    lineno: *lineno,
                                    expected,
                                    actual,
                                });
                            }
                            plan.warnings.push(PatchWarning::AnchorMismatch { edit_index });
                            continue;
                        }
                        Err(ExpectationError::Parse(error)) => {
                            if mode == ApplyMode::Strict {
                                return Err(PatchError::SourceParse(error));
                            }
                            plan.warnings.push(PatchWarning::AnchorMismatch { edit_index });
                            continue;
                        }
                    }
                }

                group
            }
            Anchor::Command { index, expect } => {
                let commands = parsed_commands
                    .as_ref()
                    .expect("command anchors always trigger a full parse");
                if *index >= commands.len() {
                    if mode == ApplyMode::Strict {
                        return Err(PatchError::CommandIndexOutOfRange {
                            index: *index,
                            total_commands: commands.len(),
                        });
                    }
                    plan.warnings.push(PatchWarning::OutOfRange { edit_index });
                    continue;
                }

                let parsed = &commands[*index];
                if let Some(expected) = expect
                    && parsed.command != *expected
                {
                    if mode == ApplyMode::Strict {
                        return Err(PatchError::CommandMismatch {
                            index: *index,
                            lineno: parsed.start_lineno,
                            expected: expected.clone(),
                            actual: parsed.command.clone(),
                        });
                    }
                    plan.warnings.push(PatchWarning::AnchorMismatch { edit_index });
                    continue;
                }

                source.line_to_group[parsed.start_lineno - 1]
            }
        };

        plan.resolved[edit_index] = Some(group);
    }

    Ok(plan)
}
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p koicore --lib writer::patch`

Expected: PASS — Task 1–4 tests all pass. A `dead_code` warning about the not-yet-used `EmissionPlan` `primary` / `before` / `after` fields is expected in this task; it disappears after Task 5.

- [ ] **Step 5: Commit**

```bash
git add src/writer/patch.rs
git commit -m "feat(writer): resolve patch anchors and validate expectations"
```

---

### Task 5: Conflict detection, ordering plans, and lenient conflict handling

**Files:**
- Modify: `src/writer/patch.rs`

**Interfaces:**
- Consumes: `validate` / `EmissionPlan` from Task 4.
- Produces: `EmissionPlan.primary` / `.before` / `.after` fully populated by `validate`, with conflict errors (`ConflictingEdits`) in strict mode and `PatchWarning::Conflict` in lenient mode.

- [ ] **Step 1: Write the failing conflict tests**

Append to the tests module:

```rust
    #[test]
    fn duplicate_replace_on_the_same_line_conflicts() {
        let source = SourceLines::parse("#a\n");
        let mut patch = Patch::new();
        patch.replace_line(1, vec![cmd("x")]);
        patch.replace_line(1, vec![cmd("y")]);

        let error = validate(&source, &patch).unwrap_err();
        assert!(matches!(
            error,
            PatchError::ConflictingEdits { lineno: 1, first_edit: 0, second_edit: 1 }
        ));
    }

    #[test]
    fn replace_and_insert_are_allowed_together() {
        let source = SourceLines::parse("#a\n");
        let mut patch = Patch::new();
        patch.replace_line(1, vec![cmd("x")]);
        patch.insert_before_line(1, vec![cmd("before")]);
        patch.insert_after_line(1, vec![cmd("after")]);

        let plan = validate(&source, &patch).unwrap();
        assert_eq!(plan.primary[0], Some(0));
        assert_eq!(plan.before[0], vec![1]);
        assert_eq!(plan.after[0], vec![2]);
    }

    #[test]
    fn insert_order_follows_patch_edit_order() {
        let source = SourceLines::parse("#a\n");
        let mut patch = Patch::new();
        patch.insert_after_line(1, vec![cmd("first")]);
        patch.insert_after_line(1, vec![cmd("second")]);

        let plan = validate(&source, &patch).unwrap();
        assert_eq!(plan.after[0], vec![0, 1]);
    }

    #[test]
    fn delete_and_insert_are_allowed_together() {
        let source = SourceLines::parse("#a\n");
        let mut patch = Patch::new();
        patch.delete_line(1);
        patch.insert_after_line(1, vec![cmd("after")]);

        let plan = validate(&source, &patch).unwrap();
        assert!(matches!(&patch.edits()[0].edit, Edit::Delete));
        assert_eq!(plan.primary[0], Some(0));
        assert_eq!(plan.after[0], vec![1]);
    }

    #[test]
    fn lenient_mode_skips_conflicting_edit_with_warning() {
        let source = SourceLines::parse("#a\n");
        let mut patch = Patch::new();
        patch.options_mut().mode = ApplyMode::Lenient;
        patch.replace_line(1, vec![cmd("first")]);
        patch.replace_line(1, vec![cmd("second")]);

        let plan = validate(&source, &patch).unwrap();
        assert_eq!(plan.primary[0], Some(0));
        assert_eq!(plan.resolved, vec![Some(0), None]);
        assert_eq!(
            plan.warnings,
            vec![PatchWarning::Conflict { edit_index: 1, owner_edit_index: 0 }]
        );
    }

    #[test]
    fn line_and_command_anchors_on_the_same_line_conflict() {
        let source = SourceLines::parse("#a\n");
        let mut patch = Patch::new();
        patch.delete_line(1);
        patch.delete_command(0);

        let error = validate(&source, &patch).unwrap_err();
        assert!(matches!(
            error,
            PatchError::ConflictingEdits { lineno: 1, first_edit: 0, second_edit: 1 }
        ));
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p koicore --lib writer::patch`

Expected: FAIL — `replace_and_insert_are_allowed_together` panics on `assert_eq!(plan.primary[0], Some(0))` (it is `None`), and conflict tests return `Ok` instead of `ConflictingEdits`.

- [ ] **Step 3: Add conflict detection to `validate`**

In `fn validate`, replace the final `Ok(plan)` line with:

```rust
    let mut primary_owner: Vec<Option<usize>> = vec![None; source.groups.len()];
    for (edit_index, patch_edit) in patch.edits.iter().enumerate() {
        let Some(group) = plan.resolved[edit_index] else {
            continue;
        };
        match &patch_edit.edit {
            Edit::Replace(_) | Edit::Delete => {
                if let Some(first_edit) = primary_owner[group] {
                    if mode == ApplyMode::Strict {
                        return Err(PatchError::ConflictingEdits {
                            lineno: source.groups[group].start_lineno,
                            first_edit,
                            second_edit: edit_index,
                        });
                    }
                    plan.warnings.push(PatchWarning::Conflict {
                        edit_index,
                        owner_edit_index: first_edit,
                    });
                    plan.resolved[edit_index] = None;
                    continue;
                }
                primary_owner[group] = Some(edit_index);
                plan.primary[group] = Some(edit_index);
            }
            Edit::InsertBefore(_) => plan.before[group].push(edit_index),
            Edit::InsertAfter(_) => plan.after[group].push(edit_index),
        }
    }

    Ok(plan)
```

Note: in `lenient_mode_skips_conflicting_edit_with_warning` the skipped edit's `resolved` slot is reset to `None`; do not also remove it from `plan.before`/`plan.after` because conflict resolution only applies to `Replace`/`Delete`.

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p koicore --lib writer::patch`

Expected: PASS — Task 1–5 tests all pass.

- [ ] **Step 5: Commit**

```bash
git add src/writer/patch.rs
git commit -m "feat(writer): detect patch conflicts and plan edit ordering"
```

---

### Task 6: Emission, `Writer::write_patched`, and `apply_patch` entry points

**Files:**
- Modify: `src/writer/patch.rs`
- Modify: `src/writer/mod.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `SourceLines`, `validate`, `render_command`, `Patch*` types.
- Produces:
  - `impl<T: Write> Writer<T> { pub fn write_patched(&mut self, source_text: &str, patch: &Patch) -> Result<PatchReport, PatchError> }`
  - `pub fn apply_patch(source: &str, patch: &Patch) -> Result<String, PatchError>`
  - `pub fn apply_patch_with_config(source: &str, patch: &Patch, config: &WriterConfig) -> Result<String, PatchError>`
  - Crate-root re-exports in `src/lib.rs`.

- [ ] **Step 1: Write the failing emission tests**

Append to the tests module:

```rust
    fn cmd_with(name: &str, params: Vec<&str>) -> Command {
        Command::new(name, params.into_iter().map(Parameter::from).collect())
    }

    #[test]
    fn replaces_one_line_and_preserves_the_rest() {
        let source = "#a\n#b\n";
        let mut patch = Patch::new();
        patch.replace_line(2, vec![cmd_with("c", vec!["x"])]);
        assert_eq!(apply_patch(source, &patch).unwrap(), "#a\n#c x\n");
    }

    #[test]
    fn inserts_before_and_after() {
        let source = "#a\n";
        let mut patch = Patch::new();
        patch.insert_before_line(1, vec![cmd("before")]);
        patch.insert_after_line(1, vec![cmd("after")]);
        assert_eq!(apply_patch(source, &patch).unwrap(), "#before\n#a\n#after\n");
    }

    #[test]
    fn deletes_a_line() {
        let source = "#a\n#b\n#c\n";
        let mut patch = Patch::new();
        patch.delete_line(2);
        assert_eq!(apply_patch(source, &patch).unwrap(), "#a\n#c\n");
    }

    #[test]
    fn appends_to_source_without_trailing_newline() {
        let mut patch = Patch::new();
        patch.append(vec![cmd("b")]);
        assert_eq!(apply_patch("#a", &patch).unwrap(), "#a\n#b");
    }

    #[test]
    fn appends_to_source_with_trailing_newline() {
        let mut patch = Patch::new();
        patch.append(vec![cmd("b")]);
        assert_eq!(apply_patch("#a\n", &patch).unwrap(), "#a\n#b\n");
    }

    #[test]
    fn replacing_the_last_line_keeps_missing_trailing_newline() {
        let mut patch = Patch::new();
        patch.replace_line(2, vec![cmd("c")]);
        assert_eq!(apply_patch("#a\n#b", &patch).unwrap(), "#a\n#c");
    }

    #[test]
    fn deleting_the_last_line_strips_the_previous_terminator() {
        let mut patch = Patch::new();
        patch.delete_line(2);
        assert_eq!(apply_patch("#a\n#b", &patch).unwrap(), "#a");
    }

    #[test]
    fn detects_crlf_for_rendered_lines() {
        let source = "#a 1\r\n#b 2\r\n";
        let mut patch = Patch::new();
        patch.replace_line(2, vec![cmd_with("c", vec!["3"])]);
        assert_eq!(apply_patch(source, &patch).unwrap(), "#a 1\r\n#c 3\r\n");
    }

    #[test]
    fn preserves_utf8_bom() {
        let source = "\u{feff}#a\n";
        let mut patch = Patch::new();
        patch.replace_line(1, vec![cmd("b")]);
        assert_eq!(apply_patch(source, &patch).unwrap(), "\u{feff}#b\n");
    }

    #[test]
    fn inherits_anchor_indentation() {
        let source = "    #a\nTail\n";
        let mut patch = Patch::new();
        patch.insert_after_line(1, vec![cmd("b")]);
        assert_eq!(apply_patch(source, &patch).unwrap(), "    #a\n    #b\nTail\n");
    }

    #[test]
    fn indent_policy_none_removes_inherited_indent() {
        let source = "    #a\nTail\n";
        let mut patch = Patch::new();
        patch.options_mut().indent = IndentPolicy::None;
        patch.insert_after_line(1, vec![cmd("b")]);
        assert_eq!(apply_patch(source, &patch).unwrap(), "    #a\n#b\nTail\n");
    }

    #[test]
    fn empty_patch_returns_source_unchanged() {
        assert_eq!(
            apply_patch("\u{feff}#a\r\n", &Patch::new()).unwrap(),
            "\u{feff}#a\r\n"
        );
    }

    #[test]
    fn appends_to_empty_source() {
        let mut patch = Patch::new();
        patch.append(vec![cmd("b")]);
        assert_eq!(apply_patch("", &patch).unwrap(), "#b");
    }

    #[test]
    fn deleting_every_line_produces_empty_output() {
        let source = "#a\n";
        let mut patch = Patch::new();
        patch.delete_line(1);
        assert_eq!(apply_patch(source, &patch).unwrap(), "");
    }

    #[test]
    fn replaces_a_blank_line() {
        let source = "\n#a\n";
        let mut patch = Patch::new();
        patch.replace_line(1, vec![cmd("x")]);
        assert_eq!(apply_patch(source, &patch).unwrap(), "#x\n#a\n");
    }

    #[test]
    fn append_inherits_last_line_indentation() {
        let source = "  #a\n";
        let mut patch = Patch::new();
        patch.append(vec![cmd("b")]);
        assert_eq!(apply_patch(source, &patch).unwrap(), "  #a\n  #b\n");
    }

    #[test]
    fn fixed_indent_policy_applies_to_new_lines() {
        let source = "#a\n";
        let mut patch = Patch::new();
        patch.options_mut().indent = IndentPolicy::Fixed("  ".to_string());
        patch.insert_after_line(1, vec![cmd("b")]);
        assert_eq!(apply_patch(source, &patch).unwrap(), "#a\n  #b\n");
    }

    #[test]
    fn mixed_line_endings_keep_untouched_bytes() {
        let source = "#a\r\n#b\n";
        let mut patch = Patch::new();
        patch.replace_line(2, vec![cmd("c")]);
        assert_eq!(apply_patch(source, &patch).unwrap(), "#a\r\n#c\r\n");
    }

    #[test]
    fn report_counts_applied_edits() {
        let source = "#a\n#b\n#c\n";
        let mut patch = Patch::new();
        patch.replace_line(1, vec![cmd("x")]);
        patch.insert_after_line(2, vec![cmd("y")]);
        patch.delete_line(3);
        patch.append(vec![cmd("z")]);

        let mut buffer = Vec::new();
        let report = {
            let mut writer = Writer::new(&mut buffer, WriterConfig::default());
            writer.write_patched(source, &patch).unwrap()
        };

        assert_eq!(String::from_utf8(buffer).unwrap(), "#x\n#b\n#y\n#z\n");
        assert_eq!(report.lines_before, 3);
        assert_eq!(report.lines_after, 4);
        assert_eq!(report.replaced, 1);
        assert_eq!(report.inserted, 1);
        assert_eq!(report.deleted, 1);
        assert_eq!(report.appended, 1);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn strict_failure_writes_nothing() {
        let mut buffer = Vec::new();
        {
            let mut writer = Writer::new(&mut buffer, WriterConfig::default());
            let mut patch = Patch::new();
            patch.replace_line(2, vec![cmd("x")]);
            let result = writer.write_patched("#a\n", &patch);
            assert!(matches!(
                result,
                Err(PatchError::LineOutOfRange { lineno: 2, total_lines: 1 })
            ));
        }
        assert!(buffer.is_empty());
    }

    #[test]
    fn write_patched_matches_apply_patch() {
        let source = "#a\n#b\n";
        let mut patch = Patch::new();
        patch.replace_line(2, vec![cmd_with("c", vec!["x"])]);
        let expected = apply_patch(source, &patch).unwrap();

        let mut buffer = Vec::new();
        {
            let mut writer = Writer::new(&mut buffer, WriterConfig::default());
            writer.write_patched(source, &patch).unwrap();
        }
        assert_eq!(String::from_utf8(buffer).unwrap(), expected);
    }

    #[test]
    fn command_anchor_replace_uses_parsed_index() {
        let source = "#a\n#b\n#c\n";
        let mut patch = Patch::new();
        patch.replace_command(1, vec![cmd_with("b", vec!["changed"])]);
        assert_eq!(apply_patch(source, &patch).unwrap(), "#a\n#b changed\n#c\n");
    }

    #[test]
    fn replace_with_multiple_commands_emits_consecutive_lines() {
        let source = "#a\n";
        let mut patch = Patch::new();
        patch.replace_line(
            1,
            vec![
                cmd_with("cmd", vec!["1"]),
                Command::new_text("hi"),
                Command::new_annotation("note"),
            ],
        );
        assert_eq!(apply_patch(source, &patch).unwrap(), "#cmd 1\nhi\n## note\n");
    }

    #[test]
    fn raw_text_with_embedded_newlines_round_trips() {
        let source = "#a\n";
        let mut patch = Patch::new();
        patch.replace_line(1, vec![Command::new_text("Hello\nWorld")]);
        let output = apply_patch(source, &patch).unwrap();
        assert_eq!(output, "Hello\\\nWorld\n");

        let mut parser = Parser::new(StringInputSource::new(&output), ParserConfig::default());
        let (command, _) = parser.next_command_with_source().unwrap().unwrap();
        assert_eq!(command, Command::new_text("Hello\nWorld"));
    }

    #[test]
    fn apply_patch_with_config_uses_writer_formatting() {
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
        let mut patch = Patch::new();
        patch.replace_line(1, vec![Command::new("hex", vec![Parameter::from(42)])]);
        assert_eq!(
            apply_patch_with_config("#a\n", &patch, &config).unwrap(),
            "#hex 0x2a\n"
        );
    }
```

- [ ] **Step 2: Run the tests to verify they fail**

Run: `cargo test -p koicore --lib writer::patch`

Expected: FAIL — `cannot find function apply_patch`, `cannot find function apply_patch_with_config`, `no method named write_patched`, `cannot find type Writer`.

- [ ] **Step 3: Update imports, add the emitter, entry points, and re-exports**

At the top of `src/writer/patch.rs`, change:

```rust
use std::fmt;
use std::io;

use crate::command::Command;
use crate::parser::{ParseError, Parser, ParserConfig, StringInputSource};

use super::WriterConfig;
use super::generators::Generators;
```

to:

```rust
use std::fmt;
use std::io::{self, Write};

use crate::command::Command;
use crate::parser::{ParseError, Parser, ParserConfig, StringInputSource};

use super::Writer;
use super::WriterConfig;
use super::generators::Generators;
```

Append below the `validate` function:

```rust
fn edit_commands(edit: &Edit) -> &[Command] {
    match edit {
        Edit::Replace(commands) | Edit::InsertBefore(commands) | Edit::InsertAfter(commands) => {
            commands
        }
        Edit::Delete => &[],
    }
}

fn total_elements(source: &SourceLines<'_>, patch: &Patch, plan: &EmissionPlan) -> usize {
    let mut total = patch.append.len();
    for group in 0..source.groups.len() {
        for &edit_index in &plan.before[group] {
            total += edit_commands(&patch.edits[edit_index].edit).len();
        }
        total += match plan.primary[group] {
            Some(edit_index) => match &patch.edits[edit_index].edit {
                Edit::Replace(commands) => commands.len(),
                _ => 0,
            },
            None => 1,
        };
        for &edit_index in &plan.after[group] {
            total += edit_commands(&patch.edits[edit_index].edit).len();
        }
    }
    total
}

fn edit_indent(
    source: &SourceLines<'_>,
    patch: &Patch,
    plan: &EmissionPlan,
    edit_index: usize,
) -> String {
    match &patch.options.indent {
        IndentPolicy::None => String::new(),
        IndentPolicy::Fixed(prefix) => prefix.clone(),
        IndentPolicy::InheritAnchor => {
            let group = plan.resolved[edit_index]
                .expect("planned edits always have a resolved target");
            source.group_first_line_indent(group).to_string()
        }
    }
}

fn append_indent(source: &SourceLines<'_>, patch: &Patch) -> String {
    match &patch.options.indent {
        IndentPolicy::InheritAnchor => source.last_group_indent().to_string(),
        IndentPolicy::None => String::new(),
        IndentPolicy::Fixed(prefix) => prefix.clone(),
    }
}

struct Emitter<'a, W: Write> {
    writer: &'a mut W,
    eol: &'static str,
    total: usize,
    emitted: usize,
    first: bool,
    previous_terminated: bool,
    source_ends_with_terminator: bool,
    ends_with_terminator: bool,
}

impl<'a, W: Write> Emitter<'a, W> {
    fn new(
        writer: &'a mut W,
        eol: &'static str,
        total: usize,
        source_ends_with_terminator: bool,
    ) -> Self {
        Self {
            writer,
            eol,
            total,
            emitted: 0,
            first: true,
            previous_terminated: true,
            source_ends_with_terminator,
            ends_with_terminator: false,
        }
    }

    fn is_last(&self) -> bool {
        self.emitted + 1 == self.total
    }

    fn write_separator_if_needed(&mut self) -> io::Result<()> {
        if !self.first && !self.previous_terminated {
            self.writer.write_all(self.eol.as_bytes())?;
        }
        Ok(())
    }

    fn emit_rendered(&mut self, indent: &str, body: &str) -> io::Result<()> {
        let omit_final_terminator = self.is_last() && !self.source_ends_with_terminator;
        self.write_separator_if_needed()?;
        self.writer.write_all(indent.as_bytes())?;
        let normalized = body.replace('\n', self.eol);
        self.writer.write_all(normalized.as_bytes())?;
        if !omit_final_terminator {
            self.writer.write_all(self.eol.as_bytes())?;
        }
        let terminated = !omit_final_terminator;
        self.first = false;
        self.previous_terminated = terminated;
        self.ends_with_terminator = terminated;
        self.emitted += 1;
        Ok(())
    }

    fn emit_raw(
        &mut self,
        with_terminator: &str,
        without_terminator: &str,
        has_terminator: bool,
    ) -> io::Result<()> {
        let strip_final_terminator =
            self.is_last() && !self.source_ends_with_terminator && has_terminator;
        self.write_separator_if_needed()?;
        let text = if strip_final_terminator {
            without_terminator
        } else {
            with_terminator
        };
        self.writer.write_all(text.as_bytes())?;
        let terminated = has_terminator && !strip_final_terminator;
        self.first = false;
        self.previous_terminated = terminated;
        self.ends_with_terminator = terminated;
        self.emitted += 1;
        Ok(())
    }
}

impl<T: Write> Writer<T> {
    /// Apply `patch` to `source_text` and stream the patched document to this writer.
    ///
    /// The writer's [`WriterConfig`] is used to render new commands, so number/quote/
    /// indentation formatting matches [`Writer::write_command`]. The writer's
    /// indent/newline state is not consulted; the call emits a complete document.
    ///
    /// In [`ApplyMode::Strict`] (the default), every target, expectation and conflict
    /// is validated before the first byte is written. I/O errors during emission can
    /// still leave a partial document, as with any `Write`-based API.
    ///
    /// # Examples
    ///
    /// ```
    /// use koicore::Command;
    /// use koicore::writer::{Patch, Writer};
    /// use koicore::WriterConfig;
    ///
    /// let mut patch = Patch::new();
    /// patch.replace_line(1, vec![Command::new("background", vec!["Forest".into()])]);
    ///
    /// let mut output = Vec::new();
    /// {
    ///     let mut writer = Writer::new(&mut output, WriterConfig::default());
    ///     writer.write_patched("#background City\n", &patch).unwrap();
    /// }
    /// assert_eq!(String::from_utf8(output).unwrap(), "#background Forest\n");
    /// ```
    pub fn write_patched(
        &mut self,
        source_text: &str,
        patch: &Patch,
    ) -> Result<PatchReport, PatchError> {
        let source = SourceLines::parse(source_text);
        let plan = validate(&source, patch)?;
        let eol = source.resolved_newline(patch.options.newline);
        let total = total_elements(&source, patch, &plan);
        let append_prefix = append_indent(&source, patch);

        let mut report = PatchReport {
            lines_before: source.lines.len(),
            lines_after: 0,
            replaced: 0,
            inserted: plan.before.iter().flatten().count() + plan.after.iter().flatten().count(),
            deleted: 0,
            appended: patch.append.len(),
            warnings: plan.warnings.clone(),
        };

        let mut ended_with_terminator = false;
        {
            let mut emitter =
                Emitter::new(&mut self.writer, eol, total, source.ends_with_terminator);

            if let Some(bom) = source.bom {
                emitter.writer.write_all(bom.as_bytes())?;
            }

            for group in 0..source.groups.len() {
                for &edit_index in &plan.before[group] {
                    let indent = edit_indent(&source, patch, &plan, edit_index);
                    for command in edit_commands(&patch.edits[edit_index].edit) {
                        let body = render_command(&self.config, command)?;
                        emitter.emit_rendered(&indent, &body)?;
                        report.lines_after += 1 + body.matches('\n').count();
                    }
                }

                match plan.primary[group] {
                    Some(edit_index) => match &patch.edits[edit_index].edit {
                        Edit::Replace(commands) => {
                            let indent = edit_indent(&source, patch, &plan, edit_index);
                            report.replaced += 1;
                            for command in commands {
                                let body = render_command(&self.config, command)?;
                                emitter.emit_rendered(&indent, &body)?;
                                report.lines_after += 1 + body.matches('\n').count();
                            }
                        }
                        Edit::Delete => {
                            report.deleted += 1;
                        }
                        _ => unreachable!("emission plan only stores Replace/Delete edits"),
                    },
                    None => {
                        emitter.emit_raw(
                            source.group_raw(group),
                            source.group_raw_without_final_terminator(group),
                            source.group_ends_with_terminator(group),
                        )?;
                        report.lines_after +=
                            source.groups[group].end_line - source.groups[group].start_line;
                    }
                }

                for &edit_index in &plan.after[group] {
                    let indent = edit_indent(&source, patch, &plan, edit_index);
                    for command in edit_commands(&patch.edits[edit_index].edit) {
                        let body = render_command(&self.config, command)?;
                        emitter.emit_rendered(&indent, &body)?;
                        report.lines_after += 1 + body.matches('\n').count();
                    }
                }
            }

            for command in &patch.append {
                let body = render_command(&self.config, command)?;
                emitter.emit_rendered(&append_prefix, &body)?;
                report.lines_after += 1 + body.matches('\n').count();
            }

            ended_with_terminator = emitter.ends_with_terminator;
        }

        self.last_was_newline = ended_with_terminator;
        Ok(report)
    }
}

/// Apply a patch to a source document using the default [`WriterConfig`].
///
/// # Examples
///
/// ```
/// use koicore::Command;
/// use koicore::writer::{Patch, apply_patch};
///
/// let mut patch = Patch::new();
/// patch.replace_line(1, vec![Command::new("background", vec!["Forest".into()])]);
///
/// assert_eq!(apply_patch("#background City\n", &patch).unwrap(), "#background Forest\n");
/// ```
pub fn apply_patch(source: &str, patch: &Patch) -> Result<String, PatchError> {
    apply_patch_with_config(source, patch, &WriterConfig::default())
}

/// Apply a patch to a source document with an explicit [`WriterConfig`].
///
/// The returned `String` is fully materialized; on error nothing is returned.
/// Use [`Writer::write_patched`] for streaming output.
///
/// # Examples
///
/// ```
/// use koicore::Command;
/// use koicore::writer::{Patch, WriterConfig, apply_patch_with_config};
///
/// let mut patch = Patch::new();
/// patch.replace_line(1, vec![Command::new("background", vec!["Forest".into()])]);
///
/// let config = WriterConfig {
///     command_threshold: 2,
///     ..Default::default()
/// };
/// assert_eq!(
///     apply_patch_with_config("#background City\n", &patch, &config).unwrap(),
///     "##background Forest\n"
/// );
/// ```
pub fn apply_patch_with_config(
    source: &str,
    patch: &Patch,
    config: &WriterConfig,
) -> Result<String, PatchError> {
    let mut buffer = Vec::new();
    {
        let mut writer = Writer::new(&mut buffer, config.clone());
        writer.write_patched(source, patch)?;
    }
    String::from_utf8(buffer).map_err(|_| {
        PatchError::Io(io::Error::new(
            io::ErrorKind::InvalidData,
            "patched output is not valid UTF-8",
        ))
    })
}
```

In `src/writer/mod.rs`, extend the patch re-export with the two functions:

```rust
pub use self::patch::{
    Anchor, ApplyMode, Edit, Expected, IndentPolicy, NewlineStyle, Patch, PatchEdit, PatchError,
    PatchOptions, PatchReport, PatchWarning, apply_patch, apply_patch_with_config,
};
```

Replace the contents of `src/lib.rs` with:

```rust
#![doc = include_str!("../README.md")]

pub mod command;
pub mod parser;
pub mod writer;

pub use command::{Command, Parameter, Value};
pub use parser::{ParseError, Parser, ParserConfig};
pub use writer::{
    Anchor, ApplyMode, Edit, Expected, FormatterOptions, IndentPolicy, NewlineStyle, Patch,
    PatchEdit, PatchError, PatchOptions, PatchReport, PatchWarning, Writer, WriterConfig,
    apply_patch, apply_patch_with_config,
};
```

- [ ] **Step 4: Run the tests to verify they pass**

Run: `cargo test -p koicore --lib writer::patch`

Expected: PASS — all Task 1–6 unit tests pass.

Then run the whole crate and doc tests:

```bash
cargo test -p koicore
cargo test -p koicore --doc
```

Expected: PASS, including the two new doc examples.

- [ ] **Step 5: Commit**

```bash
git add src/writer/patch.rs src/writer/mod.rs src/lib.rs
git commit -m "feat(writer): apply format-preserving patches"
```

---

### Task 7: Integration tests (`tests/test_patch.rs`)

**Files:**
- Create: `tests/test_patch.rs`

**Interfaces:**
- Consumes: public API only — `koicore::writer::{apply_patch, ApplyMode, Expected, Patch, PatchError}`, `koicore::{Command, Parameter, Writer, WriterConfig}`, parser types.
- Produces: integration coverage for document-level round trips, CRLF/BOM preservation, lenient warnings, and streaming/string equivalence.

- [ ] **Step 1: Write the integration tests**

Create `tests/test_patch.rs` with:

```rust
use koicore::parser::{Parser, ParserConfig, StringInputSource};
use koicore::writer::{
    Anchor, ApplyMode, Expected, Patch, PatchError, apply_patch, apply_patch_with_config,
};
use koicore::{Command, Parameter, Writer, WriterConfig};

fn cmd(name: &str, params: Vec<&str>) -> Command {
    Command::new(name, params.into_iter().map(Parameter::from).collect())
}

#[test]
fn patches_existing_document_by_command_index() {
    let source = "#character Alice \"Hello\"\n\n## scene transition\n#background Forest\n#character Bob \"Hi\"\n";
    let mut patch = Patch::new();
    patch.delete_command(1);
    patch.insert_after_command(2, vec![cmd("action", vec!["walk"])]);
    patch.replace_command(
        3,
        vec![Command::new(
            "character",
            vec![Parameter::from("Bob"), Parameter::from("Hello there")],
        )],
    );

    let expected = "#character Alice \"Hello\"\n\n#background Forest\n#action walk\n#character Bob \"Hello there\"\n";
    assert_eq!(apply_patch(source, &patch).unwrap(), expected);
}

#[test]
fn untouched_lines_are_byte_identical() {
    let source = "  #a 1\r\n\r\n#b 2\r\n#c 3\r\n";
    let mut patch = Patch::new();
    patch.replace_line(3, vec![cmd("b", vec!["changed"])]);

    let output = apply_patch(source, &patch).unwrap();
    assert_eq!(output, "  #a 1\r\n\r\n#b changed\r\n#c 3\r\n");
}

#[test]
fn preserves_crlf_and_bom_end_to_end() {
    let source = "\u{feff}#a 1\r\n#b 2\r\n";
    let mut patch = Patch::new();
    patch.replace_line(2, vec![Command::new("c", vec![Parameter::from(3)])]);
    assert_eq!(apply_patch(source, &patch).unwrap(), "\u{feff}#a 1\r\n#c 3\r\n");
}

#[test]
fn stale_line_expectation_is_rejected() {
    let source = "#a\n";
    let mut patch = Patch::new();
    patch.replace_line_checked(1, Expected::Text("#old".to_string()), vec![cmd("b", vec![])]);

    let error = apply_patch(source, &patch).unwrap_err();
    assert!(matches!(
        error,
        PatchError::AnchorMismatch { lineno: 1, .. }
    ));
}

#[test]
fn lenient_mode_skips_invalid_edits_and_warns() {
    let source = "#a\n#b\n";
    let mut patch = Patch::new();
    patch.options_mut().mode = ApplyMode::Lenient;
    patch.replace_line(9, vec![cmd("x", vec![])]);
    patch.delete_line(2);

    let mut buffer = Vec::new();
    let report = {
        let mut writer = Writer::new(&mut buffer, WriterConfig::default());
        writer.write_patched(source, &patch).unwrap()
    };

    assert_eq!(String::from_utf8(buffer).unwrap(), "#a\n");
    assert_eq!(report.warnings.len(), 1);
}

#[test]
fn patched_output_parses_back_to_expected_commands() {
    let source = "#cmd a\n#cmd b\n";
    let mut patch = Patch::new();
    patch.replace_line(2, vec![Command::new("cmd", vec![Parameter::from(2)])]);
    patch.append(vec![Command::new_text("tail")]);

    let output = apply_patch(source, &patch).unwrap();

    let mut parser = Parser::new(StringInputSource::new(&output), ParserConfig::default());
    let mut commands = Vec::new();
    while let Some((command, _)) = parser.next_command_with_source().unwrap() {
        commands.push(command);
    }

    assert_eq!(
        commands,
        vec![
            Command::new("cmd", vec![Parameter::from("a")]),
            Command::new("cmd", vec![Parameter::from(2)]),
            Command::new_text("tail"),
        ]
    );
}

#[test]
fn append_to_document_without_trailing_newline_is_parseable() {
    let source = "#a";
    let mut patch = Patch::new();
    patch.append(vec![cmd("b", vec![])]);
    let output = apply_patch(source, &patch).unwrap();
    assert_eq!(output, "#a\n#b");

    let mut parser = Parser::new(StringInputSource::new(&output), ParserConfig::default());
    let mut commands = Vec::new();
    while let Some((command, _)) = parser.next_command_with_source().unwrap() {
        commands.push(command);
    }
    assert_eq!(commands, vec![cmd("a", vec![]), cmd("b", vec![])]);
}

#[test]
fn command_anchor_with_expectation_guards_drift() {
    let source = "#a\n#b\n";
    let mut patch = Patch::new();
    patch.edit(
        Anchor::Command {
            index: 1,
            expect: Some(Command::new("b", Vec::new())),
        },
        koicore::writer::Edit::Delete,
    );
    assert_eq!(apply_patch(source, &patch).unwrap(), "#a\n");
}

#[test]
fn write_patched_matches_apply_patch_with_config() {
    let source = "    #a\n\r\n#b\n";
    let mut patch = Patch::new();
    patch.insert_after_line(1, vec![cmd("nested", vec![])]);
    patch.replace_line(3, vec![cmd("b", vec!["x"])]);

    let expected = apply_patch_with_config(source, &patch, &WriterConfig::default()).unwrap();

    let mut buffer = Vec::new();
    {
        let mut writer = Writer::new(&mut buffer, WriterConfig::default());
        writer.write_patched(source, &patch).unwrap();
    }
    assert_eq!(String::from_utf8(buffer).unwrap(), expected);
}

#[test]
fn multi_command_insert_round_trips() {
    let source = "#a\n#b\n";
    let mut patch = Patch::new();
    patch.insert_after_line(
        1,
        vec![
            Command::new("cmd", vec![Parameter::from(1)]),
            Command::new_text("hi"),
            Command::new_annotation("note"),
            Command::new_number(7, Vec::new()),
        ],
    );

    let output = apply_patch(source, &patch).unwrap();

    let mut parser = Parser::new(StringInputSource::new(&output), ParserConfig::default());
    let mut commands = Vec::new();
    while let Some((command, _)) = parser.next_command_with_source().unwrap() {
        commands.push(command);
    }

    assert_eq!(
        commands,
        vec![
            cmd("a", vec![]),
            Command::new("cmd", vec![Parameter::from(1)]),
            Command::new_text("hi"),
            Command::new_annotation("note"),
            Command::new_number(7, Vec::new()),
            cmd("b", vec![]),
        ]
    );
}
```

- [ ] **Step 2: Run the tests**

Run: `cargo test -p koicore --test test_patch`

Expected: PASS — 10 integration tests pass.

If a whitespace/CRLF assertion fails, debug by comparing `{:?}` of the expected and actual strings before changing the implementation; the unit tests in Task 6 define the intended byte behavior.

- [ ] **Step 3: Run the whole workspace test suite**

Run: `cargo test`

Expected: PASS — all previous tests plus the new integration tests.

- [ ] **Step 4: Commit**

```bash
git add tests/test_patch.rs
git commit -m "test(writer): add patch integration tests"
```

---

### Task 8: README documentation and full verification

**Files:**
- Modify: `README.md`
- Modify: `README_cn.md`

**Interfaces:**
- Consumes: the behavior implemented in Tasks 1–7.
- Produces: documentation + a recorded green verification run.

- [ ] **Step 1: Add feature bullets to both READMEs**

In `README.md`, inside the `## Features` list, add a bullet after the **Streaming Parser** bullet:

```markdown
- **Document Patching**: Apply format-preserving line edits to existing KoiLang files while untouched lines stay byte-identical
```

In `README_cn.md`, inside the `## 特性` list, add the corresponding bullet after the **流式解析器** bullet:

```markdown
- **文档补丁**：对已有 KoiLang 文件做保留格式的行级修改，未涉及的行逐字节保持不变
```

- [ ] **Step 2: Run the full repository verification checklist**

```bash
cargo build --release --workspace
cargo test
cargo clippy --workspace
cargo doc --workspace --no-deps
make ffi-test
```

Expected:

- `cargo build --release --workspace` exits 0.
- `cargo test` exits 0, including `tests/test_patch.rs`, the `writer::patch` unit tests, all pre-existing tests, and doc tests.
- `cargo clippy --workspace` exits 0 with no warnings on the new code.
- `cargo doc --workspace --no-deps` exits 0.
- `make ffi-test` exits 0 (42+ FFI tests; unchanged because FFI was not modified).

If `cargo clippy` reports warnings in `src/writer/patch.rs`, fix them (do not add `#[allow(...)]`).

- [ ] **Step 3: Review the final diff**

Run:

```bash
git status
git diff --stat master...HEAD
```

Expected: only `src/writer/patch.rs` (new), `src/writer/mod.rs`, `src/lib.rs`, `tests/test_patch.rs` (new), `README.md`, and `README_cn.md` are changed; no changes under `crates/` and no version bump.

- [ ] **Step 4: Commit**

```bash
git add README.md README_cn.md
git commit -m "docs: document writer patch support"
```

---

## Spec Coverage Notes

- Functional requirements 1–8 from the spec are implemented by Tasks 1 (API), 2 (line model), 4 (anchors/expectations), 5 (conflicts/lenient), 6 (rendering/emission/entry points), and 7 (integration coverage).
- The spec's "line anchors must point at the first physical line of a logical line" rule is enforced by `TargetInsideContinuation` (Task 4).
- The spec's strict atomicity requirement is enforced because `validate` (Tasks 4–5) completes before `Emitter` is constructed in `write_patched` (Task 6); the test `strict_failure_writes_nothing` proves it.
- BOM / EOL / EOF / indentation rules are implemented in `SourceLines` (Task 2) and `Emitter` (Task 6), with tests in Task 6 and Task 7.
- FFI/Python, intra-line edits, diff format, and version bump are explicitly out of scope (spec Non-Goals).
