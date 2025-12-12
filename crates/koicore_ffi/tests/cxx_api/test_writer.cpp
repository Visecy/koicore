#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <gtest/gtest.h>
#include "../../koicore.h"

using namespace koicore;

TEST(WriterTest, TestStringOutput) {
    // Create string output
    KoiStringOutput* output = KoiStringOutput_New();
    EXPECT_NE(output, nullptr);

    // Initial check: empty string has length 0, but requires buffer size 1 (null terminator)
    // GetString(NULL, 0) returns required buffer size: 1.
    EXPECT_EQ(KoiStringOutput_GetString(output, nullptr, 0), 1);
    
    // Clean up
    KoiStringOutput_Del(output);
}

TEST(WriterTest, TestWriteSimpleCommand) {
    KoiStringOutput* output = KoiStringOutput_New();
    
    KoiFormatterOptions opts;
    KoiFormatterOptions_Init(&opts);
    
    // Customize after init
    opts.indent = 4;
    opts.number_format = Decimal;
    
    KoiWriterConfig config;
    KoiWriterConfig_Init(&config);
    config.global_options = opts;
    
    KoiWriter* writer = KoiWriter_NewFromStringOutput(output, &config);
    EXPECT_NE(writer, nullptr);
    
    // Create command #test "hello"
    KoiCommand* cmd = KoiCommand_New("test");
    // Use string with spaces to force quotes if logic depends on it
    KoiCommand_AddStringParameter(cmd, "hello world");
    
    // Write
    EXPECT_EQ(KoiWriter_WriteCommand(writer, cmd), 0);
    
    // Check output
    uintptr_t len = KoiStringOutput_GetString(output, nullptr, 0);
    EXPECT_GT(len, 0);
    
    char* buffer = new char[len + 1];
    KoiStringOutput_GetString(output, buffer, len + 1);
    
    // Expect quotes for string with spaces
    EXPECT_STREQ(buffer, "#test \"hello world\"\n");
    
    delete[] buffer;
    KoiCommand_Del(cmd);
    KoiWriter_Del(writer);
    KoiStringOutput_Del(output);
}

TEST(WriterTest, TestCustomOptions) {
    KoiStringOutput* output = KoiStringOutput_New();
    
    KoiFormatterOptions opts;
    KoiFormatterOptions_Init(&opts); // global defaults
    // we want to test that overriding works, so keep global defaults (indent 4 typically)
    // but lets make sure global is DIFFERENT from override
    opts.indent = 2; 
    
    KoiWriterConfig config;
    KoiWriterConfig_Init(&config);
    config.global_options = opts;
    
    KoiWriter* writer = KoiWriter_NewFromStringOutput(output, &config);
    
    KoiCommand* cmd = KoiCommand_New("test");
    
    // Override options
    KoiFormatterOptions override_opts;
    KoiFormatterOptions_Init(&override_opts);
    override_opts.indent = 4;
    override_opts.compact = false; 
    
    KoiWriter_WriteCommandWithOptions(writer, cmd, &override_opts, nullptr);
    
    uintptr_t len = KoiStringOutput_GetString(output, nullptr, 0);
    char* buffer = new char[len + 1];
    KoiStringOutput_GetString(output, buffer, len + 1);
    
    EXPECT_STREQ(buffer, "#test\n"); 
    
    delete[] buffer;
    KoiCommand_Del(cmd);
    KoiWriter_Del(writer);
    KoiStringOutput_Del(output);
}

TEST(WriterTest, TestCommandOptions) {
    KoiStringOutput* output = KoiStringOutput_New();
    
    KoiFormatterOptions global_opts;
    KoiFormatterOptions_Init(&global_opts);
    global_opts.indent = 2;

    // Command specific options for "test1"
    KoiFormatterOptions test1_opts;
    KoiFormatterOptions_Init(&test1_opts);
    test1_opts.indent = 2;
    test1_opts.newline_after = true;
    
    // Command specific options for "test2"
    KoiFormatterOptions test2_opts;
    KoiFormatterOptions_Init(&test2_opts);
    test2_opts.indent = 8;
    test2_opts.force_quotes_for_vars = true;
    
    KoiCommandOption cmd_opts[] = {
        { "test1", test1_opts },
        { "test2", test2_opts },
        { nullptr, {0} } // Terminator
    };
    
    KoiWriterConfig config;
    KoiWriterConfig_Init(&config);
    config.global_options = global_opts;
    config.command_options = cmd_opts;
    
    KoiWriter* writer = KoiWriter_NewFromStringOutput(output, &config);
    
    // Write test1 command
    KoiCommand* test1 = KoiCommand_New("test1");
    KoiCommand_AddStringParameter(test1, "regular");
    KoiWriter_WriteCommand(writer, test1);
    
    // Write test2 command
    KoiCommand* test2 = KoiCommand_New("test2");
    KoiCommand_AddStringParameter(test2, "regular");

    KoiFormatterOptions param_opts;
    KoiFormatterOptions_Init(&param_opts);
    param_opts.indent = 2;
    param_opts.force_quotes_for_vars = true;
    param_opts.newline_before_param = true;
    KoiWriter_WriteCommandWithOptions(writer, test2, &param_opts, nullptr);
    
    // Verify
    uintptr_t len = KoiStringOutput_GetString(output, nullptr, 0);
    char* buffer = new char[len + 1];
    KoiStringOutput_GetString(output, buffer, len + 1);
    
    EXPECT_STREQ(buffer, "#test1 regular\n\n#test2\n  \"regular\"\n");
    
    delete[] buffer;
    KoiCommand_Del(test1);
    KoiCommand_Del(test2);
    KoiWriter_Del(writer);
    KoiStringOutput_Del(output);
}

TEST(WriterTest, TestParamOptions) {
    KoiStringOutput* output = KoiStringOutput_New();
    
    KoiWriterConfig config;
    KoiWriterConfig_Init(&config);
    // defaults: threshold 1
    
    KoiWriter* writer = KoiWriter_NewFromStringOutput(output, &config);
    
    KoiCommand* cmd = KoiCommand_New("param_test");
    KoiCommand_AddIntParameter(cmd, 255); // Pos 0
    KoiCommand_AddIntParameter(cmd, 10);  // Pos 1
    
    // Override pos 0 to Hex
    KoiFormatterOptions hex_opts;
    KoiFormatterOptions_Init(&hex_opts);
    hex_opts.number_format = Hex;
    
    KoiParamOption param_opts[] = {
        { { true, 0, nullptr }, hex_opts },
        { { false, 0, nullptr }, {0} } // Terminator: name=NULL
    };
    
    KoiWriter_WriteCommandWithOptions(writer, cmd, nullptr, param_opts);
    
    uintptr_t len = KoiStringOutput_GetString(output, nullptr, 0);
    char* buffer = new char[len + 1];
    KoiStringOutput_GetString(output, buffer, len + 1);
    
    EXPECT_STREQ(buffer, "#param_test 0xff 10\n");
    
    delete[] buffer;
    KoiCommand_Del(cmd);
    KoiWriter_Del(writer);
    KoiStringOutput_Del(output);
}

int main() {
    ::testing::InitGoogleTest();
    return RUN_ALL_TESTS();
}
