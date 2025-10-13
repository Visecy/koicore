#include <stdio.h>
#include <stdlib.h>

#include "koicore.h"

int main() {
    printf("=== KoiCore FFI Test ===\n\n");

    printf("Test 1: Creating parser...\n");
    const char *source = "#hello world\nThis is text.\n#command param";
    struct KoiParser *parser = koi_parser_new(source, 1);
    if (parser == NULL) {
        printf("Failed to create parser\n");
        return 1;
    }
    printf("✓ Parser created successfully\n\n");

    printf("Test 2: Parsing first command...\n");
    struct KoiCommand *cmd1 = koi_parser_next_command(parser);
    if (cmd1 == NULL) {
        const struct KoiError *error = koi_get_last_error();
        if (error != NULL) {
            printf("✗ Parse error at line %zu, column %zu: %s\n",
                   error->line, error->column, error->message);
            koi_error_free((struct KoiError*)error);
        } else {
            printf("✗ Unexpected NULL command\n");
        }
        koi_parser_free(parser);
        return 1;
    }

    const char *name1 = koi_command_name(cmd1);
    printf("✓ Command name: %s\n", name1);
    koi_string_free((char*)name1);
    koi_command_free(cmd1);
    printf("\n");

    printf("Test 3: Parsing text line...\n");
    struct KoiCommand *cmd2 = koi_parser_next_command(parser);
    if (cmd2 != NULL) {
        printf("✓ Text command parsed\n");

        const char *name2 = koi_command_name(cmd2);
        printf("✓ Command name: %s\n", name2);
        koi_string_free((char*)name2);
        koi_command_free(cmd2);
    } else {
        printf("✗ Failed to parse text\n");
    }
    printf("\n");

    printf("Test 4: Parsing third command...\n");
    struct KoiCommand *cmd3 = koi_parser_next_command(parser);
    if (cmd3 != NULL) {
        const char *name3 = koi_command_name(cmd3);
        printf("✓ Command name: %s\n", name3);
        koi_string_free((char*)name3);
        koi_command_free(cmd3);
    } else {
        const struct KoiError *error = koi_get_last_error();
        if (error != NULL) {
            printf("✗ Parse error: %s\n", error->message);
            koi_error_free((struct KoiError*)error);
        }
    }
    printf("\n");

    printf("Test 5: Testing EOF...\n");
    struct KoiCommand *cmd4 = koi_parser_next_command(parser);
    if (cmd4 == NULL) {
        const struct KoiError *error = koi_get_last_error();
        if (error == NULL) {
            printf("✓ Reached EOF correctly\n");
        } else {
            printf("✗ Unexpected error at EOF: %s\n", error->message);
            koi_error_free((struct KoiError*)error);
        }
    } else {
        printf("✗ Expected EOF but got command\n");
        koi_command_free(cmd4);
    }
    printf("\n");

    printf("Test 6: Testing error handling...\n");
    struct KoiParser *parser2 = koi_parser_new(" #", 1);
    struct KoiCommand *bad_cmd = koi_parser_next_command(parser2);
    if (bad_cmd == NULL) {
        const struct KoiError *error = koi_get_last_error();
        if (error != NULL) {
            printf("✓ Error caught: %s (line %zu, col %zu)\n",
                   error->message, error->line, error->column);
            koi_error_free((struct KoiError*)error);
        } else {
            printf("✗ Expected error but got NULL\n");
        }
    } else {
        printf("✗ Expected error but parsing succeeded\n");
        koi_command_free(bad_cmd);
    }
    koi_parser_free(parser2);
    printf("\n");

    printf("Cleaning up...\n");
    koi_parser_free(parser);
    koi_clear_last_error();

    printf("\n=== All tests completed ===\n");
    return 0;
}