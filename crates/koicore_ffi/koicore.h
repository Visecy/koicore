#ifndef __KOICORE_H__
#define __KOICORE_H__

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct KoiParser {
  uint8_t _private[0];
} KoiParser;

typedef struct KoiCommand {
  uint8_t _private[0];
} KoiCommand;

typedef struct KoiError {
  const char *message;
  uintptr_t line;
  uintptr_t column;
} KoiError;

struct KoiParser *koi_parser_new(const char *source, uintptr_t command_threshold);

void koi_parser_free(struct KoiParser *parser);

struct KoiCommand *koi_parser_next_command(struct KoiParser *parser);

const char *koi_command_name(const struct KoiCommand *cmd);

void koi_command_free(struct KoiCommand *cmd);

void koi_string_free(char *s);

const struct KoiError *koi_get_last_error(void);

void koi_clear_last_error(void);

void koi_error_free(struct KoiError *error);

#endif  /* __KOICORE_H__ */
