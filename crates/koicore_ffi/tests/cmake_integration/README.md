# KoicoreFFI CMake Integration Example

This directory demonstrates how to integrate the `koicore_ffi` library into your CMake project.

## Quick Start

### Using FetchContent (Recommended)

Add this to your `CMakeLists.txt`:

```cmake
cmake_minimum_required(VERSION 3.22)
project(your_project LANGUAGES C CXX)

include(FetchContent)

# Fetch Corrosion (Rust-CMake integration tool)
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.6
)

# Fetch koicore_ffi from Git repository
FetchContent_Declare(
    koicore_ffi
    GIT_REPOSITORY https://github.com/Visecy/koicore.git
    GIT_TAG main  # Or specify a version tag like "v0.2.3"
    SOURCE_SUBDIR crates/koicore_ffi
)

FetchContent_MakeAvailable(Corrosion koicore_ffi)

# Link against the library
add_executable(your_app main.c)
target_link_libraries(your_app PRIVATE koicore_ffi)
```

### Using find_package (After Installation)

First, install koicore_ffi:

```bash
cd crates/koicore_ffi
mkdir build && cd build
cmake ..
cmake --build .
cmake --install . --prefix /usr/local
```

Then in your project's `CMakeLists.txt`:

```cmake
cmake_minimum_required(VERSION 3.22)
project(your_project LANGUAGES C CXX)

find_package(KoicoreFFI REQUIRED)

add_executable(your_app main.c)
target_link_libraries(your_app PRIVATE KoiFFI::koicore_ffi)
```

## Building This Example

```bash
cd tests/cmake_integration
mkdir build && cd build
cmake ..
cmake --build .
./integration_test
```

## Requirements

- CMake 3.22 or newer
- Rust toolchain (1.46 or newer)
- cbindgen (for header generation): `cargo install cbindgen`

## Cargo Configuration Passthrough

You can customize the Rust build using Corrosion's functions:

```cmake
# Set environment variables for the Rust build
corrosion_set_env_vars(koicore_ffi "RUSTFLAGS=-C target-cpu=native")

# Add custom rustflags
corrosion_add_target_rustflags(koicore_ffi "-C opt-level=3")

# Use a specific Rust toolchain
set(Rust_TOOLCHAIN "nightly")
```

## Cross-Compilation

For cross-compilation, use CMake's toolchain file:

```bash
cmake .. -DCMAKE_TOOLCHAIN_FILE=/path/to/toolchain.cmake
```

Corrosion will automatically pass the target to Cargo.
