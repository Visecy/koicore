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

/**
 * Opaque handle for composite list parameter
 */
typedef struct KoiCompositeList {

} KoiCompositeList;

/**
 * Opaque handle for composite dict parameter
 */
typedef struct KoiCompositeDict {

} KoiCompositeDict;

struct KoiParser *koi_parser_new(const char *source, uintptr_t command_threshold);

void koi_parser_free(struct KoiParser *parser);

struct KoiCommand *koi_parser_next_command(struct KoiParser *parser);

const char *koi_command_name(const struct KoiCommand *cmd);

void koi_command_free(struct KoiCommand *cmd);

void koi_string_free(char *s);

const struct KoiError *koi_get_last_error(void);

void koi_clear_last_error(void);

void koi_error_free(struct KoiError *error);

/**
 * Get command name, caller provides buffer
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `buffer` - Buffer pointer provided by C caller
 * * `buffer_size` - Buffer size
 *
 * # Returns
 * Returns actual length of command name (excluding null terminator)
 * If buffer_size is insufficient, returns required buffer size (including null terminator)
 * Returns 0 if parameters are invalid
 */
uintptr_t KoiCommand_GetName(struct KoiCommand *command, char *buffer, uintptr_t buffer_size);

/**
 * Get command name length, always returns required size
 * Caller can call this first to get size, then allocate buffer and call full version
 */
uintptr_t KoiCommand_GetNameLen(struct KoiCommand *command);

/**
 * Create a new command with specified name and parameters
 *
 * # Arguments
 * * `name` - Command name (null-terminated C string)
 * * `param_count` - Number of parameters
 * * `params` - Array of parameter pointers
 *
 * # Returns
 * Pointer to new command object, or null pointer on error
 */
struct KoiCommand *KoiCommand_Create(const char *name,
                                     uintptr_t param_count,
                                     struct KoiCommand *const *params);

/**
 * Create a text command (@text)
 *
 * # Arguments
 * * `content` - Text content (null-terminated C string)
 *
 * # Returns
 * Pointer to new text command object
 */
struct KoiCommand *KoiCommand_CreateText(const char *content);

/**
 * Create an annotation command (@annotation)
 *
 * # Arguments
 * * `content` - Annotation content (null-terminated C string)
 *
 * # Returns
 * Pointer to new annotation command object
 */
struct KoiCommand *KoiCommand_CreateAnnotation(const char *content);

/**
 * Create a number command (@number)
 *
 * # Arguments
 * * `value` - Numeric value
 * * `param_count` - Number of additional parameters
 * * `params` - Array of additional parameter pointers
 *
 * # Returns
 * Pointer to new number command object, or null pointer on error
 */
struct KoiCommand *KoiCommand_CreateNumber(int64_t value,
                                           uintptr_t param_count,
                                           struct KoiCommand *const *params);

/**
 * Free a command object
 *
 * # Arguments
 * * `command` - Command object pointer to free
 */
void KoiCommand_Free(struct KoiCommand *command);

/**
 * Get number of parameters in command
 *
 * # Arguments
 * * `command` - Command object pointer
 *
 * # Returns
 * Number of parameters, or 0 if command is null
 */
uintptr_t KoiCommand_GetParamCount(struct KoiCommand *command);

/**
 * Get parameter type (unified enum for both basic and composite types)
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 *
 * # Returns
 * Parameter type, or KoiParamType::Invalid on error
 */
int32_t KoiCommand_GetParamType(struct KoiCommand *command, uintptr_t index);

/**
 * Get integer value from basic parameter
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 * * `out_value` - Pointer to store integer value
 *
 * # Returns
 * 1 on success, 0 on error or type mismatch
 */
int32_t KoiCommand_GetIntParam(struct KoiCommand *command, uintptr_t index, int64_t *out_value);

/**
 * Get float value from basic parameter
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 * * `out_value` - Pointer to store float value
 *
 * # Returns
 * 1 on success, 0 on error or type mismatch
 */
int32_t KoiCommand_GetFloatParam(struct KoiCommand *command, uintptr_t index, double *out_value);

/**
 * Get string value from basic parameter into provided buffer
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 * * `buffer` - Buffer for string output
 * * `buffer_size` - Buffer size
 *
 * # Returns
 * Actual string length (excluding null terminator), or required buffer size if insufficient
 * Returns 0 on error or type mismatch
 */
uintptr_t KoiCommand_GetStringParam(struct KoiCommand *command,
                                    uintptr_t index,
                                    char *buffer,
                                    uintptr_t buffer_size);

/**
 * Get literal value from basic parameter into provided buffer
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 * * `buffer` - Buffer for literal output
 * * `buffer_size` - Buffer size
 *
 * # Returns
 * Actual literal length (excluding null terminator), or required buffer size if insufficient
 * Returns 0 on error or type mismatch
 */
uintptr_t KoiCommand_GetLiteralParam(struct KoiCommand *command,
                                     uintptr_t index,
                                     char *buffer,
                                     uintptr_t buffer_size);

/**
 * Get composite parameter name into provided buffer
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 * * `buffer` - Buffer for name output
 * * `buffer_size` - Buffer size
 *
 * # Returns
 * Actual name length (excluding null terminator), or required buffer size if insufficient
 * Returns 0 on error or if parameter is not composite
 */
uintptr_t KoiCommand_GetCompositeParamName(struct KoiCommand *command,
                                           uintptr_t index,
                                           char *buffer,
                                           uintptr_t buffer_size);

/**
 * Check if command is a text command (@text)
 *
 * # Arguments
 * * `command` - Command object pointer
 *
 * # Returns
 * 1 if text command, 0 otherwise or on error
 */
int32_t KoiCommand_IsTextCommand(struct KoiCommand *command);

/**
 * Check if command is an annotation command (@annotation)
 *
 * # Arguments
 * * `command` - Command object pointer
 *
 * # Returns
 * 1 if annotation command, 0 otherwise or on error
 */
int32_t KoiCommand_IsAnnotationCommand(struct KoiCommand *command);

/**
 * Check if command is a number command (@number)
 *
 * # Arguments
 * * `command` - Command object pointer
 *
 * # Returns
 * 1 if number command, 0 otherwise or on error
 */
int32_t KoiCommand_IsNumberCommand(struct KoiCommand *command);

/**
 * Get composite list parameter from command
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 *
 * # Returns
 * Pointer to composite list parameter, or null on error
 */
struct KoiCompositeList *KoiCommand_GetCompositeList(struct KoiCommand *command, uintptr_t index);

/**
 * Get composite dict parameter from command
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 *
 * # Returns
 * Pointer to composite dict parameter, or null on error
 */
struct KoiCompositeDict *KoiCommand_GetCompositeDict(struct KoiCommand *command, uintptr_t index);

/**
 * Get composite list parameter length
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 *
 * # Returns
 * Number of elements in the list, or 0 on error
 */
uintptr_t KoiCompositeList_GetLength(struct KoiCompositeList *list);

#endif  /* __KOICORE_H__ */
