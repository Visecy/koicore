#ifndef _INCLUDE_KOICORE_
#define _INCLUDE_KOICORE_

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#ifdef __cplusplus
namespace koicore {
#endif  // __cplusplus

/**
 * Strategy for handling encoding errors when reading files
 *
 * This enum determines how the parser handles invalid byte sequences
 * when reading files with a specific encoding.
 */
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

/**
 * Number format for numeric values
 */
typedef enum KoiNumberFormat {
  Unknown = 0,
  Decimal = 1,
  Hex = 2,
  Octal = 3,
  Binary = 4,
} KoiNumberFormat;

/**
 * Opaque handle for KoiLang input sources
 *
 * This structure represents an input source that provides text to the parser.
 * It can be created from strings, files, or custom callback functions.
 */
typedef struct KoiInputSource KoiInputSource;

/**
 * Opaque handle for KoiLang parser
 *
 * This structure represents a parser instance that can parse KoiLang text from various sources.
 * The parser maintains internal state including the current position, error information,
 * and end-of-file status.
 */
typedef struct KoiParser KoiParser;

/**
 * String output buffer that can be shared with C
 */
typedef struct KoiStringOutput KoiStringOutput;

/**
 * Opaque handle for KoiWriter
 */
typedef struct KoiWriter KoiWriter;

/**
 * Opaque handle for KoiLang command
 *
 * This structure represents a command in the KoiLang language. Commands can be
 * regular commands with a name and parameters, text commands, annotation commands,
 * or number commands. The actual implementation is hidden to maintain ABI stability.
 */
typedef struct KoiCommand {

} KoiCommand;

/**
 * Opaque handle for composite dict parameter
 *
 * This struct represents a dictionary-style composite parameter that stores key-value pairs.
 * The keys are strings, and the values can be integers, floats, or strings.
 * This is an opaque type intended for use through the C FFI API.
 */
typedef struct KoiCompositeDict {

} KoiCompositeDict;

/**
 * Opaque handle for composite list parameter
 *
 * This structure represents a list parameter in a KoiLang command.
 * Lists can contain values of different types (integers, floats, strings).
 */
typedef struct KoiCompositeList {

} KoiCompositeList;

/**
 * Opaque handle for composite single parameter
 *
 * Represents a named parameter with a single value, e.g., name(value).
 */
typedef struct KoiCompositeSingle {

} KoiCompositeSingle;

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
  /**
   * Whether to preserve indentation in text and annotation lines
   *
   * If set to true, leading whitespace (indentation) will be preserved in text and
   * annotation content. If set to false, leading whitespace will be trimmed.
   */
  bool preserve_indent;
  /**
   * Whether to preserve empty lines as text commands
   *
   * If set to true, empty lines will be preserved and returned as empty text commands.
   * If set to false, empty lines will be skipped.
   */
  bool preserve_empty_lines;
} KoiParserConfig;

/**
 * Opaque handle for KoiLang parser errors
 *
 * This structure represents an error that occurred during parsing.
 * It contains detailed information about the error including the message
 * and position in the source text where the error occurred.
 */
typedef struct KoiParserError {

} KoiParserError;

/**
 * VTable for custom text input sources
 *
 * This structure provides function pointers for implementing custom input sources
 * in C or other languages. The user must provide implementations of these functions.
 */
typedef struct KoiTextInputSourceVTable {
  /**
   * Function to get the next line of text
   *
   * # Arguments
   *
   * * `user_data` - User-provided data pointer
   *
   * # Returns
   *
   * Pointer to a null-terminated C string containing the next line,
   * or NULL if there are no more lines or an error occurred.
   * The string must remain valid until the next call or until the input source is destroyed.
   */
  char *(*next_line)(void *user_data);
  /**
   * Function to get the name of the input source
   *
   * # Arguments
   *
   * * `user_data` - User-provided data pointer
   *
   * # Returns
   *
   * Pointer to a null-terminated C string containing the source name,
   * or NULL if no name is available.
   */
  const char *(*source_name)(void *user_data);
} KoiTextInputSourceVTable;

/**
 * VTable for custom writer output
 */
typedef struct KoiWriterOutputVTable {
  /**
   * Function to write data
   *
   * # Arguments
   * * `user_data` - User-provided data pointer
   * * `buf` - Pointer to data buffer
   * * `len` - Length of data buffer
   *
   * # Returns
   * Number of bytes written. 0 implies error if len > 0.
   */
  uintptr_t (*write)(void *user_data, const uint8_t *buf, uintptr_t len);
  /**
   * Function to flush output
   *
   * # Arguments
   * * `user_data` - User-provided data pointer
   *
   * # Returns
   * 0 on success, non-zero on error
   */
  int32_t (*flush)(void *user_data);
} KoiWriterOutputVTable;

/**
 * Transparent configuration struct for FFI
 */
typedef struct KoiFormatterOptions {
  uintptr_t indent;
  bool use_tabs;
  bool newline_before;
  bool newline_after;
  bool compact;
  bool force_quotes_for_vars;
  enum KoiNumberFormat number_format;
  bool newline_before_param;
  bool newline_after_param;
  bool should_override;
} KoiFormatterOptions;

/**
 * Command-specific option entry
 * Used in a null-terminated array
 */
typedef struct KoiCommandOption {
  /**
   * Command name string
   * Terminate list with name = NULL
   */
  const char *name;
  struct KoiFormatterOptions options;
} KoiCommandOption;

/**
 * Transparent WriterConfig
 * Users can allocate this on stack.
 */
typedef struct KoiWriterConfig {
  struct KoiFormatterOptions global_options;
  uintptr_t command_threshold;
  /**
   * Pointer to array of KoiCommandOption, terminated by name=NULL.
   * Can be NULL if no command options.
   */
  const struct KoiCommandOption *command_options;
} KoiWriterConfig;

/**
 * Parameter selector for options
 * If is_position is true, uses position. Otherwise uses name.
 */
typedef struct KoiParamFormatSelector {
  bool is_position;
  uintptr_t position;
  /**
   * Only used if is_position is false
   */
  const char *name;
} KoiParamFormatSelector;

/**
 * Parameter-specific option entry
 * Used in a null-terminated array
 */
typedef struct KoiParamOption {
  /**
   * Selector for which parameter this applies to
   * Terminate list with name = NULL AND is_position = false
   */
  struct KoiParamFormatSelector selector;
  struct KoiFormatterOptions options;
} KoiParamOption;

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
 * Create a new composite dict parameter
 *
 * Creates an empty dictionary with no key-value pairs. The returned pointer
 * must be freed using KoiCompositeDict_Del when no longer needed to avoid memory leaks,
 * unless it is passed to KoiCommand_AddCompositeDict.
 *
 * # Arguments
 *
 * * `name` - The name of the composite parameter (null-terminated C string)
 *
 * # Returns
 * Pointer to the new composite dict parameter, or null on error
 *
 * # Safety
 * The returned pointer must not be used after calling KoiCompositeDict_Del on it.
 * The caller is responsible for memory management of the returned object.
 */
struct KoiCompositeDict *KoiCompositeDict_New(const char *name);

/**
 * Add composite dict to command
 *
 * Adds the specified composite dict to the command's parameters.
 * This function TAKES OWNERSHIP of the dict pointer. The caller must NOT
 * free the dict using KoiCompositeDict_Del after a successful call.
 *
 * # Arguments
 *
 * * `command` - Pointer to the command object
 * * `dict` - Pointer to the composite dict parameter to add
 *
 * # Returns
 *
 * 0 on success, or a non-zero error code on failure:
 * - -1: command or dict pointer is NULL
 * - -3: command pointer is invalid
 *
 * # Safety
 *
 * The `command` pointer must be a valid KoiCommand.
 * The `dict` pointer must be a valid KoiCompositeDict created with KoiCompositeDict_New.
 */
int32_t KoiCommand_AddCompositeDict(struct KoiCommand *command, struct KoiCompositeDict *dict);

/**
 * Get composite dict parameter from command
 *
 * Retrieves a dictionary parameter from a command at the specified index.
 * The parameter must be of dictionary type, otherwise null is returned.
 *
 * # Ownership and Lifetime
 *
 * The returned pointer is a borrowed reference to data owned by the command.
 * It must NOT be freed with KoiCompositeDict_Del. The pointer is only valid
 * as long as the command object exists and is not modified or destroyed.
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index (0-based)
 *
 * # Returns
 * Pointer to composite dict parameter, or null on error:
 * - null if command is null
 * - null if index is out of bounds
 * - null if parameter at index is not a dictionary
 *
 * # Safety
 * The command pointer must be either null or point to a valid KoiCommand object.
 * The returned pointer must NOT be freed with KoiCompositeDict_Del as it is owned by the command.
 * The returned pointer becomes invalid if the command is destroyed or modified.
 */
struct KoiCompositeDict *KoiCommand_GetCompositeDict(struct KoiCommand *command, uintptr_t index);

/**
 * Get number of entries in dict
 *
 * Returns the count of key-value pairs currently stored in the dictionary.
 * This is useful for iteration or checking if the dictionary is empty.
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 *
 * # Returns
 * Number of entries in the dict, 0 on error:
 * - 0 if dict is null
 * - 0 if dict is not a valid dictionary
 *
 * # Safety
 * The dict pointer must be a valid KoiCompositeDict pointer.
 */
uintptr_t KoiCompositeDict_GetLength(struct KoiCompositeDict *dict);

/**
 * Remove entry from composite dict by key
 *
 * Removes a key-value pair from the dictionary based on the provided key.
 * If the key does not exist in the dictionary, the function returns an error.
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `key` - Key name (null-terminated UTF-8 string)
 *
 * # Returns
 * 0 on success, non-zero on error:
 * - -1 if dict or key is null
 * - -2 if key contains invalid UTF-8
 * - -3 if key not found in dictionary
 * - -4 if dict is not a valid dictionary
 *
 * # Safety
 * The dict pointer must be a valid KoiCompositeDict pointer.
 * The key pointer must be a valid null-terminated C string.
 */
int32_t KoiCompositeDict_Remove(struct KoiCompositeDict *dict, const char *key);

/**
 * Clear all entries from composite dict
 *
 * Removes all key-value pairs from the dictionary, making it empty.
 * This operation is irreversible but does not deallocate the dictionary itself.
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 *
 * # Returns
 * 0 on success, non-zero on error:
 * - -1 if dict is null
 * - -3 if dict is not a valid dictionary
 *
 * # Safety
 * The dict pointer must be a valid KoiCompositeDict pointer.
 */
int32_t KoiCompositeDict_Clear(struct KoiCompositeDict *dict);

/**
 * Free composite dict parameter
 *
 * Deallocates the memory used by the dictionary and all its key-value pairs.
 * After calling this function, the pointer becomes invalid and must not be used.
 * This function should only be called on dictionaries created with KoiCompositeDict_New,
 * not on dictionaries obtained from KoiCommand_GetCompositeDict.
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 *
 * # Safety
 * The dict pointer must be a valid KoiCompositeDict pointer created with KoiCompositeDict_New.
 * Do not call this function on pointers obtained from KoiCommand_GetCompositeDict.
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

/**
 * Get boolean value from composite dict by key
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `key` - Key name
 * * `out_value` - Pointer to store boolean value (1 for true, 0 for false)
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeDict_GetBoolValue(struct KoiCompositeDict *dict,
                                      const char *key,
                                      int32_t *out_value);

/**
 * Set boolean value in composite dict by key
 *
 * # Arguments
 * * `dict` - Composite dict parameter pointer
 * * `key` - Key name
 * * `value` - Boolean value to set (non-zero for true, 0 for false)
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeDict_SetBoolValue(struct KoiCompositeDict *dict,
                                      const char *key,
                                      int32_t value);

/**
 * Get composite list parameter from command
 *
 * This function retrieves a list parameter from a command at the specified index.
 * The parameter must be of list type, otherwise NULL is returned.
 *
 * # Ownership and Lifetime
 *
 * The returned pointer is a borrowed reference to data owned by the command.
 * It must NOT be freed with KoiCompositeList_Del. The pointer is only valid
 * as long as the command object exists and is not modified or destroyed.
 *
 * # Arguments
 *
 * * `command` - Pointer to the command object
 * * `index` - Zero-based index of the parameter to retrieve
 *
 * # Returns
 *
 * Pointer to the composite list parameter, or NULL if:
 * - command is NULL
 * - index is out of bounds
 * - the parameter at the specified index is not a list
 *
 * # Safety
 *
 * The `command` pointer must be either NULL or point to a valid KoiCommand object.
 * The returned pointer must NOT be freed with KoiCompositeList_Del as it is owned by the command.
 * The returned pointer becomes invalid if the command is destroyed or modified.
 */
struct KoiCompositeList *KoiCommand_GetCompositeList(struct KoiCommand *command, uintptr_t index);

/**
 * Get composite list parameter length
 *
 * This function returns the number of elements in the list parameter.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 *
 * # Returns
 *
 * Number of elements in the list, or 0 if the list pointer is NULL or invalid.
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object
 * obtained from KoiCommand_GetCompositeList or KoiCompositeList_New.
 */
uintptr_t KoiCompositeList_GetLength(struct KoiCompositeList *list);

/**
 * Get value type from composite list by index
 *
 * This function determines the type of a value at the specified index in the list.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 * * `index` - Zero-based index of the value to query
 *
 * # Returns
 *
 * The type of the value as a KoiParamType enum value, or KoiParamType::Invalid
 * if the list pointer is NULL, invalid, or the index is out of bounds.
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
 */
int32_t KoiCompositeList_GetValueType(struct KoiCompositeList *list, uintptr_t index);

/**
 * Get integer value from composite list by index
 *
 * This function retrieves an integer value from the list at the specified index.
 * The value at the specified index must be of integer type.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 * * `index` - Zero-based index of the value to retrieve
 * * `out_value` - Pointer to store the retrieved integer value
 *
 * # Returns
 *
 * 0 on success, or a non-zero error code on failure:
 * - -1: list pointer is NULL or out_value is NULL
 * - -2: index is out of bounds
 * - -3: value at the specified index is not an integer
 * - -4: list pointer is invalid
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
 * The `out_value` pointer must point to a valid i64 variable.
 */
int32_t KoiCompositeList_GetIntValue(struct KoiCompositeList *list,
                                     uintptr_t index,
                                     int64_t *out_value);

/**
 * Get float value from composite list by index
 *
 * This function retrieves a float value from the list at the specified index.
 * The value at the specified index must be of float type.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 * * `index` - Zero-based index of the value to retrieve
 * * `out_value` - Pointer to store the retrieved float value
 *
 * # Returns
 *
 * 0 on success, or a non-zero error code on failure:
 * - -1: list pointer is NULL or out_value is NULL
 * - -2: index is out of bounds
 * - -3: value at the specified index is not a float
 * - -4: list pointer is invalid
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
 * The `out_value` pointer must point to a valid f64 variable.
 */
int32_t KoiCompositeList_GetFloatValue(struct KoiCompositeList *list,
                                       uintptr_t index,
                                       double *out_value);

/**
 * Get string value from composite list by index
 *
 * This function retrieves a string value from the list at the specified index.
 * The value at the specified index must be of string type.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 * * `index` - Zero-based index of the value to retrieve
 * * `buffer` - Buffer to store the retrieved string value
 * * `buffer_size` - Size of the buffer in bytes
 *
 * # Returns
 *
 * The number of bytes required for the string including the null terminator.
 * If the buffer is NULL or too small, no data is written and the required size is returned.
 * Returns 0 on error or if the value at the specified index is not a string.
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
 * If `buffer` is not NULL, it must point to a valid memory region of at least `buffer_size` bytes.
 */
uintptr_t KoiCompositeList_GetStringValue(struct KoiCompositeList *list,
                                          uintptr_t index,
                                          char *buffer,
                                          uintptr_t buffer_size);

/**
 * Get string value length from composite list by index
 *
 * This function returns the length of a string value at the specified index in the list.
 * The value at the specified index must be of string type.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 * * `index` - Zero-based index of the value to query
 *
 * # Returns
 *
 * Required buffer size (including null terminator), or 0 on error or type mismatch.
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
 */
uintptr_t KoiCompositeList_GetStringValueLen(struct KoiCompositeList *list, uintptr_t index);

/**
 * Create a new empty composite list
 *
 * This function creates a new empty composite list parameter that can be
 * added to a command or used independently.
 *
 * # Arguments
 *
 * * `name` - The name of the composite parameter (null-terminated C string)
 *
 * # Returns
 *
 * Pointer to the newly created composite list parameter, or NULL on failure.
 * The caller is responsible for freeing the list using KoiCompositeList_Del,
 * unless it is passed to KoiCommand_AddCompositeList.
 *
 * # Safety
 *
 * The returned pointer must be freed using KoiCompositeList_Del when no longer needed,
 * unless ownership is transferred to a command.
 */
struct KoiCompositeList *KoiCompositeList_New(const char *name);

/**
 * Add composite list to command
 *
 * Adds the specified composite list to the command's parameters.
 * This function TAKES OWNERSHIP of the list pointer. The caller must NOT
 * free the list using KoiCompositeList_Del after a successful call.
 *
 * # Arguments
 *
 * * `command` - Pointer to the command object
 * * `list` - Pointer to the composite list parameter to add
 *
 * # Returns
 *
 * 0 on success, or a non-zero error code on failure:
 * - -1: command or list pointer is NULL
 * - -3: command pointer is invalid
 *
 * # Safety
 *
 * The `command` pointer must be a valid KoiCommand.
 * The `list` pointer must be a valid KoiCompositeList created with KoiCompositeList_New.
 */
int32_t KoiCommand_AddCompositeList(struct KoiCommand *command, struct KoiCompositeList *list);

/**
 * Add integer value to composite list
 *
 * This function appends an integer value to the end of the list.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 * * `value` - Integer value to add
 *
 * # Returns
 *
 * 0 on success, or a non-zero error code on failure:
 * - -1: list pointer is NULL
 * - -3: list pointer is invalid
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object
 * obtained from KoiCommand_GetCompositeList or KoiCompositeList_New.
 */
int32_t KoiCompositeList_AddIntValue(struct KoiCompositeList *list, int64_t value);

/**
 * Add float value to composite list
 *
 * This function appends a float value to the end of the list.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 * * `value` - Float value to add
 *
 * # Returns
 *
 * 0 on success, or a non-zero error code on failure:
 * - -1: list pointer is NULL
 * - -3: list pointer is invalid
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object
 * obtained from KoiCommand_GetCompositeList or KoiCompositeList_New.
 */
int32_t KoiCompositeList_AddFloatValue(struct KoiCompositeList *list, double value);

/**
 * Add string value to composite list
 *
 * This function appends a string value to the end of the list.
 * The string is copied and managed by the list.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 * * `value` - String value to add (null-terminated C string)
 *
 * # Returns
 *
 * 0 on success, or a non-zero error code on failure:
 * - -1: list pointer is NULL or value is NULL
 * - -2: value contains invalid UTF-8
 * - -3: list pointer is invalid
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
 * The `value` pointer must be either NULL or point to a valid null-terminated string.
 */
int32_t KoiCompositeList_AddStringValue(struct KoiCompositeList *list, const char *value);

/**
 * Set integer value in composite list by index
 *
 * This function replaces the value at the specified index with an integer value.
 * The index must be within the bounds of the list.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 * * `index` - Zero-based index of the value to replace
 * * `value` - New integer value
 *
 * # Returns
 *
 * 0 on success, or a non-zero error code on failure:
 * - -1: list pointer is NULL
 * - -2: index is out of bounds
 * - -3: list pointer is invalid
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
 */
int32_t KoiCompositeList_SetIntValue(struct KoiCompositeList *list, uintptr_t index, int64_t value);

/**
 * Set float value in composite list by index
 *
 * This function replaces the value at the specified index with a float value.
 * The index must be within the bounds of the list.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 * * `index` - Zero-based index of the value to replace
 * * `value` - New float value
 *
 * # Returns
 *
 * 0 on success, or a non-zero error code on failure:
 * - -1: list pointer is NULL
 * - -2: index is out of bounds
 * - -3: list pointer is invalid
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
 */
int32_t KoiCompositeList_SetFloatValue(struct KoiCompositeList *list,
                                       uintptr_t index,
                                       double value);

/**
 * Set string value in composite list by index
 *
 * This function replaces the value at the specified index with a string value.
 * The index must be within the bounds of the list.
 * The string is copied and managed by the list.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 * * `index` - Zero-based index of the value to replace
 * * `value` - New string value (null-terminated C string)
 *
 * # Returns
 *
 * 0 on success, or a non-zero error code on failure:
 * - -1: list pointer is NULL or value is NULL
 * - -2: value contains invalid UTF-8
 * - -3: list pointer is invalid
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
 * The `value` pointer must be either NULL or point to a valid null-terminated string.
 */
int32_t KoiCompositeList_SetStringValue(struct KoiCompositeList *list,
                                        uintptr_t index,
                                        const char *value);

/**
 * Remove value from composite list by index
 *
 * This function removes the value at the specified index from the list.
 * The index must be within the bounds of the list.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 * * `index` - Zero-based index of the value to remove
 *
 * # Returns
 *
 * 0 on success, or a non-zero error code on failure:
 * - -1: list pointer is NULL
 * - -2: index is out of bounds
 * - -3: list pointer is invalid
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
 */
int32_t KoiCompositeList_RemoveValue(struct KoiCompositeList *list, uintptr_t index);

/**
 * Clear all values from composite list
 *
 * This function removes all values from the list, making it empty.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter
 *
 * # Returns
 *
 * 0 on success, or a non-zero error code on failure:
 * - -1: list pointer is NULL
 * - -3: list pointer is invalid
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object.
 */
int32_t KoiCompositeList_Clear(struct KoiCompositeList *list);

/**
 * Free composite list parameter
 *
 * This function frees the memory used by a composite list parameter.
 * After calling this function, the list pointer becomes invalid and must not be used.
 *
 * # Arguments
 *
 * * `list` - Pointer to the composite list parameter to delete
 *
 * # Safety
 *
 * The `list` pointer must be either NULL or point to a valid KoiCompositeList object
 * obtained from KoiCommand_GetCompositeList or KoiCompositeList_New.
 * After this function returns, the pointer is invalid and must not be used.
 */
void KoiCompositeList_Del(struct KoiCompositeList *list);

/**
 * Get boolean value from composite list by index
 *
 * # Arguments
 * * `list` - Pointer to the composite list parameter
 * * `index` - Zero-based index of the value to retrieve
 * * `out_value` - Pointer to store boolean value (1 for true, 0 for false)
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeList_GetBoolValue(struct KoiCompositeList *list,
                                      uintptr_t index,
                                      int32_t *out_value);

/**
 * Add boolean value to composite list
 *
 * # Arguments
 * * `list` - Pointer to the composite list parameter
 * * `value` - Boolean value to add (non-zero for true, 0 for false)
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeList_AddBoolValue(struct KoiCompositeList *list, int32_t value);

/**
 * Set boolean value in composite list by index
 *
 * # Arguments
 * * `list` - Pointer to the composite list parameter
 * * `index` - Zero-based index of the value to replace
 * * `value` - New boolean value (non-zero for true, 0 for false)
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCompositeList_SetBoolValue(struct KoiCompositeList *list,
                                      uintptr_t index,
                                      int32_t value);

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
 * Get boolean value from basic parameter
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 * * `out_value` - Pointer to store boolean value (1 for true, 0 for false)
 *
 * # Returns
 * 0 on success, non-zero on error or type mismatch
 */
int32_t KoiCommand_GetBoolParam(struct KoiCommand *command, uintptr_t index, int32_t *out_value);

/**
 * Add a new boolean parameter to command
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `value` - Boolean value (non-zero for true, 0 for false)
 *
 * # Returns
 * 0 on success, non-zero on error
 */
int32_t KoiCommand_AddBoolParameter(struct KoiCommand *command, int32_t value);

/**
 * Modify boolean parameter value
 *
 * # Arguments
 * * `command` - Command object pointer
 * * `index` - Parameter index
 * * `value` - New boolean value (non-zero for true, 0 for false)
 *
 * # Returns
 * 0 on success, non-zero on error or type mismatch
 */
int32_t KoiCommand_SetBoolParameter(struct KoiCommand *command, uintptr_t index, int32_t value);

/**
 * Create a new composite single parameter
 *
 * # Arguments
 * * `name` - The name of the composite parameter
 * * `value` - Initial integer value (dummy, will be overwritten by Set functions)
 *
 * # Returns
 * Pointer to the new composite single parameter, or NULL on error
 */
struct KoiCompositeSingle *KoiCompositeSingle_New(const char *name);

/**
 * Get composite single parameter from command
 */
struct KoiCompositeSingle *KoiCommand_GetCompositeSingle(struct KoiCommand *command,
                                                         uintptr_t index);

/**
 * Free composite single parameter
 */
void KoiCompositeSingle_Del(struct KoiCompositeSingle *single);

/**
 * Set integer value in composite single
 */
int32_t KoiCompositeSingle_SetIntValue(struct KoiCompositeSingle *single, int64_t value);

/**
 * Get integer value from composite single
 */
int32_t KoiCompositeSingle_GetIntValue(struct KoiCompositeSingle *single, int64_t *out_value);

/**
 * Set float value in composite single
 */
int32_t KoiCompositeSingle_SetFloatValue(struct KoiCompositeSingle *single, double value);

/**
 * Get float value from composite single
 */
int32_t KoiCompositeSingle_GetFloatValue(struct KoiCompositeSingle *single, double *out_value);

/**
 * Set string value in composite single
 */
int32_t KoiCompositeSingle_SetStringValue(struct KoiCompositeSingle *single, const char *value);

/**
 * Set boolean value in composite single
 */
int32_t KoiCompositeSingle_SetBoolValue(struct KoiCompositeSingle *single, int32_t value);

/**
 * Get boolean value from composite single
 */
int32_t KoiCompositeSingle_GetBoolValue(struct KoiCompositeSingle *single, int32_t *out_value);

/**
 * Add composite single to command
 */
int32_t KoiCommand_AddCompositeSingle(struct KoiCommand *command,
                                      struct KoiCompositeSingle *single);

/**
 * Get value type from composite single
 */
int32_t KoiCompositeSingle_GetValueType(struct KoiCompositeSingle *single);

/**
 * Create a new KoiLang parser
 *
 * Creates a new parser instance with the provided input source and configuration.
 * This function takes ownership of the input source, which means the caller should
 * not use or free the input source after this call. The parser will manage the input
 * source's lifecycle.
 *
 * # Arguments
 * * `input` - Input source pointer (ownership is transferred to the parser)
 * * `config` - Parser configuration pointer
 *
 * # Returns
 * Pointer to the new parser instance, or null on error:
 * - null if config is null
 * - null if input is null
 *
 * # Safety
 *
 * The input pointer must be a valid KoiInputSource created with one of the
 * KoiInputSource_From* functions. After calling this function, the input pointer
 * becomes invalid and must not be used or freed.
 * The config pointer must be a valid KoiParserConfig created with KoiParserConfig_New.
 */
struct KoiParser *KoiParser_New(struct KoiInputSource *input, struct KoiParserConfig *config);

/**
 * Delete a KoiLang parser and free its resources
 *
 * This function destroys a parser instance and releases all associated memory,
 * including the input source that was transferred to it during creation.
 * After calling this function, the parser pointer becomes invalid and must not
 * be used.
 *
 * # Arguments
 * * `parser` - Parser pointer to delete
 *
 * # Safety
 * The parser pointer must be a valid KoiParser created with KoiParser_New.
 * After this call, the parser pointer becomes invalid and must not be used.
 */
void KoiParser_Del(struct KoiParser *parser);

/**
 * Get the next command from the parser
 *
 * Retrieves the next command from the input source. Returns null when the end of
 * the input is reached or when an error occurs. If an error occurs, the error
 * can be retrieved using KoiParser_Error.
 *
 * # Arguments
 * * `parser` - Parser pointer
 *
 * # Returns
 * Pointer to the next command, or null in these cases:
 * - null if parser is null
 * - null if end of input is reached
 * - null if a parsing error occurred
 *
 * # Safety
 * The parser pointer must be a valid KoiParser created with KoiParser_New.
 * The returned command pointer is owned by the caller and must be freed with
 * KoiCommand_Del when no longer needed.
 */
struct KoiCommand *KoiParser_NextCommand(struct KoiParser *parser);

/**
 * Get the last parsing error from the parser
 *
 * Retrieves the last error that occurred during parsing, if any. This function
 * transfers ownership of the error to the caller, so subsequent calls will
 * return null until another error occurs.
 *
 * # Arguments
 * * `parser` - Parser pointer
 *
 * # Returns
 * Pointer to the last error, or null in these cases:
 * - null if parser is null
 * - null if no error has occurred
 * - null if the error has already been retrieved
 *
 * # Safety
 * The parser pointer must be a valid KoiParser created with KoiParser_New.
 * The returned error pointer is owned by the caller and must be freed with
 * KoiParserError_Del when no longer needed.
 */
struct KoiParserError *KoiParser_Error(struct KoiParser *parser);

/**
 * Deletes a KoiParserError object and frees its memory
 *
 * # Arguments
 *
 * * `_this` - Pointer to the KoiParserError to delete. If NULL, the function does nothing.
 *
 * # Safety
 *
 * The pointer must either be NULL or point to a valid KoiParserError object
 * created by the parser functions. After calling this function, the pointer
 * becomes invalid and must not be used.
 */
void KoiParserError_Del(struct KoiParserError *_this);

/**
 * Formats the error message into a buffer
 *
 * This function creates a formatted error message including all available
 * error information and writes it to the provided buffer.
 *
 * # Arguments
 *
 * * `_this` - Pointer to the KoiParserError
 * * `buffer` - Buffer to write the formatted message to. If NULL, no data is written.
 * * `buffer_size` - Size of the buffer in bytes
 *
 * # Returns
 *
 * The total number of bytes required for the formatted message including the null terminator.
 * If the buffer is NULL or too small, no data is written and the required size is returned.
 *
 * # Safety
 *
 * The `_this` pointer must be either NULL or point to a valid KoiParserError.
 * If `buffer` is not NULL, it must point to a valid memory region of at least `buffer_size` bytes.
 */
uintptr_t KoiParserError_Format(const struct KoiParserError *_this,
                                char *buffer,
                                uintptr_t buffer_size);

/**
 * Gets the length of the formatted error message
 *
 * This function returns the number of bytes required to store the formatted
 * error message including the null terminator, without actually formatting it.
 *
 * # Arguments
 *
 * * `_this` - Pointer to the KoiParserError
 *
 * # Returns
 *
 * The number of bytes required for the formatted message including the null terminator.
 *
 * # Safety
 *
 * The `_this` pointer must be either NULL or point to a valid KoiParserError.
 */
uintptr_t KoiParserError_FormatLen(const struct KoiParserError *_this);

/**
 * Gets the error message text
 *
 * This function extracts just the message part of the error (without position
 * information) and writes it to the provided buffer.
 *
 * # Arguments
 *
 * * `_this` - Pointer to the KoiParserError
 * * `buffer` - Buffer to write the message to. If NULL, no data is written.
 * * `buffer_size` - Size of the buffer in bytes
 *
 * # Returns
 *
 * The total number of bytes required for the message including the null terminator.
 * If the buffer is NULL or too small, no data is written and the required size is returned.
 *
 * # Safety
 *
 * The `_this` pointer must be either NULL or point to a valid KoiParserError.
 * If `buffer` is not NULL, it must point to a valid memory region of at least `buffer_size` bytes.
 */
uintptr_t KoiParserError_GetMessage(const struct KoiParserError *_this,
                                    char *buffer,
                                    uintptr_t buffer_size);

/**
 * Gets the length of the error message
 *
 * This function returns the number of bytes required to store the error
 * message including the null terminator, without actually copying it.
 *
 * # Arguments
 *
 * * `_this` - Pointer to the KoiParserError
 *
 * # Returns
 *
 * The number of bytes required for the message including the null terminator.
 *
 * # Safety
 *
 * The `_this` pointer must be either NULL or point to a valid KoiParserError.
 */
uintptr_t KoiParserError_GetMessageLen(const struct KoiParserError *_this);

/**
 * Gets the position information from the error
 *
 * This function extracts the line and column position where the error occurred
 * in the source text.
 *
 * # Arguments
 *
 * * `_this` - Pointer to the KoiParserError
 * * `lineno` - Pointer to store the line number (1-based). If NULL, the value is not stored.
 * * `column` - Pointer to store the column number (1-based). If NULL, the value is not stored.
 *
 * # Returns
 *
 * 0 on success, -1 if the error does not contain position information or if `_this` is NULL.
 *
 * # Safety
 *
 * The `_this` pointer must be either NULL or point to a valid KoiParserError.
 * If `lineno` and `column` are not NULL, they must point to valid memory locations.
 */
int32_t KoiParserError_GetTracebackPosition(const struct KoiParserError *_this,
                                            uintptr_t *lineno,
                                            uintptr_t *column);

/**
 * Creates a new input source from a custom VTable implementation
 *
 * This function allows creating custom input sources by providing function pointers
 * for reading lines and getting the source name. This is useful for integrating
 * with existing C code or implementing special input handling.
 *
 * # Arguments
 *
 * * `vtable` - Pointer to a KoiTextInputSourceVTable structure containing function pointers
 * * `user_data` - User-provided data pointer that will be passed to the VTable functions
 *
 * # Returns
 *
 * Pointer to the created KoiInputSource, or NULL if vtable is NULL.
 *
 * # Safety
 *
 * The `vtable` pointer must be valid and remain valid until the input source is destroyed.
 * The function pointers in the vtable must follow the specified contracts.
 * The `user_data` pointer is passed directly to the VTable functions and is not managed by this library.
 */
struct KoiInputSource *KoiInputSource_FromVTable(const struct KoiTextInputSourceVTable *vtable,
                                                 void *user_data);

/**
 * Creates a new input source from a null-terminated C string
 *
 * This function creates an input source that will provide the specified string
 * to the parser. The string is copied and can be safely freed after this call.
 *
 * # Arguments
 *
 * * `source` - Pointer to a null-terminated UTF-8 C string containing the text to parse
 *
 * # Returns
 *
 * Pointer to the created KoiInputSource, or NULL if source is NULL or contains invalid UTF-8.
 *
 * # Safety
 *
 * The `source` pointer must be either NULL or point to a valid null-terminated C string.
 * The string should contain valid UTF-8, but invalid sequences will be replaced with the Unicode replacement character.
 */
struct KoiInputSource *KoiInputSource_FromString(const char *source);

/**
 * Creates a new input source from a file path
 *
 * This function creates an input source that will read from the specified file.
 * The file is opened with UTF-8 encoding and strict error handling.
 *
 * # Arguments
 *
 * * `path` - Pointer to a null-terminated C string containing the file path
 *
 * # Returns
 *
 * Pointer to the created KoiInputSource, or NULL if path is NULL, contains invalid UTF-8,
 * or the file cannot be opened.
 *
 * # Safety
 *
 * The `path` pointer must be either NULL or point to a valid null-terminated C string.
 * The file must exist and be readable with UTF-8 encoding.
 */
struct KoiInputSource *KoiInputSource_FromFile(const char *path);

/**
 * Creates a new input source from a file path with specific encoding
 *
 * This function creates an input source that will read from the specified file
 * using the specified text encoding and error handling strategy.
 *
 * # Arguments
 *
 * * `path` - Pointer to a null-terminated C string containing the file path
 * * `encoding` - Pointer to a null-terminated C string containing the encoding name
 * * `encoding_strategy` - Strategy for handling encoding errors
 *
 * # Returns
 *
 * Pointer to the created KoiInputSource, or NULL if path or encoding is NULL,
 * contains invalid UTF-8, the encoding is not recognized, or the file cannot be opened.
 *
 * # Safety
 *
 * The `path` and `encoding` pointers must be either NULL or point to valid null-terminated C strings.
 * The file must exist and be readable with the specified encoding.
 */
struct KoiInputSource *KoiInputSource_FromFileAndEncoding(const char *path,
                                                          const char *encoding,
                                                          enum KoiFileInputEncodingStrategy encoding_strategy);

/**
 * Deletes a KoiInputSource object and frees its memory
 *
 * # Arguments
 *
 * * `input` - Pointer to the KoiInputSource to delete. If NULL, the function does nothing.
 *
 * # Safety
 *
 * The pointer must either be NULL or point to a valid KoiInputSource object
 * created by one of the KoiInputSource_From* functions. After calling this function,
 * the pointer becomes invalid and must not be used.
 */
void KoiInputSource_Del(struct KoiInputSource *input);

/**
 * Initialize a KoiParserConfig with default values
 *
 * This function initializes a KoiParserConfig structure with sensible default values:
 * - command_threshold: 1 (lines starting with # are commands)
 * - skip_annotations: false (annotation lines are included in output)
 * - convert_number_command: true (numeric commands are converted to special commands)
 *
 * # Arguments
 * * `config` - Pointer to the KoiParserConfig structure to initialize
 *
 * # Safety
 * The config pointer must be a valid pointer to a KoiParserConfig structure.
 * If config is NULL, this function does nothing.
 */
void KoiParserConfig_Init(struct KoiParserConfig *config);

/**
 * Create a new Writer with custom output VTable.
 *
 * # Safety
 *
 * * `vtable` must be a valid pointer to a `KoiWriterOutputVTable`.
 * * `config` must be a valid pointer to a `KoiWriterConfig`.
 * * The returned pointer must be freed using `KoiWriter_Del`.
 */
struct KoiWriter *KoiWriter_NewFromVTable(const struct KoiWriterOutputVTable *vtable,
                                          void *user_data,
                                          const struct KoiWriterConfig *config);

/**
 * Create a new Writer that writes to a file.
 *
 * # Safety
 *
 * * `path` must be a valid null-terminated C string.
 * * `config` must be a valid pointer to a `KoiWriterConfig`.
 * * The returned pointer must be freed using `KoiWriter_Del`.
 */
struct KoiWriter *KoiWriter_NewFromFile(const char *path, const struct KoiWriterConfig *config);

/**
 * Create a new Writer that writes to a string output.
 *
 * # Safety
 *
 * * `output` must be a valid pointer to a `KoiStringOutput`.
 * * `config` must be a valid pointer to a `KoiWriterConfig`.
 * * The returned pointer must be freed using `KoiWriter_Del`.
 */
struct KoiWriter *KoiWriter_NewFromStringOutput(struct KoiStringOutput *output,
                                                const struct KoiWriterConfig *config);

/**
 * Delete Writer.
 *
 * Frees the memory allocated for the writer.
 *
 * # Safety
 *
 * * `writer` must be a valid pointer returned by one of the `KoiWriter_New*` functions.
 * * After calling this function, `writer` is invalid and must not be used.
 */
void KoiWriter_Del(struct KoiWriter *writer);

/**
 * Write a command.
 *
 * # Safety
 *
 * * `writer` must be a valid pointer to a `KoiWriter`.
 * * `command` must be a valid pointer to a `KoiCommand`.
 *
 * # Returns
 *
 * * 0 on success
 * * -1 if arguments are null
 * * -2 if writing fails
 */
int32_t KoiWriter_WriteCommand(struct KoiWriter *writer, const struct KoiCommand *command);

/**
 * Write a command with custom options.
 *
 * # Safety
 *
 * * `writer` must be a valid pointer to a `KoiWriter`.
 * * `command` must be a valid pointer to a `KoiCommand`.
 * * `options` can be null (uses defaults).
 * * `param_options` can be null (uses defaults).
 *
 * # Returns
 *
 * * 0 on success
 * * -1 if writer or command are null
 * * -2 if writing fails
 */
int32_t KoiWriter_WriteCommandWithOptions(struct KoiWriter *writer,
                                          const struct KoiCommand *command,
                                          const struct KoiFormatterOptions *options,
                                          const struct KoiParamOption *param_options);

/**
 * Increase indentation.
 *
 * # Safety
 *
 * * `writer` must be a valid pointer to a `KoiWriter`.
 */
void KoiWriter_IncIndent(struct KoiWriter *writer);

/**
 * Decrease indentation.
 *
 * # Safety
 *
 * * `writer` must be a valid pointer to a `KoiWriter`.
 */
void KoiWriter_DecIndent(struct KoiWriter *writer);

/**
 * Get current indentation.
 *
 * # Safety
 *
 * * `writer` must be a valid pointer to a `KoiWriter`.
 */
uintptr_t KoiWriter_GetIndent(const struct KoiWriter *writer);

/**
 * Write a newline.
 *
 * # Safety
 *
 * * `writer` must be a valid pointer to a `KoiWriter`.
 *
 * # Returns
 *
 * * 0 on success
 * * -1 if writer is null
 * * -2 if writing fails
 */
int32_t KoiWriter_Newline(struct KoiWriter *writer);

/**
 * Initialize KoiFormatterOptions with default values
 *
 * # Safety
 *
 * The pointer must be valid and writable.
 */
void KoiFormatterOptions_Init(struct KoiFormatterOptions *options);

/**
 * Initialize KoiWriterConfig with default values
 *
 * # Safety
 *
 * The pointer must be valid and writable.
 */
void KoiWriterConfig_Init(struct KoiWriterConfig *config);

/**
 * Create a new String Output.
 *
 * The returned object must be freed using `KoiStringOutput_Del`.
 *
 * # Returns
 *
 * A valid pointer to a `KoiStringOutput`.
 */
struct KoiStringOutput *KoiStringOutput_New(void);

/**
 * Delete String Output.
 *
 * Frees the memory allocated for the string output.
 *
 * # Safety
 *
 * * `output` must be a valid pointer returned by `KoiStringOutput_New`.
 * * After calling this function, `output` is invalid and must not be used.
 */
void KoiStringOutput_Del(struct KoiStringOutput *output);

/**
 * Get content of the string buffer.
 *
 * # Arguments
 *
 * * `output` - Pointer to the String Output object
 * * `buffer` - Pointer to destination buffer (can be NULL to query size)
 * * `buffer_len` - Size of the destination buffer
 *
 * # Returns
 *
 * The length of the string content PLUS null terminator.
 * If `buffer` is NULL or `buffer_len` is too small, returns required size.
 * If successful, copies content to `buffer` (null-terminated) and returns length.
 * Returns 0 if `output` is NULL.
 *
 * # Safety
 *
 * * `output` must be a valid pointer returned by `KoiStringOutput_New`.
 * * `buffer` must be a valid pointer to a writable buffer of at least `buffer_len` bytes.
 */
uintptr_t KoiStringOutput_GetString(struct KoiStringOutput *output,
                                    char *buffer,
                                    uintptr_t buffer_len);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus

#ifdef __cplusplus
}  // namespace koicore
#endif  // __cplusplus

#endif  /* _INCLUDE_KOICORE_ */
