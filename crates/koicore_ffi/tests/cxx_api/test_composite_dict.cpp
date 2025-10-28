#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <gtest/gtest.h>
#include "../../koicore.h"

using namespace koicore;

// Test KoiCompositeDict functionality
TEST(CompositeDictTest, TestCreateDict) {
    // Create a new empty dict
    KoiCompositeDict* dict = KoiCompositeDict_New();
    EXPECT_NE(dict, nullptr);
    
    // Check initial length
    EXPECT_EQ(KoiCompositeDict_GetLength(dict), 0);
    
    KoiCompositeDict_Del(dict);
}

TEST(CompositeDictTest, TestSetValues) {
    // Create a new dict
    KoiCompositeDict* dict = KoiCompositeDict_New();
    EXPECT_NE(dict, nullptr);
    
    // Set values
    EXPECT_EQ(KoiCompositeDict_SetIntValue(dict, "int_key", 42), 0);
    EXPECT_EQ(KoiCompositeDict_SetFloatValue(dict, "float_key", 3.14), 0);
    EXPECT_EQ(KoiCompositeDict_SetStringValue(dict, "string_key", "test_string"), 0);
    
    // Check length
    EXPECT_EQ(KoiCompositeDict_GetLength(dict), 3);
    
    // Check value types
    EXPECT_EQ(KoiCompositeDict_GetValueType(dict, "int_key"), 0); // Assuming 0 is Int type
    EXPECT_EQ(KoiCompositeDict_GetValueType(dict, "float_key"), 1); // Assuming 1 is Float type
    EXPECT_EQ(KoiCompositeDict_GetValueType(dict, "string_key"), 2); // Assuming 2 is String type
    
    // Check values
    int64_t int_value;
    EXPECT_EQ(KoiCompositeDict_GetIntValue(dict, "int_key", &int_value), 0);
    EXPECT_EQ(int_value, 42);
    
    double float_value;
    EXPECT_EQ(KoiCompositeDict_GetFloatValue(dict, "float_key", &float_value), 0);
    EXPECT_FLOAT_EQ(float_value, 3.14);
    
    char str_value[256];
    uintptr_t len = KoiCompositeDict_GetStringValue(dict, "string_key", str_value, sizeof(str_value));
    EXPECT_EQ(len, 12); // "test_string" buffer size
    EXPECT_STREQ(str_value, "test_string");
    
    KoiCompositeDict_Del(dict);
}

TEST(CompositeDictTest, TestModifyValues) {
    // Create a new dict with initial values
    KoiCompositeDict* dict = KoiCompositeDict_New();
    EXPECT_NE(dict, nullptr);
    
    EXPECT_EQ(KoiCompositeDict_SetIntValue(dict, "int_key", 100), 0);
    EXPECT_EQ(KoiCompositeDict_SetFloatValue(dict, "float_key", 1.0), 0);
    EXPECT_EQ(KoiCompositeDict_SetStringValue(dict, "string_key", "original"), 0);
    
    // Modify values
    EXPECT_EQ(KoiCompositeDict_SetIntValue(dict, "int_key", 200), 0);
    EXPECT_EQ(KoiCompositeDict_SetFloatValue(dict, "float_key", 2.5), 0);
    EXPECT_EQ(KoiCompositeDict_SetStringValue(dict, "string_key", "modified"), 0);
    
    // Check modified values
    int64_t int_value;
    EXPECT_EQ(KoiCompositeDict_GetIntValue(dict, "int_key", &int_value), 0);
    EXPECT_EQ(int_value, 200);
    
    double float_value;
    EXPECT_EQ(KoiCompositeDict_GetFloatValue(dict, "float_key", &float_value), 0);
    EXPECT_FLOAT_EQ(float_value, 2.5);
    
    char str_value[256];
    uintptr_t len = KoiCompositeDict_GetStringValue(dict, "string_key", str_value, sizeof(str_value));
    EXPECT_EQ(len, 9); // "modified" buffer size
    EXPECT_STREQ(str_value, "modified");
    
    KoiCompositeDict_Del(dict);
}

TEST(CompositeDictTest, TestRemoveEntries) {
    // Create a new dict with values
    KoiCompositeDict* dict = KoiCompositeDict_New();
    EXPECT_NE(dict, nullptr);
    
    EXPECT_EQ(KoiCompositeDict_SetIntValue(dict, "key1", 1), 0);
    EXPECT_EQ(KoiCompositeDict_SetIntValue(dict, "key2", 2), 0);
    EXPECT_EQ(KoiCompositeDict_SetIntValue(dict, "key3", 3), 0);
    
    // Check initial length
    EXPECT_EQ(KoiCompositeDict_GetLength(dict), 3);
    
    // Remove an entry
    EXPECT_EQ(KoiCompositeDict_Remove(dict, "key2"), 0);
    
    // Check length
    EXPECT_EQ(KoiCompositeDict_GetLength(dict), 2);
    
    // Check remaining keys
    EXPECT_EQ(KoiCompositeDict_GetValueType(dict, "key1"), 0); // Assuming 0 is Int type
    EXPECT_EQ(KoiCompositeDict_GetValueType(dict, "key3"), 0); // Assuming 0 is Int type
    
    // Check that removed key no longer exists
    EXPECT_EQ(KoiCompositeDict_GetValueType(dict, "key2"), -1); // Assuming -1 is Invalid type
    
    KoiCompositeDict_Del(dict);
}

TEST(CompositeDictTest, TestClearDict) {
    // Create a new dict with values
    KoiCompositeDict* dict = KoiCompositeDict_New();
    EXPECT_NE(dict, nullptr);
    
    EXPECT_EQ(KoiCompositeDict_SetIntValue(dict, "key1", 1), 0);
    EXPECT_EQ(KoiCompositeDict_SetFloatValue(dict, "key2", 2.0), 0);
    EXPECT_EQ(KoiCompositeDict_SetStringValue(dict, "key3", "test"), 0);
    
    // Check initial length
    EXPECT_EQ(KoiCompositeDict_GetLength(dict), 3);
    
    // Clear the dict
    EXPECT_EQ(KoiCompositeDict_Clear(dict), 0);
    
    // Check length
    EXPECT_EQ(KoiCompositeDict_GetLength(dict), 0);
    
    KoiCompositeDict_Del(dict);
}

TEST(CompositeDictTest, TestDictIteration) {
    // Create a new dict with values
    KoiCompositeDict* dict = KoiCompositeDict_New();
    EXPECT_NE(dict, nullptr);
    
    EXPECT_EQ(KoiCompositeDict_SetIntValue(dict, "key1", 1), 0);
    EXPECT_EQ(KoiCompositeDict_SetFloatValue(dict, "key2", 2.0), 0);
    EXPECT_EQ(KoiCompositeDict_SetStringValue(dict, "key3", "test"), 0);
    
    // Check length
    EXPECT_EQ(KoiCompositeDict_GetLength(dict), 3);
    
    // Iterate through keys by index
    for (uintptr_t i = 0; i < KoiCompositeDict_GetLength(dict); i++) {
        // Get key
        char key[256];
        uintptr_t key_len = KoiCompositeDict_GetKeybyIndex(dict, i, key, sizeof(key));
        EXPECT_GT(key_len, 0);
        
        // Get value type
        int32_t value_type = KoiCompositeDict_GetValueTypeByIndex(dict, i);
        EXPECT_GE(value_type, 0); // Should be a valid type
        
        // Check that the key exists when accessed by name
        int32_t type_by_name = KoiCompositeDict_GetValueType(dict, key);
        EXPECT_EQ(value_type, type_by_name);
    }
    
    KoiCompositeDict_Del(dict);
}

TEST(CompositeDictTest, TestKeyLength) {
    // Create a new dict
    KoiCompositeDict* dict = KoiCompositeDict_New();
    EXPECT_NE(dict, nullptr);
    
    // Set a value with a long key
    EXPECT_EQ(KoiCompositeDict_SetIntValue(dict, "very_long_key_name_for_testing", 42), 0);
    
    // Get key length
    uintptr_t len = KoiCompositeDict_GetKeyLenByIndex(dict, 0);
    EXPECT_EQ(len, 31); // "very_long_key_name_for_testing" buffer size including null terminator
    
    // Get key with exact buffer size
    char* buffer = new char[len];
    uintptr_t result = KoiCompositeDict_GetKeybyIndex(dict, 0, buffer, len);
    EXPECT_EQ(result, len); // Returns length with null terminator
    EXPECT_STREQ(buffer, "very_long_key_name_for_testing");
    
    delete[] buffer;
    KoiCompositeDict_Del(dict);
}

TEST(CompositeDictTest, TestStringValueLength) {
    // Create a new dict
    KoiCompositeDict* dict = KoiCompositeDict_New();
    EXPECT_NE(dict, nullptr);
    
    // Set a string value
    EXPECT_EQ(KoiCompositeDict_SetStringValue(dict, "key", "test_string_length"), 0);
    
    // Get string length
    uintptr_t len = KoiCompositeDict_GetStringValueLen(dict, "key");
    EXPECT_EQ(len, 19); // "test_string_length" buffer size including null terminator
    
    // Get string with exact buffer size
    char* buffer = new char[len];
    uintptr_t result = KoiCompositeDict_GetStringValue(dict, "key", buffer, len);
    EXPECT_EQ(result, len); // Returns length with null terminator
    EXPECT_STREQ(buffer, "test_string_length");
    
    delete[] buffer;
    KoiCompositeDict_Del(dict);
}

TEST(CompositeDictTest, TestDictInCommand) {
    // Create a command
    KoiCommand* cmd = KoiCommand_New("dict_test");
    EXPECT_NE(cmd, nullptr);
    
    // Create a dict
    KoiCompositeDict* dict = KoiCompositeDict_New();
    EXPECT_NE(dict, nullptr);
    
    // Add values to the dict
    EXPECT_EQ(KoiCompositeDict_SetIntValue(dict, "int_key", 42), 0);
    EXPECT_EQ(KoiCompositeDict_SetStringValue(dict, "string_key", "hello"), 0);
    
    // Note: There's no direct API to add a dict to a command in the header
    // This test demonstrates how to work with a dict that might be part of a command
    
    // Clean up
    KoiCompositeDict_Del(dict);
    KoiCommand_Del(cmd);
}

int main() {
    ::testing::InitGoogleTest();
    return RUN_ALL_TESTS();
}