cmake_minimum_required(VERSION 3.14)
project(koicore_ffi_test)

# GoogleTest requires at least C++17
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

include(FetchContent)
FetchContent_Declare(
  googletest
  URL https://github.com/google/googletest/archive/03597a01ee50ed33e9dfd640b249b4be3799d395.zip
)
# For Windows: Prevent overriding the parent project's compiler/linker settings
set(gtest_force_shared_crt ON CACHE BOOL "" FORCE)
FetchContent_MakeAvailable(googletest)

enable_testing()

link_directories(
  AFTER ${CMAKE_SOURCE_DIR}/../../../../target/release
)

# Test parser
add_executable(
  test_parser
  test_parser.cpp
)
target_link_libraries(
  test_parser
  GTest::gtest_main
  koicore_ffi
)

# Test command
add_executable(
  test_command
  test_command.cpp
)
target_link_libraries(
  test_command
  GTest::gtest_main
  koicore_ffi
)

# Test composite list
add_executable(
  test_composite_list
  test_composite_list.cpp
)
target_link_libraries(
  test_composite_list
  GTest::gtest_main
  koicore_ffi
)

# Test composite dict
add_executable(
  test_composite_dict
  test_composite_dict.cpp
)
target_link_libraries(
  test_composite_dict
  GTest::gtest_main
  koicore_ffi
)

# Test parser error
add_executable(
  test_parser_error
  test_parser_error.cpp
)
target_link_libraries(
  test_parser_error
  GTest::gtest_main
  koicore_ffi
)

# Test input source
add_executable(
  test_input_source
  test_input_source.cpp
)
target_link_libraries(
  test_input_source
  GTest::gtest_main
  koicore_ffi
)

include(GoogleTest)
gtest_discover_tests(test_parser)
gtest_discover_tests(test_command)
gtest_discover_tests(test_composite_list)
gtest_discover_tests(test_composite_dict)
gtest_discover_tests(test_parser_error)
gtest_discover_tests(test_input_source)
