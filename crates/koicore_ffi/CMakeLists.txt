cmake_minimum_required(VERSION 3.22)
project(koicore_ffi VERSION 0.2.3 LANGUAGES C CXX)

# Include FetchContent for Corrosion
include(FetchContent)

# Fetch Corrosion v0.6
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.6
)

# Set any global configuration variables such as `Rust_TOOLCHAIN` before this line!
FetchContent_MakeAvailable(Corrosion)

# Import only the koicore_ffi crate from Cargo.toml
# This creates CMake targets for each crate type (cdylib, staticlib)
# Use CRATES argument to import only specific crates, not the entire workspace
corrosion_import_crate(MANIFEST_PATH Cargo.toml CRATES koicore_ffi)

# Note: koicore.h is included in the repository and does not need to be generated

# Configure target properties for koicore_ffi
# Add include directories for the generated header
# For INTERFACE libraries created by corrosion, we need to use INTERFACE keyword
target_include_directories(koicore_ffi
    INTERFACE
        $<BUILD_INTERFACE:${CMAKE_CURRENT_BINARY_DIR}>
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}>
        $<INSTALL_INTERFACE:include>
)

# Add the header as a target source (CMake >= 3.23)
if(CMAKE_VERSION VERSION_GREATER_EQUAL "3.23")
    target_sources(koicore_ffi
        INTERFACE
            FILE_SET HEADERS
            BASE_DIRS ${CMAKE_CURRENT_SOURCE_DIR}
            FILES
                ${CMAKE_CURRENT_SOURCE_DIR}/koicore.h
    )
endif()

# Set output directory for the library
set_target_properties(koicore_ffi
    PROPERTIES
        OUTPUT_NAME koicore_ffi
        VERSION ${PROJECT_VERSION}
        SOVERSION ${PROJECT_VERSION_MAJOR}
)

# Installation using corrosion_install
include(GNUInstallDirs)
include(CMakePackageConfigHelpers)

# Install the Rust library target
corrosion_install(TARGETS koicore_ffi EXPORT KoicoreFFITargets
    ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)

# Install header files
# The header is included in the repository
install(FILES
    ${CMAKE_CURRENT_SOURCE_DIR}/koicore.h
    DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
)

# Generate version file
write_basic_package_version_file(
    "${CMAKE_CURRENT_BINARY_DIR}/KoicoreFFIConfigVersion.cmake"
    VERSION ${PROJECT_VERSION}
    COMPATIBILITY AnyNewerVersion
)

# Configure package config file
configure_package_config_file(
    "${CMAKE_CURRENT_SOURCE_DIR}/cmake/KoicoreFFIConfig.cmake.in"
    "${CMAKE_CURRENT_BINARY_DIR}/KoicoreFFIConfig.cmake"
    INSTALL_DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/koicore_ffi
)

# Install CMake configuration files
install(EXPORT KoicoreFFITargets
    FILE KoicoreFFITargets.cmake
    NAMESPACE KoiFFI::
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/koicore_ffi
)

install(FILES
    "${CMAKE_CURRENT_BINARY_DIR}/KoicoreFFIConfig.cmake"
    "${CMAKE_CURRENT_BINARY_DIR}/KoicoreFFIConfigVersion.cmake"
    DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/koicore_ffi
)

# Export targets for build tree usage
export(EXPORT KoicoreFFITargets
    FILE "${CMAKE_CURRENT_BINARY_DIR}/KoicoreFFITargets.cmake"
    NAMESPACE KoiFFI::
)
