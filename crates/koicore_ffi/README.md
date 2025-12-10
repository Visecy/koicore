# koicore FFI (Foreign Function Interface)

koicore provides a C-compatible Foreign Function Interface (FFI) that allows integration with C, C++, and other programming languages that can call C functions.

## Overview

The koicore FFI module (`koicore_ffi`) exposes the core KoiLang parsing functionality through a C API. This enables applications written in C, C++, or other languages to parse KoiLang files and work with parsed commands.

## Features

- **C-compatible API**: Full C API with C++ wrapper support
- **Memory Management**: Safe memory management with explicit ownership
- **Comprehensive Coverage**: Access to all core koicore functionality
- **Error Handling**: Detailed error reporting with position information
- **Multiple Input Sources**: Support for strings, files, and custom input sources
- **Composite Parameters**: Full support for lists and dictionaries
- **Thread Safety**: Designed for safe concurrent use

## Building

### Prerequisites

- Rust toolchain (stable)
- C++17 compatible compiler
- CMake 3.14 or later
- GoogleTest (automatically fetched by CMake)

### Build Steps

```bash
# Build the Rust library
make build

# Build and run C++ tests
make ffi-test
```

### Linking

Link against the compiled `libkoicore_ffi.a` (static library) or `libkoicore_ffi.so` (shared library):

```cmake
# CMakeLists.txt
target_link_libraries(your_target koicore_ffi)
```

## Quick Start

### Basic Parsing Example

```c
#include "koicore.h"
#include <stdio.h>
#include <stdlib.h>

int main() {
    // Create input source from string
    KoiInputSource* source = KoiInputSource_FromString(
        "#character Alice \"Hello, world!\"\n"
        "This is regular text.\n"
        "#background Forest"
    );
    
    if (!source) {
        fprintf(stderr, "Failed to create input source\n");
        return 1;
    }
    
    // Initialize parser configuration
    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    
    // Create parser
    KoiParser* parser = KoiParser_New(source, &config);
    if (!parser) {
        fprintf(stderr, "Failed to create parser\n");
        KoiInputSource_Del(source);
        return 1;
    }
    
    // Parse commands
    KoiCommand* command;
    while ((command = KoiParser_NextCommand(parser)) != NULL) {
        // Get command name
        char name[256];
        uintptr_t name_len = KoiCommand_GetName(command, name, sizeof(name));
        name[name_len] = '\0';
        
        printf("Command: %s\n", name);
        
        // Handle different command types
        if (KoiCommand_IsTextCommand(command)) {
            printf("  Type: Text\n");
        } else if (KoiCommand_IsAnnotationCommand(command)) {
            printf("  Type: Annotation\n");
        } else if (KoiCommand_IsNumberCommand(command)) {
            printf("  Type: Number\n");
        }
        
        // Print parameters
        uintptr_t param_count = KoiCommand_GetParamCount(command);
        for (uintptr_t i = 0; i < param_count; i++) {
            char param_str[256];
            uintptr_t param_len = KoiCommand_GetStringParam(command, i, param_str, sizeof(param_str));
            param_str[param_len] = '\0';
            printf("  Param %zu: %s\n", i, param_str);
        }
        
        // Clean up command
        KoiCommand_Del(command);
    }
    
    // Clean up parser
    KoiParser_Del(parser);
    
    return 0;
}
```

## API Reference

### Input Sources

#### String Input Source

```c
KoiInputSource* KoiInputSource_FromString(const char* source);
```

Creates an input source from a null-terminated C string.

**Parameters:**
- `source`: UTF-8 encoded string to parse

**Returns:** Input source pointer or NULL on error

#### File Input Source

```c
KoiInputSource* KoiInputSource_FromFile(const char* path);
KoiInputSource* KoiInputSource_FromFileAndEncoding(
    const char* path,
    const char* encoding,
    enum KoiFileInputEncodingStrategy encoding_strategy
);
```

Creates an input source from a file with optional encoding specification.

**Parameters:**
- `path`: File path (null-terminated)
- `encoding`: Character encoding (e.g., "utf-8", "gbk")
- `encoding_strategy`: How to handle encoding errors

**Encoding Strategies:**
- `Strict`: Panic on invalid sequences
- `Replace`: Replace with U+FFFD replacement character
- `Ignore`: Skip invalid sequences

#### Custom Input Source

```c
typedef struct KoiTextInputSourceVTable {
    char* (*next_line)(void* user_data);
    const char* (*source_name)(void* user_data);
} KoiTextInputSourceVTable;

KoiInputSource* KoiInputSource_FromVTable(
    const struct KoiTextInputSourceVTable* vtable,
    void* user_data
);
```

Create custom input sources by providing function pointers.

#### Input Source Cleanup

```c
void KoiInputSource_Del(KoiInputSource* input);
```

Frees input source memory.

### Parser Configuration

```c
typedef struct KoiParserConfig {
    uintptr_t command_threshold;    // Default: 1
    bool skip_annotations;          // Default: false
    bool convert_number_command;    // Default: true
} KoiParserConfig;

void KoiParserConfig_Init(struct KoiParserConfig* config);
```

Initialize configuration with default values.

### Parser Operations

```c
KoiParser* KoiParser_New(KoiInputSource* input, KoiParserConfig* config);
void KoiParser_Del(KoiParser* parser);
KoiCommand* KoiParser_NextCommand(KoiParser* parser);
KoiParserError* KoiParser_Error(KoiParser* parser);
```

Create and use parser instances.

### Command Operations

#### Command Creation

```c
KoiCommand* KoiCommand_New(const char* name);
KoiCommand* KoiCommand_NewText(const char* content);
KoiCommand* KoiCommand_NewAnnotation(const char* content);
KoiCommand* KoiCommand_NewNumber(int64_t value);
void KoiCommand_Del(KoiCommand* command);
```

Create different types of commands.

#### Command Information

```c
uintptr_t KoiCommand_GetName(KoiCommand* command, char* buffer, uintptr_t buffer_size);
uintptr_t KoiCommand_GetNameLen(KoiCommand* command);
uintptr_t KoiCommand_GetParamCount(KoiCommand* command);
int32_t KoiCommand_GetParamType(KoiCommand* command, uintptr_t index);
```

Get command metadata and parameter information.

#### Command Type Checking

```c
int32_t KoiCommand_IsTextCommand(KoiCommand* command);
int32_t KoiCommand_IsAnnotationCommand(KoiCommand* command);
int32_t KoiCommand_IsNumberCommand(KoiCommand* command);
```

Check command type.

#### Parameter Access

```c
int32_t KoiCommand_GetIntParam(KoiCommand* command, uintptr_t index, int64_t* out_value);
int32_t KoiCommand_GetFloatParam(KoiCommand* command, uintptr_t index, double* out_value);
uintptr_t KoiCommand_GetStringParam(KoiCommand* command, uintptr_t index, char* buffer, uintptr_t buffer_size);
uintptr_t KoiCommand_GetStringParamLen(KoiCommand* command, uintptr_t index);
```

Get parameter values by type.

#### Parameter Modification

```c
int32_t KoiCommand_AddIntParameter(KoiCommand* command, int64_t value);
int32_t KoiCommand_AddFloatParameter(KoiCommand* command, double value);
int32_t KoiCommand_AddStringParameter(KoiCommand* command, const char* value);
int32_t KoiCommand_SetIntParameter(KoiCommand* command, uintptr_t index, int64_t value);
int32_t KoiCommand_SetFloatParameter(KoiCommand* command, uintptr_t index, double value);
int32_t KoiCommand_SetStringParameter(KoiCommand* command, uintptr_t index, const char* value);
int32_t KoiCommand_RemoveParameter(KoiCommand* command, uintptr_t index);
int32_t KoiCommand_ClearParameters(KoiCommand* command);
```

Modify command parameters.

### Composite Parameters

#### Lists

```c
// Create list
KoiCompositeList* KoiCompositeList_New(void);
void KoiCompositeList_Del(KoiCompositeList* list);

// Get list from command
KoiCompositeList* KoiCommand_GetCompositeList(KoiCommand* command, uintptr_t index);

// List operations
uintptr_t KoiCompositeList_GetLength(KoiCompositeList* list);
int32_t KoiCompositeList_GetValueType(KoiCompositeList* list, uintptr_t index);

// Add values
int32_t KoiCompositeList_AddIntValue(KoiCompositeList* list, int64_t value);
int32_t KoiCompositeList_AddFloatValue(KoiCompositeList* list, double value);
int32_t KoiCompositeList_AddStringValue(KoiCompositeList* list, const char* value);

// Get values
int32_t KoiCompositeList_GetIntValue(KoiCompositeList* list, uintptr_t index, int64_t* out_value);
int32_t KoiCompositeList_GetFloatValue(KoiCompositeList* list, uintptr_t index, double* out_value);
uintptr_t KoiCompositeList_GetStringValue(KoiCompositeList* list, uintptr_t index, char* buffer, uintptr_t buffer_size);
uintptr_t KoiCompositeList_GetStringValueLen(KoiCompositeList* list, uintptr_t index);

// Modify values
int32_t KoiCompositeList_SetIntValue(KoiCompositeList* list, uintptr_t index, int64_t value);
int32_t KoiCompositeList_SetFloatValue(KoiCompositeList* list, uintptr_t index, double value);
int32_t KoiCompositeList_SetStringValue(KoiCompositeList* list, uintptr_t index, const char* value);

// Remove values
int32_t KoiCompositeList_RemoveValue(KoiCompositeList* list, uintptr_t index);
int32_t KoiCompositeList_Clear(KoiCompositeList* list);
```

#### Dictionaries

```c
// Create dict
KoiCompositeDict* KoiCompositeDict_New(void);
void KoiCompositeDict_Del(KoiCompositeDict* dict);

// Get dict from command
KoiCompositeDict* KoiCommand_GetCompositeDict(KoiCommand* command, uintptr_t index);

// Dict operations
uintptr_t KoiCompositeDict_GetLength(KoiCompositeDict* dict);

// Set values
int32_t KoiCompositeDict_SetIntValue(KoiCompositeDict* dict, const char* key, int64_t value);
int32_t KoiCompositeDict_SetFloatValue(KoiCompositeDict* dict, const char* key, double value);
int32_t KoiCompositeDict_SetStringValue(KoiCompositeDict* dict, const char* key, const char* value);

// Get values
int32_t KoiCompositeDict_GetIntValue(KoiCompositeDict* dict, const char* key, int64_t* out_value);
int32_t KoiCompositeDict_GetFloatValue(KoiCompositeDict* dict, const char* key, double* out_value);
uintptr_t KoiCompositeDict_GetStringValue(KoiCompositeDict* dict, const char* key, char* buffer, uintptr_t buffer_size);
uintptr_t KoiCompositeDict_GetStringValueLen(KoiCompositeDict* dict, const char* key);

// Get value type
int32_t KoiCompositeDict_GetValueType(KoiCompositeDict* dict, const char* key);

// Get keys by index
uintptr_t KoiCompositeDict_GetKeybyIndex(KoiCompositeDict* dict, uintptr_t index, char* buffer, uintptr_t buffer_size);
uintptr_t KoiCompositeDict_GetKeyLenByIndex(KoiCompositeDict* dict, uintptr_t index);
int32_t KoiCompositeDict_GetValueTypeByIndex(KoiCompositeDict* dict, uintptr_t index);

// Remove entries
int32_t KoiCompositeDict_Remove(KoiCompositeDict* dict, const char* key);
int32_t KoiCompositeDict_Clear(KoiCompositeDict* dict);
```

### Error Handling

```c
void KoiParserError_Del(KoiParserError* error);

// Format error message
uintptr_t KoiParserError_Format(const KoiParserError* error, char* buffer, uintptr_t buffer_size);
uintptr_t KoiParserError_FormatLen(const KoiParserError* error);

// Get error message only
uintptr_t KoiParserError_GetMessage(const KoiParserError* error, char* buffer, uintptr_t buffer_size);
uintptr_t KoiParserError_GetMessageLen(const KoiParserError* error);

// Get error position
int32_t KoiParserError_GetTracebackPosition(const KoiParserError* error, uintptr_t* lineno, uintptr_t* column);
```

## Advanced Examples

### Working with Composite Lists

```c
// Parse a command with a list parameter
KoiCommand* cmd = KoiParser_NextCommand(parser);
if (cmd && !KoiCommand_IsTextCommand(cmd) && !KoiCommand_IsAnnotationCommand(cmd)) {
    // Get composite list parameter
    KoiCompositeList* list = KoiCommand_GetCompositeList(cmd, 0);
    if (list) {
        uintptr_t length = KoiCompositeList_GetLength(list);
        printf("List has %zu elements:\n", length);
        
        for (uintptr_t i = 0; i < length; i++) {
            int32_t type = KoiCompositeList_GetValueType(list, i);
            
            switch (type) {
                case 0: { // Int
                    int64_t value;
                    KoiCompositeList_GetIntValue(list, i, &value);
                    printf("  [%zu] Int: %lld\n", i, value);
                    break;
                }
                case 1: { // Float
                    double value;
                    KoiCompositeList_GetFloatValue(list, i, &value);
                    printf("  [%zu] Float: %f\n", i, value);
                    break;
                }
                case 2: { // String
                    char buffer[256];
                    uintptr_t len = KoiCompositeList_GetStringValue(list, i, buffer, sizeof(buffer));
                    buffer[len] = '\0';
                    printf("  [%zu] String: %s\n", i, buffer);
                    break;
                }
            }
        }
    }
}
KoiCommand_Del(cmd);
```

### Creating Commands with Composite Parameters

```c
// Create a command with a list parameter
KoiCommand* cmd = KoiCommand_New("draw");
KoiCommand_AddStringParameter(cmd, "Line");
KoiCommand_AddIntParameter(cmd, 2);

// Create and populate a list
KoiCompositeList* pos_list = KoiCompositeList_New();
KoiCompositeList_AddIntValue(pos_list, 16);
KoiCompositeList_AddIntValue(pos_list, 16);

// Note: Adding composite parameters directly to commands
// requires additional API (not shown in basic example)

// Clean up
KoiCompositeList_Del(pos_list);
KoiCommand_Del(cmd);
```

### Error Handling Example

```c
KoiCommand* cmd = KoiParser_NextCommand(parser);
if (!cmd) {
    KoiParserError* error = KoiParser_Error(parser);
    if (error) {
        // Get formatted error message
        uintptr_t msg_len = KoiParserError_FormatLen(error);
        char* error_msg = malloc(msg_len);
        if (error_msg) {
            KoiParserError_Format(error, error_msg, msg_len);
            fprintf(stderr, "Parse error: %s\n", error_msg);
            free(error_msg);
        }
        
        // Get position information
        uintptr_t line, column;
        if (KoiParserError_GetTracebackPosition(error, &line, &column) == 0) {
            fprintf(stderr, "Error at line %zu, column %zu\n", line, column);
        }
        
        KoiParserError_Del(error);
    }
}
```

### Custom Input Source Example

```c
#include <stdio.h>

// Custom data structure
typedef struct {
    const char** lines;
    size_t count;
    size_t current;
} CustomInputData;

// Custom next_line function
char* custom_next_line(void* user_data) {
    CustomInputData* data = (CustomInputData*)user_data;
    if (data->current >= data->count) {
        return NULL;  // End of input
    }
    return (char*)data->lines[data->current++];  // Return current line
}

// Custom source_name function
const char* custom_source_name(void* user_data) {
    return "custom_input";
}

// Usage
const char* lines[] = {
    "#character Alice \"Hello!\"",
    "This is text.",
    "#end"
};

CustomInputData custom_data = {
    .lines = lines,
    .count = 3,
    .current = 0
};

KoiTextInputSourceVTable vtable = {
    .next_line = custom_next_line,
    .source_name = custom_source_name
};

KoiInputSource* source = KoiInputSource_FromVTable(&vtable, &custom_data);
// Use the source...
```

## Memory Management

### Ownership Rules

1. **Input Sources**: Created with `KoiInputSource_From*` functions, must be freed with `KoiInputSource_Del`
2. **Parsers**: Created with `KoiParser_New`, must be freed with `KoiParser_Del`
3. **Commands**: Created with `KoiCommand_New*` or returned from parser, must be freed with `KoiCommand_Del`
4. **Composite Lists/Dicts**: Created with `KoiComposite*_New`, must be freed with `KoiComposite*_Del`
5. **Errors**: Created by parser, must be freed with `KoiParserError_Del`

### Lifetime Management

- **Borrowed References**: Composite lists/dicts obtained from commands are borrowed references owned by the command. Do not free them with `KoiComposite*_Del`
- **Owned References**: Composite lists/dicts created with `KoiComposite*_New` must be explicitly freed
- **Transferred Ownership**: When creating a parser with `KoiParser_New`, ownership of the input source is transferred to the parser

### Buffer Management

For string operations that require buffers:

1. Call the `*_Len` function first to get the required buffer size
2. Allocate a buffer of the returned size (including null terminator)
3. Call the actual function with the allocated buffer
4. Remember to null-terminate the string if the function doesn't do it automatically

## C++ Integration

For C++ users, koicore provides a namespace wrapper:

```cpp
#include "koicore.h"

using namespace koicore;

// All C functions are available in the koicore namespace
KoiInputSource* source = koicore::KoiInputSource_FromString("#test command");
KoiParserConfig config;
koicore::KoiParserConfig_Init(&config);
```

## Thread Safety

- Individual parser instances are not thread-safe
- Each thread should create its own parser instance
- Shared input sources can be used by multiple parsers if the source implementation is thread-safe
- Commands and composite parameters are immutable after creation and can be safely shared between threads

## Error Codes

Most functions return integer status codes:
- `0`: Success
- `-1`: Invalid parameters (NULL pointers, etc.)
- `-2`: Index out of bounds
- `-3`: Type mismatch
- `-4`: Invalid object state

String functions return the required buffer size when the provided buffer is insufficient.

## Testing

The FFI module includes comprehensive tests using GoogleTest. Run all tests:

```bash
cd crates/koicore_ffi/tests/cxx_api
mkdir build && cd build
cmake ..
make
ctest --verbose
```

Individual test executables:
- `test_parser`: Basic parser functionality
- `test_command`: Command operations
- `test_composite_list`: List parameter operations
- `test_composite_dict`: Dictionary parameter operations
- `test_parser_error`: Error handling
- `test_input_source`: Input source operations

## Troubleshooting

### Common Issues

1. **Linking Errors**: Ensure you're linking against the correct koicore_ffi library
2. **Encoding Issues**: Use `KoiInputSource_FromFileAndEncoding` for non-UTF-8 files
3. **Buffer Overflows**: Always check return values and allocate sufficient buffer space
4. **Memory Leaks**: Follow ownership rules and use proper cleanup functions
5. **Thread Safety**: Don't share parser instances between threads

### Debug Tips

1. Enable Rust debug symbols: `cargo build`
2. Use sanitizers: `RUSTFLAGS="-Z sanitizer=address" cargo build`
3. Check test output for usage examples
4. Use error formatting to get detailed error messages

## Performance Considerations

1. **Buffer Reuse**: Reuse buffers for string operations to reduce allocations
2. **Batch Processing**: Process multiple commands in batches when possible
3. **Memory Pools**: Consider implementing memory pools for high-frequency usage
4. **Streaming**: Use streaming parsers for large files to maintain constant memory usage

## Version Compatibility

The FFI API maintains backward compatibility within major versions. When upgrading:

1. Check the changelog for any breaking changes
2. Rebuild your application with the new library
3. Test thoroughly with your existing KoiLang files

## Contributing

When contributing to the FFI:

1. Maintain C compatibility
2. Add comprehensive tests
3. Update documentation
4. Follow Rust FFI best practices
5. Ensure thread safety where applicable
