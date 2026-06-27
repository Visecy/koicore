# @text / @annotation Newline Escape — Design Spec

## Goal

Fix the broken round-trip for `@text` and `@annotation` commands whose content contains embedded newline characters (`\n`). Currently the writer emits raw newlines, which `Input::next_line` interprets as command separators, splitting one logical command into multiple. The fix introduces a minimal escape scheme that reuses the existing line-continuation mechanism.

## Background

### Current behavior (broken)

* **Writer** ([generators.rs:47-65](file:///home/ovizro/Code/libkoilang/src/writer/generators.rs#L47-L65)): writes `@text` / `@annotation` content verbatim, then appends a trailing `\n` via `writeln!`.

* **`Input::next_line`** ([input.rs:210-235](file:///home/ovizro/Code/libkoilang/src/parser/input.rs#L210-L235)): splits input into logical lines, accumulating physical lines that end with `\<newline>` (backslash + newline) as line continuation.

* **Parser** ([mod.rs:324-341](file:///home/ovizro/Code/libkoilang/src/parser/mod.rs#L324-L341): for text/annotation branches, takes `line_text.trim_end()` / `trimmed` directly as the content, without processing any escape sequences.

As a result, a `@text("Hello\nWorld")` command is serialized as `Hello\nWorld\n`, which the parser reads as **two** commands: `@text("Hello")` and `@text("World")`.

### Why `@text` / `@annotation` differ from regular command parameters

Regular command parameters are quoted strings, and `format_string` ([formatters.rs:176-198](file:///home/ovizro/Code/libkoilang/src/writer/formatters.rs#L176-L198)) already escapes `\n` → `\\n` (backslash + literal `n`). The quoted-string parser ([command\_parser.rs](file:///home/ovizro/Code/libkoilang/src/parser/command_parser.rs)) decodes `\\n` back to `\n`.

`@text` / `@annotation` are **raw text** commands — their content is not a quoted string and does not go through `parse_command_line`. They need their own escape mechanism.

## Design

### Escape scheme

A minimal, non-C-style scheme:

| Direction           | Rule                                       | Implementation            |
| ------------------- | ------------------------------------------ | ------------------------- |
| **Writer escape**   | Prepend one `\` before each `\n`           | `s.replace('\n', "\\\n")` |
| **Parser unescape** | Strip one `\` immediately before each `\n` | `s.replace("\\\n", "\n")` |

**Key property:** The parser strips exactly the `\` directly preceding a `\n`, regardless of how many `\` precede it. Other `\` are left untouched. This is **not** standard C-style escape (which would also unescape `\\` → `\`); it is deliberately simpler and sufficient for round-trip correctness.

### Round-trip proof

Notation: `⏎` = literal newline char, `\` = literal backslash char.

| Original content                        | Writer output (content + trailing `⏎`) | `Input::next_line` accumulation    | Parser unescape + trim                                                                                   |
| --------------------------------------- | -------------------------------------- | ---------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `Hello⏎World`                           | `Hello\⏎World⏎`                        | `Hello\⏎World⏎` (1 continuation)   | `Hello⏎World` ✓                                                                                          |
| `Hello\⏎World` (backslash+newline)      | `Hello\\⏎World⏎`                       | `Hello\\⏎World⏎` (1 continuation)  | `Hello\⏎World` ✓                                                                                         |
| `Hello\\⏎World` (2 backslashes+newline) | `Hello\\\⏎World⏎`                      | `Hello\\\⏎World⏎` (1 continuation) | `Hello\\⏎World` ✓                                                                                        |
| `Hello⏎` (trailing newline)             | `Hello\⏎⏎`                             | `Hello\⏎⏎` (1 continuation)        | unescape → `Hello⏎⏎` → trim\_end → `Hello` (trailing newline dropped, consistent with existing behavior) |
| `Hello` (no newline)                    | `Hello⏎`                               | `Hello⏎`                           | `Hello` ✓                                                                                                |

**Why** **`str::replace`** **is correct for unescape:** `str::replace` scans left-to-right for non-overlapping occurrences of the 2-byte pattern `\<newline>`. For `\\\⏎` (3 backslashes + newline), it matches at position 2 (the third `\` + `⏎`), never at positions 0 or 1 (which are `\\`, not `\⏎`). The first two `\` are preserved.

### Critical ordering: unescape BEFORE trim

The parser **must** unescape `line_text` **before** applying `trim_end()` / `trim()`. If trim runs first:

```
line_text:   Hello\⏎⏎        (escaped trailing newline + writer's trailing newline)
trim_end():  Hello\          (both ⏎ removed, leaves dangling backslash)
unescape():  Hello\          (no \⏎ pattern left — BROKEN, spurious backslash)
```

Correct order:

```
line_text:   Hello\⏎⏎
unescape():  Hello⏎⏎        (\\⏎ → ⏎)
trim_end():  Hello           (trailing ⏎ removed — acceptable, matches existing behavior)
```

This ordering also handles the annotation case correctly because `#` prefix characters are unaffected by newline unescaping, so `hash_count` computed from the pre-unescape `trimmed` remains valid.

## Components

### 1. Writer — `escape_text_newlines`

**File:** [src/writer/generators.rs](file:///home/ovizro/Code/libkoilang/src/writer/generators.rs)

Add a `pub(crate)` helper function and call it in the `@text` and `@annotation` branches of `write_command_with_param_options`.

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

**Call sites:**

* `@text` branch (line 49-51): `write!(writer, "{}", escape_text_newlines(text))?;`

* `@annotation` branch (line 53-65): compute `let escaped = escape_text_newlines(text);` and write `escaped` instead of `text`. The `text.trim_start().starts_with(&hashes)` check stays on the **original** `text` (escaping does not affect leading `#`).

### 2. Parser — `unescape_text_newlines`

**File:** [src/parser/mod.rs](file:///home/ovizro/Code/libkoilang/src/parser/mod.rs)

Add a private helper function and apply it in the text/annotation branches of `next_command_with_source` (currently lines 324-341).

```rust
/// Reverse the writer's newline escaping for `@text` / `@annotation` content.
///
/// Each `\<newline>` (backslash + newline) becomes `<newline>` (just newline).
/// Only the backslash immediately preceding a newline is stripped; all other
/// backslashes are preserved verbatim.
fn unescape_text_newlines(s: &str) -> String {
    s.replace("\\\n", "\n")
}
```

**Call sites** (text branch, currently lines 324-330):

```rust
if hash_count < self.config.command_threshold {
    let unescaped = unescape_text_newlines(line_text);
    let text_content = if self.config.preserve_indent {
        unescaped.trim_end().to_string()
    } else {
        unescaped.trim().to_string()
    };
    break Ok(Some((Command::new_text(text_content), source)));
}
```

**Call sites** (annotation branch, currently lines 331-341):

```rust
} else if hash_count > self.config.command_threshold {
    if self.config.skip_annotations {
        continue;
    }
    // hash_count from pre-unescape trimmed is still valid: `#` is unaffected
    // by newline unescaping.
    let unescaped = unescape_text_newlines(line_text);
    let annotation_content = if self.config.preserve_indent {
        unescaped.trim_end().to_string()
    } else {
        let content: String = unescaped.trim().chars().skip(hash_count).collect();
        content.trim().to_string()
    };
    break Ok(Some((Command::new_annotation(annotation_content), source)));
}
```

Note: `hash_count` is still computed from the pre-unescape `trimmed` (line 322). This is safe because newline unescaping does not insert or remove `#` characters, so the count of leading `#` is identical before and after unescape.

### 3. Tests

**File:** [tests/test\_writer.rs](file:///home/ovizro/Code/libkoilang/tests/test_writer.rs)

Add a round-trip test module covering:

1. `@text` with `\n` in the middle
2. `@text` ending with `\n` (trailing newline dropped, no spurious backslash)
3. `@text` containing `\` + `\n` (backslash + newline)
4. `@text` containing `\\` + `\n` (two backslashes + newline)
5. `@annotation` with the same four cases
6. A `preserve_indent: true` variant for one mid-content newline case, verifying leading indentation is preserved

Each test constructs a `Command`, writes it, parses the output back, and asserts the parsed command equals the original (with the documented exception that trailing newlines are trimmed).

### 4. Documentation

Add a short doc-comment block above the `@text` / `@annotation` match arms in [generators.rs](file:///home/ovizro/Code/libkoilang/src/writer/generators.rs) and above the text/annotation branches in [parser/mod.rs](file:///home/ovizro/Code/libkoilang/src/parser/mod.rs) explaining the escape rule and the round-trip guarantee. The helper functions themselves carry doc comments (shown above).

## Out of scope (YAGNI)

* **`\r`** **(CR) handling:** `BufReadWrapper` already normalizes `\r\n` → `\n` before lines reach `Input::next_line`. `StringInputSource` is intended for programmatic use where the caller controls line endings. Standalone `\r` (old Mac line endings) is not handled.

* **Standard command parameters:** Quoted-string parameter escaping (`format_string`) is a separate, already-correct mechanism and is not touched.

* **`Input::next_line`** **changes:** The existing line-continuation logic is reused unchanged. No backslash-counting heuristic is added.

* **Escaping** **`\`** **in the middle of content when not followed by** **`\n`:** Not needed for round-trip correctness; the parser only strips `\` before `\n`.

* **Trailing backslash handling:** If content ends with `\`, the writer appends an extra `\` to prevent the writer's trailing newline (from `writeln!`) from creating a bare `\<newline>` continuation that the parser would strip. The extra backslash is naturally reversed by `unescape_text_newlines` because it strips exactly one backslash before each newline. This is NOT full backslash escaping — only the trailing backslash gets an extra one.

## Verification

Per AGENTS.md verification checklist:

* `cargo build --release --workspace` succeeds

* `cargo test` passes (including new round-trip tests)

* `cargo clippy --workspace` has no warnings

* `cargo doc --workspace --no-deps` generates without errors

* `make ffi-test` passes (FFI surface unchanged; behavior change is internal to writer/parser)

