#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <gtest/gtest.h>
#include "../../koicore.h"

using namespace koicore;

// Test KoiInputSource functionality
TEST(InputSourceTest, TestStringInputSource) {
    // Test creating an input source from a string
    const char* test_string = "#test command";
    KoiInputSource* source = KoiInputSource_FromString(test_string);
    EXPECT_NE(source, nullptr);
    
    // Create a parser with this input source
    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    KoiParser* parser = KoiParser_New(source, &config);
    EXPECT_NE(parser, nullptr);
    
    // Try to parse the command
    KoiCommand* cmd = KoiParser_NextCommand(parser);
    EXPECT_NE(cmd, nullptr);
    
    // Check the command name
    uintptr_t name_len = KoiCommand_GetNameLen(cmd);
    EXPECT_GT(name_len, 0);
    
    char* name = new char[name_len];
    uintptr_t result = KoiCommand_GetName(cmd, name, name_len);
    EXPECT_EQ(result, name_len);
    EXPECT_STREQ(name, "test");
    
    delete[] name;
    KoiCommand_Del(cmd);
    KoiParser_Del(parser);
}

TEST(InputSourceTest, TestFileInputSource) {
    // Create a temporary file for testing
    FILE* temp_file = tmpfile();
    EXPECT_NE(temp_file, nullptr);
    
    // Write test content to the file
    const char* test_content = "#file command\n";
    fprintf(temp_file, "%s", test_content);
    fflush(temp_file);
    
    // Get the file path
    rewind(temp_file);
    char file_path[L_tmpnam];
    tmpnam(file_path);
    fclose(temp_file);
    
    // Reopen the file with the temporary name
    temp_file = fopen(file_path, "w");
    fprintf(temp_file, "%s", test_content);
    fclose(temp_file);
    
    // Test creating an input source from a file path
    KoiInputSource* source = KoiInputSource_FromFile(file_path);
    EXPECT_NE(source, nullptr);
    
    // Create a parser with this input source
    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    KoiParser* parser = KoiParser_New(source, &config);
    EXPECT_NE(parser, nullptr);
    
    // Try to parse the command
    KoiCommand* cmd = KoiParser_NextCommand(parser);
    EXPECT_NE(cmd, nullptr);
    
    // Check the command name
    uintptr_t name_len = KoiCommand_GetNameLen(cmd);
    EXPECT_GT(name_len, 0);
    
    char* name = new char[name_len];
    uintptr_t result = KoiCommand_GetName(cmd, name, name_len);
    EXPECT_EQ(result, name_len);
    EXPECT_STREQ(name, "file");
    
    delete[] name;
    KoiCommand_Del(cmd);
    KoiParser_Del(parser);
    
    // Clean up the temporary file
    remove(file_path);
}

TEST(InputSourceTest, TestFileInputSourceWithDifferentEncodings) {
    // Test with different encoding strategies
    KoiInputSource* source_utf8 = KoiInputSource_FromFileAndEncoding("../../examples/ktxt/example0.ktxt", "utf-8", KoiFileInputEncodingStrategy::Strict);
    EXPECT_NE(source_utf8, nullptr);
    
    KoiInputSource* source_utf16 = KoiInputSource_FromFileAndEncoding("../../examples/ktxt/example0_utf16.ktxt", "utf-16", KoiFileInputEncodingStrategy::Strict);
    EXPECT_NE(source_utf16, nullptr);

    KoiInputSource* source_gbk = KoiInputSource_FromFileAndEncoding("../../examples/ktxt/example0_gbk.ktxt", "gbk", KoiFileInputEncodingStrategy::Strict);
    EXPECT_NE(source_gbk, nullptr);
    
    // Test with each encoding
    KoiInputSource* sources[] = {source_utf8, source_utf16, source_gbk};
    for (int i = 0; i < 3; i++) {
        KoiParserConfig config;
        KoiParserConfig_Init(&config);
        KoiParser* parser = KoiParser_New(sources[i], &config);
        EXPECT_NE(parser, nullptr);
        
        KoiCommand* cmd = KoiParser_NextCommand(parser);
        if (cmd != nullptr) {
            uintptr_t name_len = KoiCommand_GetNameLen(cmd);
            EXPECT_GT(name_len, 0);
            
            char* name = new char[name_len];
            uintptr_t result = KoiCommand_GetName(cmd, name, name_len);
            EXPECT_EQ(result, name_len);
            
            delete[] name;
            KoiCommand_Del(cmd);
        }
        
        KoiParser_Del(parser);
    }
}

TEST(InputSourceTest, TestEmptyInputSource) {
    // Test with an empty string
    KoiInputSource* source = KoiInputSource_FromString("");
    EXPECT_NE(source, nullptr);
    
    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    KoiParser* parser = KoiParser_New(source, &config);
    EXPECT_NE(parser, nullptr);
    
    // Should return null for empty input
    KoiCommand* cmd = KoiParser_NextCommand(parser);
    EXPECT_EQ(cmd, nullptr);
    
    KoiParser_Del(parser);
}

TEST(InputSourceTest, TestInvalidFileInputSource) {
    // Test with a non-existent file
    KoiInputSource* source = KoiInputSource_FromFileAndEncoding("/non/existent/file.koi", "utf-8", KoiFileInputEncodingStrategy::Strict);
    // This might return null or a source that produces an error when used
    // The exact behavior depends on the implementation
    
    if (source != nullptr) {
        KoiParserConfig config;
        KoiParserConfig_Init(&config);
        KoiParser* parser = KoiParser_New(source, &config);
        EXPECT_NE(parser, nullptr);
        
        KoiCommand* cmd = KoiParser_NextCommand(parser);
        EXPECT_EQ(cmd, nullptr);
        
        if (cmd != nullptr) {
            KoiCommand_Del(cmd);
        }
        
        KoiParser_Del(parser);
    }
}

TEST(InputSourceTest, TestMultipleCommandsFromInput) {
    // Test parsing multiple commands from a single input source
    const char* test_string = "#command1\n#command2\n#command3";
    KoiInputSource* source = KoiInputSource_FromString(test_string);
    EXPECT_NE(source, nullptr);
    
    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    KoiParser* parser = KoiParser_New(source, &config);
    EXPECT_NE(parser, nullptr);
    
    // Parse all commands
    KoiCommand* cmd1 = KoiParser_NextCommand(parser);
    EXPECT_NE(cmd1, nullptr);
    
    KoiCommand* cmd2 = KoiParser_NextCommand(parser);
    EXPECT_NE(cmd2, nullptr);
    
    KoiCommand* cmd3 = KoiParser_NextCommand(parser);
    EXPECT_NE(cmd3, nullptr);
    
    // Check that we got all commands
    uintptr_t name_len;
    char* name;
    
    // First command
    name_len = KoiCommand_GetNameLen(cmd1);
    name = new char[name_len];
    KoiCommand_GetName(cmd1, name, name_len);
    EXPECT_STREQ(name, "command1");
    delete[] name;
    
    // Second command
    name_len = KoiCommand_GetNameLen(cmd2);
    name = new char[name_len];
    KoiCommand_GetName(cmd2, name, name_len);
    EXPECT_STREQ(name, "command2");
    delete[] name;
    
    // Third command
    name_len = KoiCommand_GetNameLen(cmd3);
    name = new char[name_len];
    KoiCommand_GetName(cmd3, name, name_len);
    EXPECT_STREQ(name, "command3");
    delete[] name;
    
    // Should return null when no more commands are available
    KoiCommand* cmd4 = KoiParser_NextCommand(parser);
    EXPECT_EQ(cmd4, nullptr);
    
    KoiCommand_Del(cmd1);
    KoiCommand_Del(cmd2);
    KoiCommand_Del(cmd3);
    KoiParser_Del(parser);
}

int main() {
    ::testing::InitGoogleTest();
    return RUN_ALL_TESTS();
}