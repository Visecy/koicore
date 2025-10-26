#ifndef _INCLUDE_KOICORE_
#define _INCLUDE_KOICORE_

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#ifdef __cplusplus
namespace koicore {
#endif  // __cplusplus

typedef enum KoiFileInputEncodingStrategy {
  /**
   * Strict encoding error strategy, panics on invalid sequences
   */
  Strict = 0,
  /**
   * Replace invalid sequences with the replacement character (U+FFFD)
   */
  Replace = 1,
  /**
   * Ignore invalid sequences
   */
  Ignore = 2,
} KoiFileInputEncodingStrategy;

typedef struct KoiInputSource KoiInputSource;

typedef struct KoiParser KoiParser;

/**
 * Opaque handle for KoiLang command
 */
typedef struct KoiCommand {

} KoiCommand;

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

typedef struct KoiParserConfig {
  /**
   * The command threshold (number of # required for commands)
   *
   * Lines with fewer # characters than this threshold are treated as text.
   * Lines with exactly this many # characters are treated as commands.
   * Lines with more # characters are treated as annotations.
   */
  uintptr_t command_threshold;
  /**
   * Whether to skip annotation lines (lines starting with #)
   *
   * If set to true, annotation lines will be skipped and not processed as commands.
   * If set to false, annotation lines will be included in the output as special commands.
   */
  bool skip_annotations;
  /**
   * Whether to convert number commands to special commands
   *
   * If set to true, commands with names that are valid integers will be converted
   * to special number commands. If set to false, they will be treated as regular commands.
   */
  bool convert_number_command;
} KoiParserConfig;

typedef struct KoiParserError {

} KoiParserError;

typedef struct KoiTextInputSourceVTable {
  char *(*next_line)(void *user_data);
  const char *(*source_name)(void *user_data);
} KoiTextInputSourceVTable;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

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
 *
 * # Returns
 * Pointer to new command object, or null pointer on error
 */
struct KoiCommand *KoiCommand_New(const char *name);

/**
 * Create a text command (@text)
 *
 * # Arguments
 * * `content` - Text content (null-terminated C string)
 *
 * # Returns
 * Pointer to new text command object
 */
struct KoiCommand *KoiCommand_NewText(const char *content);

/**
 * Create an annotation command (@annotation)
 *
 * # Arguments
 * * `content` - Annotation content (null-terminated C string)
 *
 * # Returns
 * Pointer to new annotation command object
 */
struct KoiCommand *KoiCommand_NewAnnotation(const char *content);

/**
 * Create a number command (@number)
 *
 * # Arguments
 * * `value` - Numeric value
 *
 * # Returns
 * Pointer to new number command object, or null pointer on error
 */
struct KoiCommand *KoiCommand_NewNumber(int64_t value);

/**
 * Free a command object
 *
 * # Arguments
 * * `command` - Command object pointer to free
 */
void KoiCommand_Del(struct KoiCommand *command);

/**
 * Set command name
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `name` - New command name (null-terminated C string)
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCommand_SetName(struct KoiCommand *command, const char *name);

/**
 * Clone a command object
 *
 * # Arguments
 * * `command` - Command object pointer to clone
 *
 * # Returns
 * Pointer to new command object, or null on error
 */
struct KoiCommand *KoiCommand_Clone(const struct KoiCommand *command);

/**
 * Compare two command objects for equality
 *
 * # Arguments
 * * `command1` - First command object pointer
 * * `command2` - Second command object pointer
 *
 * # Returns
 * 1 if commands are equal, 0 if not equal or on error
 */
int32_t KoiCommand_Compare(const struct KoiCommand *command1, const struct KoiCommand *command2);

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
 * 0 on success, non-zero on error or type mismatch
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
 * 0 on success, non-zero on error or type mismatch
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
 * Get string parameter length
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 *
 * # Returns
 * Required buffer size (including null terminator), or 0 on error
 */
uintptr_t KoiCommand_GetStringParamLen(struct KoiCommand *command, uintptr_t index);

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
 * Get composite parameter name length
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 *
 * # Returns
 * Required buffer size (including null terminator), or 0 on error
 */
uintptr_t KoiCommand_GetCompositeParamNameLen(struct KoiCommand *command, uintptr_t index);

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
 * Add a new integer parameter to command
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `value` - Integer value
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCommand_AddIntParameter(struct KoiCommand *command, int64_t value);

/**
 * Add a new float parameter to command
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `value` - Float value
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCommand_AddFloatParameter(struct KoiCommand *command, double value);

/**
 * Add a new string parameter to command
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `value` - String value (null-terminated C string)
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCommand_AddStringParameter(struct KoiCommand *command, const char *value);

/**
 * Remove parameter from command by index
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index to remove
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCommand_RemoveParameter(struct KoiCommand *command, uintptr_t index);

/**
 * Clear all parameters from command
 *
 * # Arguments
 * * `command` - Command object pointer
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCommand_ClearParameters(struct KoiCommand *command);

/**
 * Modify integer parameter value
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 * * `value` - New integer value
 *
 * # Returns
 * 0 on success, non-zero on error or type mismatch
 */
int32_t KoiCommand_SetIntParameter(struct KoiCommand *command, uintptr_t index, int64_t value);

/**
 * Modify float parameter value
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 * * `value` - New float value
 *
 * # Returns
 * 0 on success, non-zero on error or type mismatch
 */
int32_t KoiCommand_SetFloatParameter(struct KoiCommand *command, uintptr_t index, double value);

/**
 * Modify string parameter value
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 * * `value` - New string value (null-terminated C string)
 *
 * # Returns
 * 0 on success, non-zero on error or type mismatch
 */
int32_t KoiCommand_SetStringParameter(struct KoiCommand *command,
                                      uintptr_t index,
                                      const char *value);

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
 * Get composite list parameter length
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 *
 * # Returns
 * Number of Values in the list, or 0 on error
 */
uintptr_t KoiCompositeList_GetLength(struct KoiCompositeList *list);

/**
 * Get Value type from composite list by index
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 * * `index` - Value index
 *
 * # Returns
 * Parameter type of the Value, or KoiParamType::Invalid on error
 */
int32_t KoiCompositeList_GetValueType(struct KoiCompositeList *list, uintptr_t index);

/**
 * Get integer Value from composite list by index
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 * * `index` - Value index
 * * `out_value` - Pointer to store integer value
 *
 * # Returns
 * 0 on success, non-zero on error or type mismatch
 */
int32_t KoiCompositeList_GetIntValue(struct KoiCompositeList *list,
                                     uintptr_t index,
                                     int64_t *out_value);

/**
 * Get float Value from composite list by index
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 * * `index` - Value index
 * * `out_value` - Pointer to store float value
 *
 * # Returns
 * 0 on success, non-zero on error or type mismatch
 */
int32_t KoiCompositeList_GetFloatValue(struct KoiCompositeList *list,
                                       uintptr_t index,
                                       double *out_value);

/**
 * Get string Value from composite list by index
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 * * `index` - Value index
 * * `buffer` - Buffer for string output
 * * `buffer_size` - Buffer size
 *
 * # Returns
 * Actual string length (excluding null terminator), or required buffer size if insufficient
 * Returns 0 on error or type mismatch
 */
uintptr_t KoiCompositeList_GetStringValue(struct KoiCompositeList *list,
                                          uintptr_t index,
                                          char *buffer,
                                          uintptr_t buffer_size);

/**
 * Get string Value length from composite list by index
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 * * `index` - Value index
 *
 * # Returns
 * Required buffer size (including null terminator)
 * Returns 0 on error or type mismatch
 */
uintptr_t KoiCompositeList_GetStringValueLen(struct KoiCompositeList *list, uintptr_t index);

/**
 * Create a new empty composite list
 *
 * # Returns
 * Pointer to new composite list parameter, or null on error
 */
struct KoiCompositeList *KoiCompositeList_New(void);

/**
 * Add integer value to composite list
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 * * `value` - Integer value to add
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeList_AddIntValue(struct KoiCompositeList *list, int64_t value);

/**
 * Add float value to composite list
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 * * `value` - Float value to add
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeList_AddFloatValue(struct KoiCompositeList *list, double value);

/**
 * Add string value to composite list
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 * * `value` - String value to add (null-terminated C string)
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeList_AddStringValue(struct KoiCompositeList *list, const char *value);

/**
 * Set integer value in composite list by index
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 * * `index` - Value index
 * * `value` - New integer value
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeList_SetIntValue(struct KoiCompositeList *list, uintptr_t index, int64_t value);

/**
 * Set float value in composite list by index
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 * * `index` - Value index
 * * `value` - New float value
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeList_SetFloatValue(struct KoiCompositeList *list,
                                       uintptr_t index,
                                       double value);

/**
 * Set string value in composite list by index
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 * * `index` - Value index
 * * `value` - New string value (null-terminated C string)
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeList_SetStringValue(struct KoiCompositeList *list,
                                        uintptr_t index,
                                        const char *value);

/**
 * Remove value from composite list by index
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 * * `index` - Value index
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeList_RemoveValue(struct KoiCompositeList *list, uintptr_t index);

/**
 * Clear all values from composite list
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeList_Clear(struct KoiCompositeList *list);

/**
 * Free composite list parameter
 *
 * # Arguments
 * * `list` - Composite list parameter pointer
 */
void KoiCompositeList_Del(struct KoiCompositeList *list);

/**
 * Create a new composite dict parameter
 *
 * # Returns
 * Pointer to the new composite dict parameter, or null on error
 */
struct KoiCompositeDict *KoiCompositeDict_New(void);

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
 * Get number of entries in dict
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 *
 * # Returns
 * Number of entries in the dict, 0 on error
 */
uintptr_t KoiCompositeDict_GetLength(struct KoiCompositeDict *dict);

/**
 * Remove entry from composite dict by key
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `key` - Key name
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeDict_Remove(struct KoiCompositeDict *dict, const char *key);

/**
 * Clear all entries from composite dict
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeDict_Clear(struct KoiCompositeDict *dict);

/**
 * Free composite dict parameter
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 */
void KoiCompositeDict_Del(struct KoiCompositeDict *dict);

/**
 * Set integer value in composite dict by key
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `key` - Key name
 * * `value` - Integer value to set
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeDict_SetIntValue(struct KoiCompositeDict *dict, const char *key, int64_t value);

/**
 * Set float value in composite dict by key
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `key` - Key name
 * * `value` - Float value to set
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeDict_SetFloatValue(struct KoiCompositeDict *dict,
                                       const char *key,
                                       double value);

/**
 * Set string value in composite dict by key
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `key` - Key name
 * * `value` - String value to set
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeDict_SetStringValue(struct KoiCompositeDict *dict,
                                        const char *key,
                                        const char *value);

/**
 * Get dict key by index into provided buffer
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `index` - Entry index
 * * `buffer` - Buffer for key output
 * * `buffer_size` - Buffer size
 *
 * # Returns
 * Actual key length (excluding null terminator), or required buffer size if insufficient
 * Returns 0 on error
 */
uintptr_t KoiCompositeDict_GetKeybyIndex(struct KoiCompositeDict *dict,
                                         uintptr_t index,
                                         char *buffer,
                                         uintptr_t buffer_size);

/**
 * Get dict key length by index
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `index` - Entry index
 *
 * # Returns
 * Required buffer size (including null terminator)
 * Returns 0 on error
 */
uintptr_t KoiCompositeDict_GetKeyLenByIndex(struct KoiCompositeDict *dict, uintptr_t index);

/**
 * Get dict value type by index
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `index` - Entry index
 *
 * # Returns
 * Value type as KoiParamType enum value
 */
int32_t KoiCompositeDict_GetValueTypeByIndex(struct KoiCompositeDict *dict, uintptr_t index);

/**
 * Get value type from composite dict by key
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `key` - Key name
 *
 * # Returns
 * Value type as KoiParamType enum value
 */
int32_t KoiCompositeDict_GetValueType(struct KoiCompositeDict *dict, const char *key);

/**
 * Get integer value from composite dict by key
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `key` - Key name
 * * `out_value` - Pointer to store integer value
 *
 * # Returns
 * 0 on success, non-zero on error or type mismatch
 */
int32_t KoiCompositeDict_GetIntValue(struct KoiCompositeDict *dict,
                                     const char *key,
                                     int64_t *out_value);

/**
 * Get float value from composite dict by key
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `key` - Key name
 * * `out_value` - Pointer to store float value
 *
 * # Returns
 * 0 on success, non-zero on error or type mismatch
 */
int32_t KoiCompositeDict_GetFloatValue(struct KoiCompositeDict *dict,
                                       const char *key,
                                       double *out_value);

/**
 * Get string value from composite dict by key into provided buffer
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `key` - Key name
 * * `buffer` - Buffer for string output
 * * `buffer_size` - Buffer size
 *
 * # Returns
 * Actual string length (excluding null terminator), or required buffer size if insufficient
 * Returns 0 on error or type mismatch
 */
uintptr_t KoiCompositeDict_GetStringValue(struct KoiCompositeDict *dict,
                                          const char *key,
                                          char *buffer,
                                          uintptr_t buffer_size);

/**
 * Get string value length from composite dict by key
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `key` - Key name
 *
 * # Returns
 * Required buffer size (including null terminator), or 0 on error or type mismatch
 */
uintptr_t KoiCompositeDict_GetStringValueLen(struct KoiCompositeDict *dict, const char *key);

struct KoiParser *KoiParser_New(struct KoiInputSource *input, struct KoiParserConfig *config);

void KoiParser_Del(struct KoiParser *parser);

struct KoiCommand *KoiParser_NextCommand(struct KoiParser *parser);

struct KoiParserError *KoiParser_Error(struct KoiParser *parser);

void KoiParserError_Del(struct KoiParserError *_this);

uintptr_t KoiParserError_Format(const struct KoiParserError *_this,
                                char *buffer,
                                uintptr_t buffer_size);

uintptr_t KoiParserError_FormatLen(const struct KoiParserError *_this);

uintptr_t KoiParserError_GetMessage(const struct KoiParserError *_this,
                                    char *buffer,
                                    uintptr_t buffer_size);

uintptr_t KoiParserError_GetMessageLen(const struct KoiParserError *_this);

int32_t KoiParserError_GetTracebackPosition(const struct KoiParserError *_this,
                                            uintptr_t *lineno,
                                            uintptr_t *column);

struct KoiInputSource *KoiInputSource_FromVTable(const struct KoiTextInputSourceVTable *vtable,
                                                 void *user_data);

struct KoiInputSource *KoiInputSource_FromString(const char *source);

struct KoiInputSource *KoiInputSource_FromFile(const char *path);

struct KoiInputSource *KoiInputSource_FromFileAndEncoding(const char *path,
                                                          const char *encoding,
                                                          enum KoiFileInputEncodingStrategy encoding_strategy);

void KoiInputSource_Del(struct KoiInputSource *input);

void KoiParserConfig_Init(struct KoiParserConfig *config);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#ifdef __cplusplus
}  // namespace koicore
#endif  // __cplusplus

#endif  /* _INCLUDE_KOICORE_ */
