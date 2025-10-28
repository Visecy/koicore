#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <gtest/gtest.h>
#include "../../koicore.h"

using namespace koicore;

// Test KoiParserError functionality
TEST(ParserErrorTest, TestErrorHandling) {
    // Create a parser with invalid input to trigger an error
    // Note: We need to create a scenario that will cause a parsing error
    // This might vary depending on the actual implementation
    
    // Create a parser with a string that might cause an error
    KoiInputSource* source = KoiInputSource_FromString("#");
    EXPECT_NE(source, nullptr);
    
    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    KoiParser* parser = KoiParser_New(source, &config);
    EXPECT_NE(parser, nullptr);
    
    // Try to get a command from an empty source
    KoiCommand* cmd = KoiParser_NextCommand(parser);
    // This might return NULL due to end of input or error
    
    // Check for errors
    KoiParserError* error = KoiParser_Error(parser);
    if (error != nullptr) {
        // If there's an error, test the error handling functions
        
        // Get error message length
        uintptr_t msg_len = KoiParserError_GetMessageLen(error);
        EXPECT_GT(msg_len, 0);
        
        // Get error message
        char* msg = new char[msg_len];
        uintptr_t result = KoiParserError_GetMessage(error, msg, msg_len);
        EXPECT_EQ(result, msg_len); // Returns length with null terminator
        
        // Get formatted error length
        uintptr_t fmt_len = KoiParserError_FormatLen(error);
        EXPECT_GE(fmt_len, msg_len); // Formatted error should be at least as long as message
        
        // Get formatted error
        char* fmt = new char[fmt_len];
        result = KoiParserError_Format(error, fmt, fmt_len);
        EXPECT_EQ(result, fmt_len); // Returns length with null terminator
        
        // Get traceback position
        uintptr_t lineno, column;
        int32_t pos_result = KoiParserError_GetTracebackPosition(error, &lineno, &column);
        // Position might not be available for all errors
        if (pos_result == 0) {
            EXPECT_GT(lineno, 0);
            EXPECT_GT(column, 0);
        }
        
        delete[] msg;
        delete[] fmt;
        KoiParserError_Del(error);
    }
    
    if (cmd != nullptr) {
        KoiCommand_Del(cmd);
    }
    KoiParser_Del(parser);
}

TEST(ParserErrorTest, TestErrorWithInvalidInput) {
    // Create a parser with potentially problematic input
    // Note: The exact input that causes an error depends on the parser implementation
    
    KoiInputSource* source = KoiInputSource_FromString("#\0Invalid character"); // Null byte in string
    if (source != nullptr) { // This might fail due to invalid UTF-8
        KoiParserConfig config;
        KoiParserConfig_Init(&config);
        KoiParser* parser = KoiParser_New(source, &config);
        EXPECT_NE(parser, nullptr);
        
        // Try to parse
        KoiCommand* cmd = KoiParser_NextCommand(parser);
        
        // Check for errors
        KoiParserError* error = KoiParser_Error(parser);
        if (error != nullptr) {
            // Test error handling
            uintptr_t msg_len = KoiParserError_GetMessageLen(error);
            EXPECT_GT(msg_len, 0);
            
            char* msg = new char[msg_len];
            uintptr_t result = KoiParserError_GetMessage(error, msg, msg_len);
            EXPECT_EQ(result, msg_len);
            
            delete[] msg;

            KoiParserError_Del(error);
        }
        
        if (cmd != nullptr) {
            KoiCommand_Del(cmd);
        }
        KoiParser_Del(parser);
    }
}

TEST(ParserErrorTest, TestErrorWithNullParameters) {
    // Test error handling functions with null parameters
    
    // Test with null error
    uintptr_t len = KoiParserError_GetMessageLen(nullptr);
    EXPECT_EQ(len, 0);
    
    len = KoiParserError_FormatLen(nullptr);
    EXPECT_EQ(len, 0);
    
    // Test with null buffer
    KoiInputSource* source = KoiInputSource_FromString("#");
    EXPECT_NE(source, nullptr);
    
    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    KoiParser* parser = KoiParser_New(source, &config);
    EXPECT_NE(parser, nullptr);
    
    KoiCommand* cmd = KoiParser_NextCommand(parser);
    if (cmd != nullptr) {
        KoiCommand_Del(cmd);
    }
    
    KoiParserError* error = KoiParser_Error(parser);
    if (error != nullptr) {
        // Test with null buffer
        len = KoiParserError_GetMessage(error, nullptr, 100);
        EXPECT_GT(len, 0); // Should return required size
        
        len = KoiParserError_Format(error, nullptr, 100);
        EXPECT_GT(len, 0); // Should return required size
        
        // Test with zero buffer size
        char buffer[1];
        len = KoiParserError_GetMessage(error, buffer, 0);
        EXPECT_GT(len, 0); // Should return required size
        
        len = KoiParserError_Format(error, buffer, 0);
        EXPECT_GT(len, 0); // Should return required size
        
        KoiParserError_Del(error);
    }
    
    KoiParser_Del(parser);
}

TEST(ParserErrorTest, TestErrorWithSmallBuffer) {
    // Test error handling with a buffer that's too small
    
    KoiInputSource* source = KoiInputSource_FromString("#");
    EXPECT_NE(source, nullptr);
    
    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    KoiParser* parser = KoiParser_New(source, &config);
    EXPECT_NE(parser, nullptr);
    
    KoiCommand* cmd = KoiParser_NextCommand(parser);
    if (cmd != nullptr) {
        KoiCommand_Del(cmd);
    }
    
    KoiParserError* error = KoiParser_Error(parser);
    if (error != nullptr) {
        // Get required size
        uintptr_t required_len = KoiParserError_GetMessageLen(error);
        EXPECT_GT(required_len, 0);
        
        // Try with a smaller buffer
        char small_buffer[2]; // Only space for null terminator
        uintptr_t result = KoiParserError_GetMessage(error, small_buffer, sizeof(small_buffer));
        EXPECT_EQ(result, required_len); // Should return required size, not truncate
        
        // Try with a buffer that's one byte too small
        char* buffer = new char[required_len];
        result = KoiParserError_GetMessage(error, buffer, required_len - 1);
        EXPECT_EQ(result, required_len); // Should return required size, not truncate
        
        delete[] buffer;
        
        KoiParserError_Del(error);
    }
    
    KoiParser_Del(parser);
}

int main() {
    ::testing::InitGoogleTest();
    return RUN_ALL_TESTS();
}