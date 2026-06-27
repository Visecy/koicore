# @text / @annotation Newline Escape Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the broken round-trip for `@text` and `@annotation` commands whose content contains embedded newline characters by adding escape/unescape helpers in the writer and parser.

**Architecture:** The writer escapes each `\n` as `\<newline>` (backslash + newline) in `@text`/`@annotation` content, which the existing `Input::next_line` line-continuation mechanism reassembles into a single logical line. The parser reverses this by stripping the backslash immediately preceding each newline. This is a minimal, non-C-style scheme — backslashes not followed by newlines are left untouched.

**Tech Stack:** Rust 2024 Edition, koicore crate, existing `str::replace` semantics.

## Global Constraints

- Rust Edition 2024 — use 2024 edition syntax (e.g., `let ... && let ...` chains are allowed).
- All public items need doc comments (per AGENTS.md).
- FFI memory ownership rules apply to public API surface — no new public FFI functions are introduced here.
- Verification checklist (per AGENTS.md): `cargo build --release --workspace`, `cargo test`, `cargo clippy --workspace`, `cargo doc --workspace --no-deps`, `make ffi-test`.
- The escape scheme is deliberately minimal: only `\n` → `\<newline>` (writer) and `\<newline>` → `\n` (parser). Backslash itself is NOT escaped when not followed by newline.
- Parser MUST unescape BEFORE trim to avoid leaving dangling backslashes.

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `src/writer/generators.rs` | Modify | Add `escape_text_newlines` helper; call it in `@text` and `@annotation` branches of `write_command_with_param_options`; add unit tests in existing `#[cfg(test)] mod tests` block. |
| `src/parser/mod.rs` | Modify | Add `unescape_text_newlines` helper; call it in text/annotation branches of `next_command_with_source` with correct ordering (unescape BEFORE trim); add unit tests in existing `#[cfg(test)] mod tests` block. |
| `tests/test_writer.rs` | Modify | Add round-trip integration tests covering mid-content newlines, trailing newlines, backslash+newline combinations, and a `preserve_indent` variant. |

No new files are created. All changes are in existing files.

---

### Task 1: Writer — `escape_text_newlines` + `@text`/`@annotation` integration

**Files:**
- Modify: `src/writer/generators.rs:1-15` (add helper after imports)
- Modify: `src/writer/generators.rs:46-65` (apply escape in `@text`/`@annotation` match arms)
- Test: `src/writer/generators.rs:368-790` (add unit tests in existing `#[cfg(test)] mod tests`)

**Interfaces:**
- Produces: `pub(crate) fn escape_text_newlines(s: &str) -> String` — takes raw text content, returns text with each `\n` replaced by `\<newline>` (backslash + newline).

- [ ] **Step 1: Write the failing unit test**

Add the following test to the `#[cfg(test)] mod tests` block in `src/writer/generators.rs`, after the existing `test_write_command_with_param_options` test (after line 726):

```rust
#[test]
fn test_escape_text_newlines() {
    // No newlines - unchanged
    assert_eq!(escape_text_newlines("Hello"), "Hello");

    // Single newline in middle: \n becomes \<newline>
    assert_eq!(escape_text_newlines("Hello\nWorld"), "Hello\\\nWorld");

    // Trailing newline: \n becomes \<newline>
    assert_eq!(escape_text_newlines("Hello\n"), "Hello\\\n");

    // Leading newline: \n becomes \<newline>
    assert_eq!(escape_text_newlines("\nHello"), "\\\nHello");

    // Multiple newlines: each \n becomes \<newline>
    assert_eq!(escape_text_newlines("a\nb\nc"), "a\\\nb\\\nc");

    // Existing backslash + newline: backslash preserved, newline gets
    // another backslash prepended → two backslashes + newline
    assert_eq!(escape_text_newlines("Hello\\\nWorld"), "Hello\\\\\nWorld");

    // Two existing backslashes + newline: preserved + one more → three
    // backslashes + newline
    assert_eq!(escape_text_newlines("Hello\\\\\nWorld"), "Hello\\\\\\\nWorld");

    // Empty string - unchanged
    assert_eq!(escape_text_newlines(""), "");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib -p koicore generators::tests::test_escape_text_newlines`
Expected: FAIL with "cannot find function `escape_text_newlines`" (function not yet defined).

- [ ] **Step 3: Implement `escape_text_newlines`**

Add the following function to `src/writer/generators.rs`, after the `use` statements (after line 12, before the `Generators` struct at line 15):

```rust
/// Escape newlines in raw text/annotation content by prepending a backslash.
///
/// Each `\n` in the input becomes `\<newline>` (backslash + newline). This
/// allows text with embedded newlines to be written as a single logical line
/// that `Input::next_line` reassembles via its existing line-continuation rule.
///
/// Backslash characters themselves are NOT escaped — the parser strips only
/// the backslash immediately preceding a newline, leaving all others intact.
pub(crate) fn escape_text_newlines(s: &str) -> String {
    s.replace('\n', "\\\n")
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib -p koicore generators::tests::test_escape_text_newlines`
Expected: PASS.

- [ ] **Step 5: Apply escape to `@text` and `@annotation` branches**

Modify `src/writer/generators.rs` in `write_command_with_param_options`. Replace the current `@text` and `@annotation` match arms (lines 46-65) with:

```rust
            "@text" => {
                // Raw text command — escape newlines so the content stays on
                // one logical line. The parser's unescape_text_newlines
                // reverses this. Unlike regular params, @text content is not
                // a quoted string, so format_string does not apply here.
                if let Some(Parameter::Basic(Value::String(text))) = command.params.first() {
                    write!(writer, "{}", escape_text_newlines(text))?;
                }
            }
            "@annotation" => {
                // Raw annotation command — same newline-escape rule as @text.
                // The leading-# check uses the original text (escaping does not
                // affect `#` characters).
                if let Some(Parameter::Basic(Value::String(text))) = command.params.first() {
                    let hashes = "#".repeat(config.command_threshold + 1);
                    let escaped = escape_text_newlines(text);
                    if text.trim_start().starts_with(&hashes) {
                        // If text already has enough #, just write it
                        write!(writer, "{}", escaped)?;
                    } else {
                        // Otherwise, add extra #
                        write!(writer, "{} {}", hashes, escaped)?;
                    }
                }
            }
```

- [ ] **Step 6: Run writer tests to verify existing tests still pass**

Run: `cargo test --lib -p koicore generators::tests`
Expected: PASS — all existing generator tests pass (the `escape_text_newlines` test and all pre-existing tests). The existing `test_write_command_with_param_options` test passes because `"Hello, world!"` and `"This is an annotation"` contain no newlines, so escaping is a no-op.

- [ ] **Step 7: Commit**

```bash
git add src/writer/generators.rs
git commit -m "feat(writer): escape newlines in @text/@annotation content

Add escape_text_newlines helper that prepends a backslash before each
newline in raw text/annotation content. This prevents Input::next_line
from splitting a single @text/@annotation command into multiple commands
when the content contains embedded newlines."
```

---

### Task 2: Parser — `unescape_text_newlines` + text/annotation integration

**Files:**
- Modify: `src/parser/mod.rs:40-42` (add helper after imports)
- Modify: `src/parser/mod.rs:324-341` (apply unescape in text/annotation branches)
- Test: `src/parser/mod.rs:519-779` (add unit tests in existing `#[cfg(test)] mod tests`)

**Interfaces:**
- Consumes: writer's `escape_text_newlines` output (via the text that `Input::next_line` reassembles).
- Produces: `fn unescape_text_newlines(s: &str) -> String` (private) — takes the accumulated `line_text` from `Input::next_line`, reverses the writer's escaping by stripping one backslash before each newline.

- [ ] **Step 1: Write the failing unit test**

Add the following test to the `#[cfg(test)] mod tests` block in `src/parser/mod.rs`, after the existing `test_next_command_with_source_skip_annotations` test (after line 779):

```rust
#[test]
fn test_unescape_text_newlines() {
    // No escape sequences - unchanged
    assert_eq!(unescape_text_newlines("Hello"), "Hello");

    // Backslash + newline → newline
    assert_eq!(unescape_text_newlines("Hello\\\nWorld"), "Hello\nWorld");

    // Trailing backslash + newline → trailing newline
    assert_eq!(unescape_text_newlines("Hello\\\n"), "Hello\n");

    // Leading backslash + newline → leading newline
    assert_eq!(unescape_text_newlines("\\\nHello"), "\nHello");

    // Multiple escape sequences
    assert_eq!(unescape_text_newlines("a\\\nb\\\nc"), "a\nb\nc");

    // Two backslashes + newline → one backslash + newline
    // (only the backslash immediately preceding the newline is stripped)
    assert_eq!(unescape_text_newlines("Hello\\\\\nWorld"), "Hello\\\nWorld");

    // Three backslashes + newline → two backslashes + newline
    assert_eq!(unescape_text_newlines("Hello\\\\\\\nWorld"), "Hello\\\\\nWorld");

    // Plain newline (no preceding backslash) - unchanged
    assert_eq!(unescape_text_newlines("Hello\nWorld"), "Hello\nWorld");

    // Empty string - unchanged
    assert_eq!(unescape_text_newlines(""), "");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib -p koicore parser::tests::test_unescape_text_newlines`
Expected: FAIL with "cannot find function `unescape_text_newlines`" (function not yet defined).

- [ ] **Step 3: Implement `unescape_text_newlines`**

Add the following function to `src/parser/mod.rs`, after the `use` statements (after line 41, before the `ParserConfig` struct at line 43):

```rust
/// Reverse the writer's newline escaping for `@text` / `@annotation` content.
///
/// Each `\<newline>` (backslash + newline) becomes `<newline>` (just newline).
/// Only the backslash immediately preceding a newline is stripped; all other
/// backslashes are preserved verbatim.
///
/// This must be called BEFORE `trim_end` / `trim` on the accumulated `line_text`
/// to avoid leaving a dangling backslash when the content ends with a newline.
fn unescape_text_newlines(s: &str) -> String {
    s.replace("\\\n", "\n")
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib -p koicore parser::tests::test_unescape_text_newlines`
Expected: PASS.

- [ ] **Step 5: Apply unescape to text/annotation branches**

Modify `src/parser/mod.rs` in `next_command_with_source`. Replace the current text and annotation branches (lines 324-341) with:

```rust
            if hash_count < self.config.command_threshold {
                // Unescape newlines BEFORE trim to avoid dangling backslashes.
                // The writer's escape_text_newlines prepended a \ before each
                // newline in the original content; this reverses that.
                let unescaped = unescape_text_newlines(&line_text);
                let text_content = if self.config.preserve_indent {
                    unescaped.trim_end().to_string()
                } else {
                    unescaped.trim().to_string()
                };
                break Ok(Some((Command::new_text(text_content), source)));
            } else if hash_count > self.config.command_threshold {
                if self.config.skip_annotations {
                    continue;
                }
                // hash_count from pre-unescape trimmed is still valid: `#` is
                // unaffected by newline unescaping.
                let unescaped = unescape_text_newlines(&line_text);
                let annotation_content = if self.config.preserve_indent {
                    unescaped.trim_end().to_string()
                } else {
                    let content: String = unescaped.trim().chars().skip(hash_count).collect();
                    content.trim().to_string()
                };
                break Ok(Some((Command::new_annotation(annotation_content), source)));
            } else {
```

- [ ] **Step 6: Run parser tests to verify existing tests still pass**

Run: `cargo test --lib -p koicore parser::tests`
Expected: PASS — all existing parser tests pass, plus the new `test_unescape_text_newlines`. The existing tests pass because:
- `test_preserve_indent` / `test_preserve_empty_lines`: inputs contain no `\<newline>` sequences, so unescape is a no-op.
- `test_next_command_with_source_*`: inputs contain no `\<newline>` sequences.
- `test_preserve_indent_annotation`: input `"##  annotation text"` has no `\<newline>`, unescape is a no-op.

- [ ] **Step 7: Commit**

```bash
git add src/parser/mod.rs
git commit -m "feat(parser): unescape newlines in @text/@annotation content

Add unescape_text_newlines helper that strips the backslash immediately
preceding each newline in accumulated line_text. Applied to text and
annotation branches with correct ordering (unescape BEFORE trim) to
reverse the writer's escape_text_newlines and complete the round-trip."
```

---

### Task 3: Round-trip integration tests

**Files:**
- Modify: `tests/test_writer.rs` (append new test functions at end of file)

**Interfaces:**
- Consumes: writer's `escape_text_newlines` (via `Writer::write_command`) and parser's `unescape_text_newlines` (via `Parser::next_command`).
- Produces: round-trip verification tests.

- [ ] **Step 1: Write the round-trip test helper and test cases**

Append the following tests to the end of `tests/test_writer.rs`:

```rust
// ============================================================================
// Round-trip tests for @text / @annotation newline escaping
// ============================================================================

/// Helper: write a text command, parse it back, return the parsed command.
fn round_trip_text(content: &str, preserve_indent: bool) -> Command {
    let original = Command::new_text(content.to_string());
    let writer_config = WriterConfig::default();
    let mut output = Vec::new();
    let mut writer = Writer::new(&mut output, writer_config);
    writer
        .write_command(&original)
        .expect("Failed to write @text command");
    let generated = String::from_utf8(output).unwrap();

    let parser_config = ParserConfig::default().with_preserve_indent(preserve_indent);
    let input = StringInputSource::new(generated.as_str());
    let mut parser = Parser::new(input, parser_config);

    parser
        .next_command()
        .expect("Failed to parse generated @text")
        .expect("No command parsed from generated @text output")
}

/// Helper: write an annotation command, parse it back, return the parsed command.
fn round_trip_annotation(content: &str) -> Command {
    let original = Command::new_annotation(content.to_string());
    let writer_config = WriterConfig::default();
    let mut output = Vec::new();
    let mut writer = Writer::new(&mut output, writer_config);
    writer
        .write_command(&original)
        .expect("Failed to write @annotation command");
    let generated = String::from_utf8(output).unwrap();

    let parser_config = ParserConfig::default();
    let input = StringInputSource::new(generated.as_str());
    let mut parser = Parser::new(input, parser_config);

    parser
        .next_command()
        .expect("Failed to parse generated @annotation")
        .expect("No command parsed from generated @annotation output")
}

#[test]
fn test_text_round_trip_mid_newline() {
    // Content with newline in the middle
    let parsed = round_trip_text("Hello\nWorld", false);
    assert_eq!(parsed, Command::new_text("Hello\nWorld".to_string()));
}

#[test]
fn test_text_round_trip_trailing_newline() {
    // Trailing newline is trimmed by the parser (existing behavior), but
    // no spurious backslash should remain.
    let parsed = round_trip_text("Hello\n", false);
    assert_eq!(parsed, Command::new_text("Hello".to_string()));
}

#[test]
fn test_text_round_trip_backslash_plus_newline() {
    // Content containing a backslash followed by a newline.
    // The writer prepends another backslash; the parser strips only the
    // backslash immediately before the newline, leaving the user's backslash.
    let parsed = round_trip_text("Hello\\\nWorld", false);
    assert_eq!(parsed, Command::new_text("Hello\\\nWorld".to_string()));
}

#[test]
fn test_text_round_trip_two_backslashes_plus_newline() {
    // Two backslashes followed by a newline.
    let parsed = round_trip_text("Hello\\\\\nWorld", false);
    assert_eq!(parsed, Command::new_text("Hello\\\\\nWorld".to_string()));
}

#[test]
fn test_annotation_round_trip_mid_newline() {
    let parsed = round_trip_annotation("Hello\nWorld");
    assert_eq!(
        parsed,
        Command::new_annotation("Hello\nWorld".to_string())
    );
}

#[test]
fn test_annotation_round_trip_trailing_newline() {
    // Trailing newline trimmed, no spurious backslash
    let parsed = round_trip_annotation("Hello\n");
    assert_eq!(parsed, Command::new_annotation("Hello".to_string()));
}

#[test]
fn test_annotation_round_trip_backslash_plus_newline() {
    let parsed = round_trip_annotation("Hello\\\nWorld");
    assert_eq!(
        parsed,
        Command::new_annotation("Hello\\\nWorld".to_string())
    );
}

#[test]
fn test_annotation_round_trip_two_backslashes_plus_newline() {
    let parsed = round_trip_annotation("Hello\\\\\nWorld");
    assert_eq!(
        parsed,
        Command::new_annotation("Hello\\\\\nWorld".to_string())
    );
}

#[test]
fn test_text_round_trip_preserve_indent() {
    // With preserve_indent=true, leading indentation is preserved.
    // Mid-content newline keeps internal indentation.
    let parsed = round_trip_text("  Hello\n  World", true);
    assert_eq!(parsed, Command::new_text("  Hello\n  World".to_string()));
}
```

- [ ] **Step 2: Run round-trip tests to verify they pass**

Run: `cargo test --test test_writer`
Expected: PASS — all pre-existing tests and all new round-trip tests pass. The round-trip works because:
1. Writer escapes `\n` → `\<newline>` in `@text`/`@annotation` content.
2. `Input::next_line` treats `\<newline>` as line continuation, accumulating the full content into one logical line.
3. Parser unescapes `\<newline>` → `\n` before trimming, recovering the original content.

- [ ] **Step 3: Commit**

```bash
git add tests/test_writer.rs
git commit -m "test: add round-trip tests for @text/@annotation newline escaping

Verify that @text and @annotation commands with embedded newlines,
backslash+newline combinations, trailing newlines, and preserve_indent
mode survive a write→parse round-trip correctly."
```

---

## Final Verification

After all three tasks are complete, run the full AGENTS.md verification checklist:

- [ ] **Build:** `cargo build --release --workspace` succeeds
- [ ] **Test:** `cargo test` passes (including all new unit and integration tests)
- [ ] **Lint:** `cargo clippy --workspace` has no warnings
- [ ] **Doc:** `cargo doc --workspace --no-deps` generates without errors
- [ ] **FFI:** `make ffi-test` passes (FFI surface unchanged; behavior change is internal)
