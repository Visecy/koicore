# AGENTS.md

## Project Context

KoiLang is a Rust workspace (koicore + koicore_ffi + koicli). KoiLang is a markup language that emphasizes high readability by embedding structured commands within natural language text. It does not provide built-in commands or domain-specific features, but offers a flexible syntax where applications define their own command semantics.
For full project overview, see README.md.

## Essential Workflow

| Action | Command |
|--------|---------|
| Build | `cargo build --release --workspace` |
| Test | `cargo test` |
| FFI tests | `make ffi-test` |
| CMake tests | `make cmake-integration-test` |
| Doc | `cargo doc --workspace --no-deps` |
| Lint | `cargo clippy --workspace` |

## Critical Conventions

1. **Rust Edition 2024** — Use 2024 edition syntax
2. **FFI Memory Ownership** — MUST be explicit and documented:
   - Functions returning owned memory must be clearly marked
   - Caller is responsible for freeing what they allocate
   - Use `cbindgen` for header generation
3. **Documentation** — All public items need doc comments
4. **Error Handling** — Use `Result` types with descriptive errors

## Module Quick Reference

- `koicore/src/parser/` — Streaming parser (Parser, ParserConfig, inputs)
- `koicore/src/command/` — Command and Parameter structs
- `koicore/src/writer/` — KoiLang code generation
- `koicore_ffi/src/` — C-compatible FFI API

## Verification Checklist

Before completing any task:
- [ ] `cargo build --release --workspace` succeeds
- [ ] `cargo test` passes
- [ ] `cargo clippy --workspace` has no warnings
- [ ] `cargo doc --workspace --no-deps` generates without errors
- [ ] FFI tests pass: `make ffi-test`
