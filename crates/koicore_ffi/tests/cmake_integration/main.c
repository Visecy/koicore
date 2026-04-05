/**
 * CMake Integration Test for KoicoreFFI
 * 
 * This test demonstrates basic usage of the koicore_ffi library
 * when integrated via CMake.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Include the generated header
// When using FetchContent, the header should be available through the target's include directories
#include "koicore.h"

int main(void) {
    printf("Testing KoicoreFFI CMake Integration...\n");
    
    // Test 1: Create a parser from a simple KoiLang command
    // Use a mutable buffer since FFI may need to modify the string
    char test_input[] = "#test_command param1 param2";
    printf("Input: %s\n", test_input);
    
    struct KoiInputSource* input = KoiInputSource_FromString(test_input);
    if (!input) {
        fprintf(stderr, "Failed to create input source\n");
        return 1;
    }
    
    struct KoiParserConfig config;
    KoiParserConfig_Init(&config);
    
    struct KoiParser* parser = KoiParser_New(input, &config);
    if (!parser) {
        fprintf(stderr, "Failed to create parser\n");
        KoiInputSource_Del(input);
        return 1;
    }
    
    // Test 2: Parse a command
    struct KoiCommand* cmd = KoiParser_NextCommand(parser);
    if (cmd) {
        char name_buffer[256];
        uintptr_t name_len = KoiCommand_GetName(cmd, name_buffer, sizeof(name_buffer));
        if (name_len < sizeof(name_buffer)) {
            name_buffer[name_len] = '\0';
        } else {
            name_buffer[sizeof(name_buffer) - 1] = '\0';
        }
        printf("Parsed command: %s\n", name_buffer);
        
        // Test 3: Get command parameters
        uintptr_t param_count = KoiCommand_GetParamCount(cmd);
        printf("Parameter count: %zu\n", param_count);
        
        // Clean up command
        KoiCommand_Del(cmd);
    } else {
        printf("No command parsed\n");
    }
    
    // Clean up
    // Note: KoiInputSource is owned by the parser after KoiParser_New, so we only free the parser
    KoiParser_Del(parser);
    
    printf("CMake Integration Test Completed Successfully!\n");
    return 0;
}
