# KoiLang VSCode Extension Development Guide

This document provides information about the KoiLang VSCode extension and how to develop and maintain it.

## Project Structure

```
koilang-vscode-extension/
├── package.json          # Extension manifest
├── language-configuration.json  # Language configuration (comments, brackets, etc.)
├── syntaxes/
│   └── koilang.json      # Syntax highlighting grammar
├── README.md             # User documentation
├── CHANGELOG.md          # Version history
├── DEVELOPMENT.md        # This file
├── build.sh              # Build script
├── example.koi           # Example KoiLang file
└── test.koi              # Test file for syntax highlighting
```

## Syntax Highlighting

The syntax highlighting is defined in `syntaxes/koilang.json` using TextMate grammar syntax. The main components are:

### Commands
- Pattern: `^\\s*(#)\\s*([a-zA-Z_][a-zA-Z0-9_]*)`
- Highlights the `#` symbol and command name
- Handles parameters, strings, numbers, literals, lists, and dictionaries within commands

### Parameters
- Pattern: `\\b([a-zA-Z_][a-zA-Z0-9_]*)\\s*(=)\\s*`
- Highlights parameter names and assignment operators

### Data Types
- **Strings**: Both double-quoted and single-quoted strings with escape character support
- **Numbers**: Integers and floating-point numbers
- **Booleans**: true/false in various cases
- **Lists**: Arrays with nested support
- **Dictionaries**: Key-value objects with nested support

### Comments
- Line comments: `// comment`
- Block comments: `/* comment */`

### Text Content
- Content not starting with `#` or `//` is treated as text content

## Development Workflow

1. Make changes to the grammar in `syntaxes/koilang.json`
2. Test with example files (`example.koi` and `test.koi`)
3. Reload VSCode window (Ctrl+Shift+P → "Developer: Reload Window")
4. Verify syntax highlighting works correctly
5. Update documentation if needed

## Testing

The extension includes two test files:
- `example.koi`: A comprehensive example covering various syntax features
- `test.koi`: A focused test file for specific syntax highlighting scenarios

## Building

To build the extension package:
1. Run `./build.sh` (or `bash build.sh`)
2. The script will install vsce if needed and package the extension as a .vsix file

## Publishing

To publish to the VSCode marketplace:
1. Ensure you have the vsce tool installed: `npm install -g vsce`
2. Run `vsce publish` from the extension directory
3. You'll need appropriate credentials for the marketplace

## Troubleshooting

### Syntax Highlighting Not Working
- Ensure file extension is .koi
- Reload VSCode window after making changes
- Check developer tools (Ctrl+Shift+P → "Developer: Toggle Developer Tools") for errors

### Grammar Issues
- Validate JSON syntax in the grammar file
- Use the "Developer: Inspect Editor Tokens and Scopes" command to debug tokenization