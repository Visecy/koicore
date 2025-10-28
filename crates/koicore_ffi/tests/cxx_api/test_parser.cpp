#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <gtest/gtest.h>
#include "../../koicore.h"

using namespace koicore;

TEST(ParserTest, TestStringInputSource) {
    KoiInputSource* source = KoiInputSource_FromString("Hello, world!");
    EXPECT_NE(source, nullptr);

    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    KoiParser* parser = KoiParser_New(source, &config);
    EXPECT_NE(parser, nullptr);

    KoiParser_Del(parser);
}

TEST(ParserTest, TestParseTextCommand) {
    KoiInputSource* source = KoiInputSource_FromString("Hello");
    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    KoiParser* parser = KoiParser_New(source, &config);
    EXPECT_NE(parser, nullptr);

    KoiCommand* cmd = KoiParser_NextCommand(parser);
    EXPECT_NE(cmd, nullptr);

    // Check command name
    char name[256];
    uintptr_t len = KoiCommand_GetName(cmd, name, sizeof(name));
    EXPECT_EQ(len, 6); // "@text" buffer size
    EXPECT_STREQ(name, "@text");

    // Check it&#39;s a text command
    EXPECT_EQ(KoiCommand_IsTextCommand(cmd), 1);

    // Check parameter count
    EXPECT_EQ(KoiCommand_GetParamCount(cmd), 1);

    // Check parameter value
    char content[256];
    len = KoiCommand_GetStringParam(cmd, 0, content, sizeof(content));
    EXPECT_EQ(len, 6); // "Hello" buffer size
    EXPECT_STREQ(content, "Hello");

    KoiCommand_Del(cmd);
    KoiParser_Del(parser);
}

TEST(ParserTest, TestParseAnnotationCommand) {
    KoiInputSource* source = KoiInputSource_FromString("##Note");
    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    KoiParser* parser = KoiParser_New(source, &config);
    EXPECT_NE(parser, nullptr);

    KoiCommand* cmd = KoiParser_NextCommand(parser);
    EXPECT_NE(cmd, nullptr);

    // Check command name
    char name[256];
    uintptr_t len = KoiCommand_GetName(cmd, name, sizeof(name));
    EXPECT_EQ(len, 12); // "annotation" buffer size
    EXPECT_STREQ(name, "@annotation");

    // Check it&#39;s an annotation command
    EXPECT_EQ(KoiCommand_IsAnnotationCommand(cmd), 1);

    // Check parameter value
    char content[256];
    len = KoiCommand_GetStringParam(cmd, 0, content, sizeof(content));
    EXPECT_EQ(len, 7); // "##Note" buffer size
    EXPECT_STREQ(content, "##Note");

    KoiCommand_Del(cmd);
    KoiParser_Del(parser);
}

TEST(ParserTest, TestParseNumberCommand) {
    KoiInputSource* source = KoiInputSource_FromString("#42");
    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    KoiParser* parser = KoiParser_New(source, &config);
    EXPECT_NE(parser, nullptr);

    KoiCommand* cmd = KoiParser_NextCommand(parser);
    EXPECT_NE(cmd, nullptr);

    // Check command name
    char name[256];
    uintptr_t len = KoiCommand_GetName(cmd, name, sizeof(name));
    EXPECT_EQ(len, 8); // "@number" buffer size
    EXPECT_STREQ(name, "@number");

    // Check it&#39;s a number command
    EXPECT_EQ(KoiCommand_IsNumberCommand(cmd), 1);

    // Check parameter value
    int64_t value;
    EXPECT_EQ(KoiCommand_GetIntParam(cmd, 0, &value), 0);
    EXPECT_EQ(value, 42);

    KoiCommand_Del(cmd);
    KoiParser_Del(parser);
}

int main() {
    ::testing::InitGoogleTest();
    return RUN_ALL_TESTS();
}
