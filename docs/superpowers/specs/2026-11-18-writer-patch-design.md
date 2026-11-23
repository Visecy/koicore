# Writer Patch Support — Design Spec

## Status

Approved — ready for implementation planning.

## Goal

Add **format-preserving patch application** to `koicore::writer`.

Given an existing KoiLang source text and a set of whole-line edits (replace, insert, delete, append), the writer emits the patched document while leaving every untouched line **byte-for-byte unchanged** — blank lines, indentation, line endings, annotation placement, and existing quoting/number formatting are all preserved. Edits are addressed either by 1-based physical line number or by 0-based parsed command index, and both forms can be mixed in one patch.

## Background

- The current [`Writer<T: Write>`](../../../src/writer/mod.rs) is append-only: `write_command` / `write_command_with_options` serialize a `Command` to the output stream, `inc_indent` / `dec_indent` / `newline` manage layout, and the writer never revisits emitted bytes.
- Rewriting a whole document through `write_command` is lossy for tooling: blank lines are dropped by the parser (`preserve_empty_lines` defaults to `false`), original indentation, comment/annotation placement, line endings, and the source's chosen number/quote style are not preserved.
- The parser is **line oriented**: [`Parser::next_command_with_source`](../../../src/parser/mod.rs) already returns `(Command, ParserLineSource)` where `ParserLineSource` carries `lineno` and the raw line text. A logical line may span several physical lines only through the `\<newline>` line-continuation rule in [`Input::next_line`](../../../src/parser/input.rs).
- [`Command`](../../../src/command.rs) carries **no source position**, so a patch layer must map commands to source lines itself.
- There is no existing patch, diff, or write-back capability in `koicore`, `koicore_ffi`, `koilang-rs`, `koilang-py`, or the VS Code extension. This is a new feature.

## Requirements

### Functional

1. **Addressing.** An edit targets either:
   - a 1-based physical line number (`Anchor::Line`), optionally guarded by an expectation, or
   - a 0-based command index (`Anchor::Command`), resolved by parsing the source, optionally guarded by an expected `Command`.
2. **Operations.** `Replace`, `InsertBefore`, `InsertAfter`, `Delete` on a target, plus `append` at end of file. New content is always `Vec<Command>`.
3. **Rendering.** Newly written commands are rendered through the existing writer formatting pipeline (`WriterConfig` / `FormatterOptions` / `Generators`), so their text matches what `write_command` would emit.
4. **Preservation.** Untouched lines are copied verbatim. Source line-ending style, UTF-8 BOM, and the "ends with newline / no trailing newline" convention are preserved.
5. **Indentation.** Newly written lines inherit the target line's leading whitespace by default; policy is configurable (`InheritAnchor` / `None` / `Fixed`).
6. **Safety.** Default `Strict` mode validates every target, expectation and conflict before writing a single byte; semantic failure means "no output". `Lenient` mode skips invalid/conflicting edits and reports warnings.
7. **Report.** Application returns a `PatchReport` with line counts, applied-edit counts, and warnings.
8. **Two entry points.** Streaming (write patched document into `Writer<T>`) and convenience string-returning helpers.

### Non-functional

- Rust 2024 edition, no `unsafe`, no new dependencies.
- Existing `write_command` behavior must not change.
- Complexity `O(n + e log e)` in source bytes `n` and edits `e`; source and edit plan are held in memory.
- All public items documented with doc comments and rustdoc examples.
- No FFI or Python binding changes in this change.
- Repository verification checklist applies: `cargo build --release --workspace`, `cargo test`, `cargo clippy --workspace`, `cargo doc --workspace --no-deps`, `make ffi-test`.

## Non-Goals

- Intra-line edits (changing one parameter of a line while keeping the rest of the line verbatim). Callers construct a new `Command` and replace the whole logical line instead.
- Unified-diff / diff-generation format, and applying external patch files.
- FFI C API and Python (`koilang-py`) exposure.
- Encoding conversion. The API takes `&str`; callers decode first (e.g. via `FileInputSource` / `DecodeBufReader`).
- Streaming input sources. Patch application needs the full source to validate anchors before writing.
- A persistent `Document` model for accumulating many edits across time (possible future convenience layer, see [Alternatives](#alternatives-considered)).

## Design

### Module layout

| File | Action | Responsibility |
|---|---|---|
| `src/writer/patch.rs` | **Create** | Line/logical-line splitting, anchor resolution, validation, conflict detection, emission, `Patch*` types, `Writer::write_patched`, `apply_patch*` helpers, unit tests. |
| `src/writer/mod.rs` | Modify | `mod patch;` + re-exports. Existing `write_command` behavior unchanged. |
| `src/lib.rs` | Modify | Re-export `Patch`, `Anchor`, `Edit`, `PatchError`, `apply_patch` at crate root. |
| `tests/test_patch.rs` | **Create** | Integration tests: write → patch → re-parse round trips, CRLF/BOM/EOF preservation, command anchors, strict/lenient behavior. |
| `README.md`, `README_cn.md` | Modify | Add a "Patch existing documents" feature bullet. |

No version bump in this change; releases are cut by a separate release commit, consistent with repository history.

### Concepts

#### Physical lines and logical line groups

KoiLang is line oriented. A **physical line** ends with `\n` or `\r\n`; the final physical line may have no terminator. Consecutive physical lines form one **logical line** when the previous physical line ends with `\<newline>` (the existing line-continuation rule).

- `Command` anchors resolve to the logical line that produced the command.
- `Line` anchors must point at the **first physical line of a logical line**. Pointing into a continuation line is an error (`TargetInsideContinuation`), because a partial rewrite would corrupt the command.
- `Replace` and `Delete` operate on the whole logical line; `InsertBefore` inserts before the logical line; `InsertAfter` inserts after the last physical line of the logical line.
- A physical line that does not end with `\<newline>` is a logical line of length 1, including blank lines.

Line numbers in errors and the report are always physical, 1-based.

#### Empty source

`""` has zero physical lines. Only `append` is valid; every anchored edit is out of range.

#### BOM

If the source starts with `U+FEFF`, the BOM is stripped before line splitting and re-emitted at the very start of the output (even when the original first line is replaced or deleted). The BOM is preserved even if the resulting document is empty.

#### Line endings

- Untouched lines are copied with their original terminators, so mixed line endings inside the source are preserved.
- Newly rendered lines use: `NewlineStyle::Auto` (detect the first line terminator in the source: `\r\n` if found first, else `\n`; default `\n` when the source has no terminator) — or the explicit `NewlineStyle::Lf` / `NewlineStyle::Crlf`.
- A lone `\r` not followed by `\n` is treated as ordinary content (classic-Mac line endings are unsupported).

#### Trailing newline convention

The output ends with a line terminator **iff** the source did. When appending content to a source whose last line has no terminator, the last original line gains a separator terminator (unavoidable), and the final appended line is emitted without a terminator so the "no trailing newline" convention is kept.

### Public API

```rust
use crate::command::Command;
use crate::parser::{ParserConfig, ParseError};

/// How an edit locates its target.
#[derive(Debug, Clone, PartialEq)]
pub enum Anchor {
    /// 1-based physical line number. Must be the first line of a logical line.
    /// `expect` guards against the source having changed since it was inspected.
    Line { lineno: usize, expect: Option<Expected> },
    /// 0-based index into the parsed command stream
    /// (order of `Parser::next_command_with_source` results).
    /// Requires the source to be fully parseable when applied.
    Command { index: usize, expect: Option<Command> },
}

/// Optional integrity check for an anchor.
#[derive(Debug, Clone, PartialEq)]
pub enum Expected {
    /// Raw text of the targeted physical line (which must be a logical-line start),
    /// without its terminator. A single trailing `\r\n` / `\n` in the expectation
    /// is normalized away for convenience.
    Text(String),
    /// The targeted logical line must parse to exactly this command.
    Command(Command),
}

/// Whole-line edit operations.
#[derive(Debug, Clone, PartialEq)]
pub enum Edit {
    /// Replace the targeted logical line with these commands (must be non-empty).
    Replace(Vec<Command>),
    /// Insert before the targeted logical line.
    InsertBefore(Vec<Command>),
    /// Insert after the targeted logical line.
    InsertAfter(Vec<Command>),
    /// Delete the targeted logical line.
    Delete,
}

/// One edit in a patch.
#[derive(Debug, Clone, PartialEq)]
pub struct PatchEdit {
    pub anchor: Anchor,
    pub edit: Edit,
}

/// A format-preserving patch for a KoiLang source document.
#[derive(Debug, Clone, PartialEq)]
pub struct Patch { /* edits: Vec<PatchEdit>, append: Vec<Command>, options: PatchOptions */ }

impl Patch {
    pub fn new() -> Self;
    pub fn is_empty(&self) -> bool;
    pub fn edits(&self) -> &[PatchEdit];
    pub fn options(&self) -> &PatchOptions;
    pub fn options_mut(&mut self) -> &mut PatchOptions;
    pub fn with_options(self, options: PatchOptions) -> Self;

    /// Generic insertion point; all convenience methods below delegate here.
    pub fn edit(&mut self, anchor: Anchor, edit: Edit) -> &mut Self;
    /// Append commands after the last logical line of the source.
    /// Repeated calls accumulate in call order.
    pub fn append(&mut self, commands: Vec<Command>) -> &mut Self;

    pub fn replace_line(&mut self, lineno: usize, commands: Vec<Command>) -> &mut Self;
    pub fn replace_line_checked(
        &mut self,
        lineno: usize,
        expect: Expected,
        commands: Vec<Command>,
    ) -> &mut Self;
    pub fn insert_before_line(&mut self, lineno: usize, commands: Vec<Command>) -> &mut Self;
    pub fn insert_after_line(&mut self, lineno: usize, commands: Vec<Command>) -> &mut Self;
    pub fn delete_line(&mut self, lineno: usize) -> &mut Self;
    pub fn replace_command(&mut self, index: usize, commands: Vec<Command>) -> &mut Self;
    pub fn insert_before_command(&mut self, index: usize, commands: Vec<Command>) -> &mut Self;
    pub fn insert_after_command(&mut self, index: usize, commands: Vec<Command>) -> &mut Self;
    pub fn delete_command(&mut self, index: usize) -> &mut Self;
}

impl Default for Patch { /* same as Patch::new() */ }

/// Conflict policy and format-preservation knobs.
#[derive(Debug, Clone, PartialEq)]
pub struct PatchOptions {
    /// `Strict` (default) validates everything before writing; `Lenient` skips
    /// invalid or conflicting edits and records warnings.
    pub mode: ApplyMode,
    /// Indentation policy for newly written lines.
    pub indent: IndentPolicy,
    /// Line-ending style for newly written lines.
    pub newline: NewlineStyle,
    /// Parser configuration used to resolve `Anchor::Command` and to parse
    /// `Expected::Command`. Must match the configuration the source was read with.
    pub parser_config: ParserConfig,
}

impl Default for PatchOptions { /* Strict, InheritAnchor, Auto, ParserConfig::default() */ }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ApplyMode { #[default] Strict, Lenient }

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum IndentPolicy {
    /// Copy the leading whitespace of the target logical line (default).
    /// For `append`, inherit the leading whitespace of the last logical line
    /// (empty when the source is empty).
    #[default]
    InheritAnchor,
    /// Emit new lines with no leading whitespace.
    None,
    /// Emit new lines with exactly this prefix (spaces and/or tabs).
    Fixed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NewlineStyle { #[default] Auto, Lf, Crlf }

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
    /// Edits skipped in `Lenient` mode.
    pub warnings: Vec<PatchWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchWarning {
    /// Target line/command index was out of range; edit skipped.
    OutOfRange { edit_index: usize },
    /// Anchor expectation did not match; edit skipped.
    AnchorMismatch { edit_index: usize },
    /// Another edit already owns this logical line; edit skipped.
    Conflict { edit_index: usize, owner_edit_index: usize },
    /// Edit payload was empty; edit skipped.
    EmptyContent { edit_index: usize },
    /// Line anchor points into a continuation line; edit skipped.
    InsideContinuation { edit_index: usize },
}

/// Errors returned by patch application.
#[derive(Debug)]
pub enum PatchError {
    Io(std::io::Error),
    LineOutOfRange { lineno: usize, total_lines: usize },
    CommandIndexOutOfRange { index: usize, total_commands: usize },
    TargetInsideContinuation { lineno: usize, start_lineno: usize },
    SourceParse(Box<ParseError>),
    AnchorMismatch { lineno: usize, expected: Expected, actual: String },
    CommandMismatch { index: usize, lineno: usize, expected: Command, actual: Command },
    ConflictingEdits { lineno: usize, first_edit: usize, second_edit: usize },
    EmptyContent { edit_index: usize, operation: &'static str },
}

impl std::fmt::Display for PatchError { /* ... */ }
impl std::error::Error for PatchError { /* source() */ }
impl From<std::io::Error> for PatchError { /* ... */ }

impl<T: std::io::Write> crate::writer::Writer<T> {
    /// Apply `patch` to `source` and stream the patched document to this writer.
    ///
    /// The writer's `WriterConfig` is used to render new commands. The writer's
    /// indent/newline state is not consulted; the call emits a complete document.
    pub fn write_patched(&mut self, source: &str, patch: &Patch)
        -> Result<PatchReport, PatchError>;
}

/// Apply a patch with the default `WriterConfig`.
pub fn apply_patch(source: &str, patch: &Patch) -> Result<String, PatchError>;

/// Apply a patch with an explicit `WriterConfig`.
pub fn apply_patch_with_config(
    source: &str,
    patch: &Patch,
    config: &WriterConfig,
) -> Result<String, PatchError>;
```

### Applying a patch

An empty patch (no edits, no append) returns the source unchanged, including its BOM, with a zero-count `PatchReport`.

#### Phase 1 — parse source shape

1. Strip and remember BOM; split source into physical lines preserving each terminator; build logical-line groups (start line, end line, raw bytes) using the `\<newline>` continuation rule.
2. If the patch contains any `Anchor::Command` or `Expected::Command`, parse the source with `PatchOptions::parser_config`:
   - `Anchor::Command` resolution requires a full parse (`CommandIndexOutOfRange` / `SourceParse` on failure). Record each command's start `lineno` and its logical-line span.
   - `Expected::Command` on a line anchor only parses the targeted logical line.

#### Phase 2 — validate (strict mode)

Checks run in a deterministic order, and the first failure is returned:

1. Payload validation: `Replace` / `Insert*` with an empty `Vec` → `EmptyContent`; `Delete` has no payload.
2. Target resolution: line number in `1..=line_count`; command index `< command_count`; line anchor must be a logical-line start (`TargetInsideContinuation`); command anchors resolve to their logical line.
3. Expectation checks: `Expected::Text` compares the targeted physical line's text (terminator stripped, no trimming of whitespace); `Expected::Command` compares full `Command` equality with the parsed command; `Anchor::Command` `expect` compares full equality.
4. Conflict detection: at most one `Replace`/`Delete` per logical line (`ConflictingEdits`). Multiple `InsertBefore` / `InsertAfter` on the same line are allowed and ordered by their position in `Patch::edits`.
5. Produce an immutable emission plan keyed by logical-line start; no bytes are written until this phase succeeds.

In `Lenient` mode the same checks are performed, but failures mark the edit skipped (`PatchWarning`) instead of returning an error; conflict resolution is "first edit in `Patch::edits` order wins". A `SourceParse` failure when command anchors are present is always fatal (nothing can be resolved).

#### Phase 3 — emit

```
emit BOM (if any)
for each logical line L in source order:
    emit InsertBefore edits for L      (Patch order)
    match emission plan for L:
        Replace(cmds) -> emit cmds      (indent policy, file EOL)
        Delete        -> emit nothing
        none          -> copy L's raw bytes verbatim
    emit InsertAfter edits for L       (Patch order)
emit append commands                    (IndentPolicy::InheritAnchor -> last line's indent)
```

Separator/terminator rules:

- Untouched logical lines are emitted as raw bytes (terminator included as in the source).
- All commands produced by one edit share that edit's resolved indentation prefix.
- Rendered lines are emitted as `indent + command_text + eol` except for the very last emitted line, which omits the final `eol` if the source did not end with a terminator.
- If the last original line has no terminator but something is emitted after it, the original line gets a separator `eol` before the next element.

#### Rendering new commands

A crate-private helper in `patch.rs` renders a command for patch emission. The existing `write_command` path is untouched:

```rust
/// Render one command as patch line content: no leading indentation,
/// no trailing newline. Patch mode disables line-breaking formatter
/// options so one command always becomes one (logical) line.
pub(crate) fn render_command(config: &WriterConfig, command: &Command) -> std::io::Result<String>;
```

- Resolve effective options with `Generators::get_effective_options` (the writer's `WriterConfig`, unchanged).
- Clone the effective options and force `newline_before`, `newline_after`, `newline_before_param`, `newline_after_param` to `false`. Rationale: those options emit blank lines / multi-line parameter layouts that are not valid round-trip KoiLang (the parser only joins lines via `\<newline>`), and a patch must stay minimal and parseable.
- Render the command body through `Generators::write_command_with_param_options` with `current_indent = 0` into a `Vec<u8>`, then convert to `String`. `@text` / `@annotation` newline escaping is inherited from the existing generator, so raw text with embedded newlines still serializes as one logical line.
- The caller prefixes the indent and appends the file EOL. For `@text`/`@annotation`, the escaped continuation lines are intrinsic to the content and receive no extra prefix (only the leading indent is added before the first line). Replacing `\n` with the detected EOL inside the body is safe because the parser's `FileInputSource`/`BufReadWrapper` normalize `\r\n` to `\n` before the continuation/unescape logic runs.

`Generators::get_effective_options` and `write_command_with_param_options` are private to the `writer` module; `patch` is a child module, so no visibility change is needed. `Writer::write_command_with_options` is not modified; existing behavior is covered by the current tests.

#### Writer integration

`Writer::write_patched` uses the writer's `config` for rendering and writes directly to the inner output:

- It does not consult `current_indent`; indentation comes from `PatchOptions::indent`.
- After a successful call it updates `last_was_newline` to reflect whether the emitted document ends with a terminator, so a subsequent `newline()` / `write_command()` does not produce a double blank line.
- Semantic validation failures emit no bytes. I/O failures during emission may leave a partial document, as with any `Write`-based API; this is documented.

String helpers build `Writer<Vec<u8>>` internally and convert to `String` (rendering only produces valid UTF-8). They are fully atomic: on error no `String` is returned.

### Conflict and ordering rules

| Situation | Result |
|---|---|
| Two `Replace` on the same logical line | `ConflictingEdits` (lenient: first wins) |
| `Replace` + `Delete` on the same logical line | `ConflictingEdits` |
| `Replace` + `InsertBefore` / `InsertAfter` | Allowed: insert before/after the replaced content |
| `Delete` + `InsertBefore` / `InsertAfter` | Allowed: inserts remain around the removed line |
| Multiple `InsertBefore` on same line | Allowed; emitted in `Patch::edits` order |
| Multiple `InsertAfter` on same line | Allowed; emitted in `Patch::edits` order |
| `InsertAfter(L)` and `InsertBefore(L+1)` | `InsertAfter(L)` first, then `InsertBefore(L+1)` |
| `append` + `InsertAfter(last line)` | `InsertAfter` first, then appended commands |
| Line anchor and command anchor resolving to the same logical line | Treated as the same target for conflict detection |

### Edge cases

| Case | Behavior |
|---|---|
| Empty source + `append` | Emits commands, no trailing terminator (source had none) |
| Source without trailing newline, replacing/deleting last line | No trailing terminator in output |
| Source with trailing newline, deleting all lines | Context preserved: untouched lines keep their bytes; if the document becomes empty the output is empty (or BOM only) |
| CRLF source | Untouched lines keep `\r\n`; rendered lines use `\r\n` under `Auto` |
| Mixed LF/CRLF source | Untouched lines keep their own terminators; rendered lines use `Auto` detection result |
| UTF-8 BOM | Preserved at output start |
| Line anchor on a blank line | Replacing a blank line is valid; inherited indent is the empty string |
| Line anchor into a continuation line | `TargetInsideContinuation` |
| Replacing a text line (`@text`) with commands or vice versa | Rendered through the writer pipeline; indent policy applies |
| `@text` / `@annotation` content with embedded newlines | Serialized with the existing `\<newline>` escape as one logical line |
| Command index with skipped blank lines / annotations | Indices follow `PatchOptions::parser_config` exactly; document that indices are parser-output order |
| Source unparseable but only line anchors used | Allowed — line anchors never parse the source (`Expected::Command` parses only the targeted logical line) |
| Writer `command_threshold` differs from the source's | Caller responsibility; documented on `PatchOptions::parser_config` and `Writer::write_patched` |

### Error handling summary

- `Strict` (default): first validation failure returns `PatchError`; no bytes written.
- `Lenient`: per-edit failures become `PatchWarning`s; parse failures when command anchors are present remain fatal.
- Rendering itself cannot fail (writes go to an in-memory buffer).
- `Writer::write_patched` maps inner I/O errors to `PatchError::Io`.

## Testing Strategy

### Unit tests (`src/writer/patch.rs`, `#[cfg(test)] mod tests`)

- Physical/logical line splitting: empty source, trailing newline, no trailing newline, CRLF, mixed endings, continuation groups.
- BOM detection and re-emission.
- Anchor resolution and every `PatchError` variant (out of range, continuation target, expectation mismatch, conflict, empty payload).
- `Expected::Text` normalization (`\r\n` / `\n` trailing), whitespace sensitivity.
- `Expected::Command` single-line and continuation-group parsing.
- Ordering rules from the conflict table; lenient warnings.
- Render helper: normal command, `@text` with embedded newline, `@annotation`, number command, quote/number formatting options honored, line-breaking options ignored.
- EOF/BOM/EOL emission rules.
- Strict atomicity: failed validation writes zero bytes.
- Existing `Writer` tests remain untouched and passing (no behavior change).

### Integration tests (`tests/test_patch.rs`)

- Round trip: source → patch → output → `Parser` re-parse; assert the command sequence matches the expected patched command stream.
- Untouched-region byte equality: diff output against input around the edits.
- Command-index patch (`replace_command`, `insert_after_command`) against a parsed source.
- CRLF + BOM end-to-end preservation.
- `Lenient` skip-and-warn scenario.
- `Writer::write_patched` writes exactly the same bytes as `apply_patch_with_config`.
- Multi-command insert (regular + `@text` + `@annotation` + `#123`) and re-parse.

### Verification

Run the repository checklist from `AGENTS.md`:

```
cargo build --release --workspace
cargo test
cargo clippy --workspace
cargo doc --workspace --no-deps
make ffi-test
```

## Implementation Outline

1. **Skeleton + line model** — create `src/writer/patch.rs`, `mod patch;` in `writer/mod.rs`, `Patch`/`Anchor`/`Expected`/`Edit`/`PatchEdit`/`PatchOptions`/`PatchError`/`PatchReport`/`PatchWarning`, physical-line and logical-line splitting with BOM, unit tests.
2. **Render helper** — add `render_command` in `patch.rs`; verify existing writer tests are unchanged and passing.
3. **Anchor resolution + validation** — physical/command anchors, expectations, continuation rules, deterministic error ordering, strict atomicity (no output).
4. **Emission plan + conflicts + ordering** — conflict table, multi-insert ordering, lenient warnings.
5. **Emission** — indent policy, EOL detection/override, BOM, EOF convention, `append`, report counters.
6. **Public entry points** — `Writer::write_patched`, `apply_patch`, `apply_patch_with_config`, crate-root re-exports, rustdoc examples.
7. **Integration tests** — `tests/test_patch.rs` per the test strategy.
8. **Docs** — module-level documentation, README / README_cn feature bullets.
9. **Verification** — full checklist above.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| `command_threshold` / `parser_config` mismatch between source and writer config renders `#` prefixes the parser would not read as commands | Documented requirement; tests cover matching configs; future `verify` re-parse option listed below |
| Command indices drift when the source changes between inspection and patch application | Optional `expect` guards on both anchor kinds; strict mode errors on mismatch |
| Continuation lines (`@text` embedded newlines) corrupted by naive indentation | Continuation-aware logical-line model; continuation lines never get extra prefix or partial edits |
| Line-breaking formatter options producing unparseable multi-line commands | Patch rendering forces those options off (one logical line per command) |
| Partial output on I/O failure in streaming mode | Documented; string helpers are atomic and recommended for tools |
| Mixed line endings / no trailing newline mishandled | Explicit EOL/EOF rules and dedicated tests |

## Alternatives Considered

1. **`Document` model** (`Document::parse()`, `doc.replace_command(i, cmd)`, `doc.write_to(writer)`): ergonomic for long-lived editors, but requires a parseable source, needs index/line-drift bookkeeping, and duplicates most of the patch machinery. Deferred as a possible future convenience layer on top of `Patch`.
2. **LSP-style `TextEdit { range, new_text }`**: maximally flexible (including intra-line edits), but callers must compute byte/line ranges themselves and can easily produce invalid KoiLang; it also does not serialize `Command` values. Rejected for this iteration since whole-line edits were chosen.
3. **Unified-diff generation/application**: valuable for review and distribution, but orthogonal to format-preserving document editing and much larger in scope. Out of scope.

## Future Work (Out of Scope Now)

- Optional `verify` mode that re-parses the patched output and reports the first parse error (guards config/threshold mismatches).
- `koicore_ffi` and `koilang-py` bindings for patch application.
- `Document` model for accumulating edits (see Alternatives).
- Diff generation (`unified diff`) and external patch files.
- Streaming input support with two-pass anchor validation.
