# AGENTS.md

## Project Overview

KoiLang is a markup language designed for narrative content (visual novels, interactive fiction). The project is a Rust workspace consisting of:

- **koicore** - Core parsing and writing library
- **koicore_ffi** - C-compatible FFI bindings
- **koicli** - Command-line interface

## Build Commands

```bash
# Build entire workspace
cargo build --release --workspace

# Run tests
cargo test

# Run FFI tests (requires CMake)
make ffi-test

# CMake integration tests
make cmake-integration-test

# CMake build
make cmake-build

# Documentation
cargo doc --workspace --no-deps
```

## Project Structure

```
koicore/
├── Cargo.toml              # Workspace manifest
├── Makefile                # Build automation
├── src/                    # koicore source (parser, writer, command structures)
├── examples/               # Usage examples
├── benches/                # Performance benchmarks
├── crates/
│   ├── koicore_ffi/        # FFI bindings with cbindgen
│   │   ├── src/            # C API implementation
│   │   ├── include/        # C header files
│   │   └── tests/           # Integration tests
│   └── koicli/             # CLI tool
```

## Key Modules

### koicore
- `parser/` - Streaming parser with `Parser`, `ParserConfig`, input sources
- `command/` - `Command` and `Parameter` data structures
- `writer/` - KoiLang code generation

### koicore_ffi
- C-compatible API in `src/`
- Generated headers via cbindgen
- CMake build support for integration

## Code Conventions

1. **Rust Edition 2024** - Project uses Rust edition 2024
2. **Documentation** - All public items should have doc comments
3. **Error Handling** - Use `Result` types with descriptive errors
4. **FFI** - Memory ownership must be explicit and documented

## Testing

- Unit tests: `cargo test`
- FFI tests: `make ffi-test` (C++ integration tests)
- CMake tests: `make cmake-integration-test`
- Benchmarks: `cargo bench`
