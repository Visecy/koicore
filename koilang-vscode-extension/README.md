# KoiLang Language Support

This extension provides syntax highlighting for KoiLang, a markup language designed for narrative content, particularly visual novels and interactive fiction.

## Features

- Syntax highlighting for KoiLang commands (starting with `#`)
- Highlighting of parameters, strings, numbers, and boolean values
- Support for comments (both line comments `//` and block comments `/* */`)
- Recognition of text content outside commands
- Support for complex data structures like lists and dictionaries

## Supported Syntax

### Commands
Commands start with `#` followed by the command name:
```
#scene title="My Visual Novel" author="Author Name"
```

### Parameters
Parameters are specified as `name=value`:
```
#show char="main" position="center"
```

### Data Types
- **Strings**: `"Double quoted"` or `'single quoted'`
- **Numbers**: `123` (integer) or `123.45` (float)
- **Booleans**: `true`, `false`, `True`, `False`, `TRUE`, `FALSE`
- **Lists**: `["item1", "item2", 123]`
- **Dictionaries**: `{"key1": "value1", "key2": 123}`

### Comments
- Line comments: `// This is a comment`
- Block comments: `/* This is a block comment */`

## File Extension

This extension supports files with the `.koi` extension.

## Example

```koi
// This is a sample KoiLang file
#scene title="My Visual Novel" author="KoiLang Developer"

#char main name="Alice" color="#FF0000"
#char secondary name="Bob" color="#0000FF"

#show char="main" position="center"
Alice: "Hello there! Welcome to our visual novel."

#if mood == "happy" {
    #show effect="sparkle" duration=2.5
    Alice: "I'm feeling happy today!"
} else {
    #show effect="sad" duration=1.0
    Alice: "I'm not feeling so great..."
}
```

## Installation

1. Package the extension: `vsce package` (requires vsce to be installed: `npm install -g vsce`)
2. Install the generated `.vsix` file in VSCode through "Install from VSIX" option

## Contributing

Feel free to submit issues and enhancement requests via the GitHub repository.