#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <gtest/gtest.h>
#include "../../koicore.h"

using namespace koicore;

// Test KoiCommand creation and manipulation
TEST(CommandTest, TestCreateCommand) {
    // Create a new command with name
    KoiCommand* cmd = KoiCommand_New("test_command");
    EXPECT_NE(cmd, nullptr);
    
    // Check command name
    char name[256];
    uintptr_t len = KoiCommand_GetName(cmd, name, sizeof(name));
    EXPECT_EQ(len, 13); // "test_command" buffer size
    EXPECT_STREQ(name, "test_command");
    
    // Check it's not a special command
    EXPECT_EQ(KoiCommand_IsTextCommand(cmd), 0);
    EXPECT_EQ(KoiCommand_IsAnnotationCommand(cmd), 0);
    EXPECT_EQ(KoiCommand_IsNumberCommand(cmd), 0);
    
    // Check parameter count
    EXPECT_EQ(KoiCommand_GetParamCount(cmd), 0);
    
    KoiCommand_Del(cmd);
}

TEST(CommandTest, TestCreateTextCommand) {
    // Create a text command
    KoiCommand* cmd = KoiCommand_NewText("Hello, world!");
    EXPECT_NE(cmd, nullptr);
    
    // Check it's a text command
    EXPECT_EQ(KoiCommand_IsTextCommand(cmd), 1);
    EXPECT_EQ(KoiCommand_IsAnnotationCommand(cmd), 0);
    EXPECT_EQ(KoiCommand_IsNumberCommand(cmd), 0);
    
    // Check parameter count
    EXPECT_EQ(KoiCommand_GetParamCount(cmd), 1);
    
    // Check parameter value
    char content[256];
    uintptr_t len = KoiCommand_GetStringParam(cmd, 0, content, sizeof(content));
    EXPECT_EQ(len, 14); // "Hello, world!" buffer size
    EXPECT_STREQ(content, "Hello, world!");
    
    KoiCommand_Del(cmd);
}

TEST(CommandTest, TestCreateAnnotationCommand) {
    // Create an annotation command
    KoiCommand* cmd = KoiCommand_NewAnnotation("##Note");
    EXPECT_NE(cmd, nullptr);
    
    // Check it's an annotation command
    EXPECT_EQ(KoiCommand_IsTextCommand(cmd), 0);
    EXPECT_EQ(KoiCommand_IsAnnotationCommand(cmd), 1);
    EXPECT_EQ(KoiCommand_IsNumberCommand(cmd), 0);
    
    // Check parameter count
    EXPECT_EQ(KoiCommand_GetParamCount(cmd), 1);
    
    // Check parameter value
    char content[256];
    uintptr_t len = KoiCommand_GetStringParam(cmd, 0, content, sizeof(content));
    EXPECT_EQ(len, 7); // "##Note" buffer size
    EXPECT_STREQ(content, "##Note");
    
    KoiCommand_Del(cmd);
}

TEST(CommandTest, TestCreateNumberCommand) {
    // Create a number command
    KoiCommand* cmd = KoiCommand_NewNumber(42);
    EXPECT_NE(cmd, nullptr);
    
    // Check it's a number command
    EXPECT_EQ(KoiCommand_IsTextCommand(cmd), 0);
    EXPECT_EQ(KoiCommand_IsAnnotationCommand(cmd), 0);
    EXPECT_EQ(KoiCommand_IsNumberCommand(cmd), 1);
    
    // Check parameter count
    EXPECT_EQ(KoiCommand_GetParamCount(cmd), 1);
    
    // Check parameter value
    int64_t value;
    EXPECT_EQ(KoiCommand_GetIntParam(cmd, 0, &value), 0);
    EXPECT_EQ(value, 42);
    
    KoiCommand_Del(cmd);
}

TEST(CommandTest, TestCommandParameters) {
    // Create a new command
    KoiCommand* cmd = KoiCommand_New("param_test");
    EXPECT_NE(cmd, nullptr);
    
    // Add parameters
    EXPECT_EQ(KoiCommand_AddIntParameter(cmd, 123), 0);
    EXPECT_EQ(KoiCommand_AddFloatParameter(cmd, 3.14), 0);
    EXPECT_EQ(KoiCommand_AddStringParameter(cmd, "test_string"), 0);
    
    // Check parameter count
    EXPECT_EQ(KoiCommand_GetParamCount(cmd), 3);
    
    // Check parameter values
    int64_t int_value;
    EXPECT_EQ(KoiCommand_GetIntParam(cmd, 0, &int_value), 0);
    EXPECT_EQ(int_value, 123);
    
    double float_value;
    EXPECT_EQ(KoiCommand_GetFloatParam(cmd, 1, &float_value), 0);
    EXPECT_FLOAT_EQ(float_value, 3.14);
    
    char str_value[256];
    uintptr_t len = KoiCommand_GetStringParam(cmd, 2, str_value, sizeof(str_value));
    EXPECT_EQ(len, 12); // "test_string" buffer size
    EXPECT_STREQ(str_value, "test_string");
    
    // Modify parameters
    EXPECT_EQ(KoiCommand_SetIntParameter(cmd, 0, 456), 0);
    EXPECT_EQ(KoiCommand_SetFloatParameter(cmd, 1, 2.71), 0);
    EXPECT_EQ(KoiCommand_SetStringParameter(cmd, 2, "modified_string"), 0);
    
    // Check modified values
    EXPECT_EQ(KoiCommand_GetIntParam(cmd, 0, &int_value), 0);
    EXPECT_EQ(int_value, 456);
    
    EXPECT_EQ(KoiCommand_GetFloatParam(cmd, 1, &float_value), 0);
    EXPECT_FLOAT_EQ(float_value, 2.71);
    
    len = KoiCommand_GetStringParam(cmd, 2, str_value, sizeof(str_value));
    EXPECT_EQ(len, 16); // "modified_string" buffer size
    EXPECT_STREQ(str_value, "modified_string");
    
    // Remove a parameter
    EXPECT_EQ(KoiCommand_RemoveParameter(cmd, 1), 0);
    EXPECT_EQ(KoiCommand_GetParamCount(cmd), 2);
    
    // Check remaining parameters
    EXPECT_EQ(KoiCommand_GetIntParam(cmd, 0, &int_value), 0);
    EXPECT_EQ(int_value, 456);
    
    len = KoiCommand_GetStringParam(cmd, 1, str_value, sizeof(str_value));
    EXPECT_EQ(len, 16); // "modified_string" buffer size
    EXPECT_STREQ(str_value, "modified_string");
    
    // Clear all parameters
    EXPECT_EQ(KoiCommand_ClearParameters(cmd), 0);
    EXPECT_EQ(KoiCommand_GetParamCount(cmd), 0);
    
    KoiCommand_Del(cmd);
}

TEST(CommandTest, TestCommandClone) {
    // Create a command with parameters
    KoiCommand* cmd = KoiCommand_New("clone_test");
    EXPECT_NE(cmd, nullptr);
    
    EXPECT_EQ(KoiCommand_AddIntParameter(cmd, 123), 0);
    EXPECT_EQ(KoiCommand_AddStringParameter(cmd, "test_string"), 0);
    
    // Clone the command
    KoiCommand* cloned = KoiCommand_Clone(cmd);
    EXPECT_NE(cloned, nullptr);
    
    // Check they are equal
    EXPECT_EQ(KoiCommand_Compare(cmd, cloned), 1);
    
    // Modify the original
    EXPECT_EQ(KoiCommand_SetIntParameter(cmd, 0, 456), 0);
    
    // They should no longer be equal
    EXPECT_EQ(KoiCommand_Compare(cmd, cloned), 0);
    
    // Clean up
    KoiCommand_Del(cmd);
    KoiCommand_Del(cloned);
}

TEST(CommandTest, TestCommandName) {
    // Create a command
    KoiCommand* cmd = KoiCommand_New("original_name");
    EXPECT_NE(cmd, nullptr);
    
    // Check original name
    char name[256];
    uintptr_t len = KoiCommand_GetName(cmd, name, sizeof(name));
    EXPECT_EQ(len, 14); // "original_name" buffer size
    EXPECT_STREQ(name, "original_name");
    
    // Change the name
    EXPECT_EQ(KoiCommand_SetName(cmd, "new_name"), 0);
    
    // Check new name
    len = KoiCommand_GetName(cmd, name, sizeof(name));
    EXPECT_EQ(len, 9); // "new_name" buffer size
    EXPECT_STREQ(name, "new_name");
    
    KoiCommand_Del(cmd);
}

int main() {
    ::testing::InitGoogleTest();
    return RUN_ALL_TESTS();
}