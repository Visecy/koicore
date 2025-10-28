#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <gtest/gtest.h>
#include "../../koicore.h"

using namespace koicore;

// Test KoiCompositeList functionality
TEST(CompositeListTest, TestCreateList) {
    // Create a new empty list
    KoiCompositeList* list = KoiCompositeList_New();
    EXPECT_NE(list, nullptr);
    
    // Check initial length
    EXPECT_EQ(KoiCompositeList_GetLength(list), 0);
    
    KoiCompositeList_Del(list);
}

TEST(CompositeListTest, TestAddValues) {
    // Create a new list
    KoiCompositeList* list = KoiCompositeList_New();
    EXPECT_NE(list, nullptr);
    
    // Add values
    EXPECT_EQ(KoiCompositeList_AddIntValue(list, 42), 0);
    EXPECT_EQ(KoiCompositeList_AddFloatValue(list, 3.14), 0);
    EXPECT_EQ(KoiCompositeList_AddStringValue(list, "test_string"), 0);
    
    // Check length
    EXPECT_EQ(KoiCompositeList_GetLength(list), 3);
    
    // Check value types
    EXPECT_EQ(KoiCompositeList_GetValueType(list, 0), 0); // Assuming 0 is Int type
    EXPECT_EQ(KoiCompositeList_GetValueType(list, 1), 1); // Assuming 1 is Float type
    EXPECT_EQ(KoiCompositeList_GetValueType(list, 2), 2); // Assuming 2 is String type
    
    // Check values
    int64_t int_value;
    EXPECT_EQ(KoiCompositeList_GetIntValue(list, 0, &int_value), 0);
    EXPECT_EQ(int_value, 42);
    
    double float_value;
    EXPECT_EQ(KoiCompositeList_GetFloatValue(list, 1, &float_value), 0);
    EXPECT_FLOAT_EQ(float_value, 3.14);
    
    char str_value[256];
    uintptr_t len = KoiCompositeList_GetStringValue(list, 2, str_value, sizeof(str_value));
    EXPECT_EQ(len, 12); // "test_string" buffer size
    EXPECT_STREQ(str_value, "test_string");
    
    KoiCompositeList_Del(list);
}

TEST(CompositeListTest, TestSetValues) {
    // Create a new list with initial values
    KoiCompositeList* list = KoiCompositeList_New();
    EXPECT_NE(list, nullptr);
    
    EXPECT_EQ(KoiCompositeList_AddIntValue(list, 100), 0);
    EXPECT_EQ(KoiCompositeList_AddFloatValue(list, 1.0), 0);
    EXPECT_EQ(KoiCompositeList_AddStringValue(list, "original"), 0);
    
    // Modify values
    EXPECT_EQ(KoiCompositeList_SetIntValue(list, 0, 200), 0);
    EXPECT_EQ(KoiCompositeList_SetFloatValue(list, 1, 2.5), 0);
    EXPECT_EQ(KoiCompositeList_SetStringValue(list, 2, "modified"), 0);
    
    // Check modified values
    int64_t int_value;
    EXPECT_EQ(KoiCompositeList_GetIntValue(list, 0, &int_value), 0);
    EXPECT_EQ(int_value, 200);
    
    double float_value;
    EXPECT_EQ(KoiCompositeList_GetFloatValue(list, 1, &float_value), 0);
    EXPECT_FLOAT_EQ(float_value, 2.5);
    
    char str_value[256];
    uintptr_t len = KoiCompositeList_GetStringValue(list, 2, str_value, sizeof(str_value));
    EXPECT_EQ(len, 9); // "modified" buffer size
    EXPECT_STREQ(str_value, "modified");
    
    KoiCompositeList_Del(list);
}

TEST(CompositeListTest, TestRemoveValues) {
    // Create a new list with values
    KoiCompositeList* list = KoiCompositeList_New();
    EXPECT_NE(list, nullptr);
    
    EXPECT_EQ(KoiCompositeList_AddIntValue(list, 1), 0);
    EXPECT_EQ(KoiCompositeList_AddIntValue(list, 2), 0);
    EXPECT_EQ(KoiCompositeList_AddIntValue(list, 3), 0);
    EXPECT_EQ(KoiCompositeList_AddIntValue(list, 4), 0);
    
    // Check initial length
    EXPECT_EQ(KoiCompositeList_GetLength(list), 4);
    
    // Remove a value
    EXPECT_EQ(KoiCompositeList_RemoveValue(list, 1), 0);
    
    // Check length
    EXPECT_EQ(KoiCompositeList_GetLength(list), 3);
    
    // Check remaining values
    int64_t int_value;
    EXPECT_EQ(KoiCompositeList_GetIntValue(list, 0, &int_value), 0);
    EXPECT_EQ(int_value, 1);
    
    EXPECT_EQ(KoiCompositeList_GetIntValue(list, 1, &int_value), 0);
    EXPECT_EQ(int_value, 3);
    
    EXPECT_EQ(KoiCompositeList_GetIntValue(list, 2, &int_value), 0);
    EXPECT_EQ(int_value, 4);
    
    KoiCompositeList_Del(list);
}

TEST(CompositeListTest, TestClearList) {
    // Create a new list with values
    KoiCompositeList* list = KoiCompositeList_New();
    EXPECT_NE(list, nullptr);
    
    EXPECT_EQ(KoiCompositeList_AddIntValue(list, 1), 0);
    EXPECT_EQ(KoiCompositeList_AddFloatValue(list, 2.0), 0);
    EXPECT_EQ(KoiCompositeList_AddStringValue(list, "test"), 0);
    
    // Check initial length
    EXPECT_EQ(KoiCompositeList_GetLength(list), 3);
    
    // Clear the list
    EXPECT_EQ(KoiCompositeList_Clear(list), 0);
    
    // Check length
    EXPECT_EQ(KoiCompositeList_GetLength(list), 0);
    
    KoiCompositeList_Del(list);
}

TEST(CompositeListTest, TestListInCommand) {
    // Create a command
    KoiCommand* cmd = KoiCommand_New("list_test");
    EXPECT_NE(cmd, nullptr);
    
    // Create a list
    KoiCompositeList* list = KoiCompositeList_New();
    EXPECT_NE(list, nullptr);
    
    // Add values to the list
    EXPECT_EQ(KoiCompositeList_AddIntValue(list, 42), 0);
    EXPECT_EQ(KoiCompositeList_AddStringValue(list, "hello"), 0);
    
    // Note: There's no direct API to add a list to a command in the header
    // This test demonstrates how to work with a list that might be part of a command
    
    // Clean up
    KoiCompositeList_Del(list);
    KoiCommand_Del(cmd);
}

TEST(CompositeListTest, TestStringLen) {
    // Create a new list
    KoiCompositeList* list = KoiCompositeList_New();
    EXPECT_NE(list, nullptr);
    
    // Add a string value
    EXPECT_EQ(KoiCompositeList_AddStringValue(list, "test_string_length"), 0);
    
    // Get string length
    uintptr_t len = KoiCompositeList_GetStringValueLen(list, 0);
    EXPECT_EQ(len, 19); // "test_string_length" buffer size including null terminator
    
    // Get string with exact buffer size
    char* buffer = new char[len];
    uintptr_t result = KoiCompositeList_GetStringValue(list, 0, buffer, len);
    EXPECT_EQ(result, len); // Returns length with null terminator
    EXPECT_STREQ(buffer, "test_string_length");
    
    delete[] buffer;
    KoiCompositeList_Del(list);
}

int main() {
    ::testing::InitGoogleTest();
    return RUN_ALL_TESTS();
}