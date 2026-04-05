cmake_minimum_required(VERSION 3.22)
project(koicore_ffi_integration_test LANGUAGES C CXX)

# Example: Using FetchContent to integrate koicore_ffi
# This demonstrates the recommended way to include koicore_ffi in external projects

include(FetchContent)

# Fetch koicore_ffi from Git repository (production usage)
# For development/testing, you can override with local path using:
#   cmake -D USE_LOCAL_KOICORE_FFI=ON ..
if(USE_LOCAL_KOICORE_FFI)
    # Use absolute path for local development
    # CMAKE_CURRENT_SOURCE_DIR points to the directory containing this CMakeLists.txt
    set(KOICORE_FFI_LOCAL_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../..")
    message(STATUS "Using local koicore_ffi from: ${KOICORE_FFI_LOCAL_PATH}")
    FetchContent_Declare(
        koicore_ffi
        SOURCE_DIR ${KOICORE_FFI_LOCAL_PATH}
    )
else()
    # Production usage - fetch from Git repository
    FetchContent_Declare(
        koicore_ffi
        GIT_REPOSITORY https://github.com/Visecy/koicore.git
        GIT_TAG main
        SOURCE_SUBDIR crates/koicore_ffi
    )
endif()

# Make koicore_ffi available - this will build the Rust library
FetchContent_MakeAvailable(koicore_ffi)

# Now you can link against the koicore_ffi target
add_executable(integration_test main.c)
target_link_libraries(integration_test PRIVATE koicore_ffi)

# Add include directories for the generated header
# When using FetchContent with SOURCE_SUBDIR, the header is in the source directory
target_include_directories(integration_test PRIVATE
    ${koicore_ffi_SOURCE_DIR}
)

# Enable testing
enable_testing()
add_test(NAME integration_test COMMAND integration_test)
