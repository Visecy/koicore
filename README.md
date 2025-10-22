# koicore

Core KoiLang language module providing basic language features.

[![License](https://img.shields.io/github/license/Visecy/koicore.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/koicore.svg)](https://crates.io/crates/koicore)
[![Documentation](https://docs.rs/koicore/badge.svg)](https://docs.rs/koicore)

**English** | [中文](./README_cn.md)

## Overview

KoiLang is a markup language designed for narrative content, particularly suited for visual novels, interactive fiction, and dialogue-driven applications. The `koicore` crate provides the fundamental parsing and data structures needed to work with KoiLang files.

The core idea of KoiLang is to separate data and instructions. KoiLang files contain the data (commands and text), while your application provides the instructions (how to handle those commands). This makes KoiLang files easy to read and write for humans, while being powerful enough for complex applications.

## Features

- **Streaming Parser**: Process files of any size with constant memory usage
- **Multiple Input Sources**: Parse from strings, files, or custom input sources
- **Encoding Support**: Handle various text encodings (UTF-8, GBK, etc.) through `DecodeBufReader`
- **Comprehensive Error Handling**: Detailed error messages with source locations and context
- **Configurable Parsing**: Customizable command thresholds and parsing rules
- **Type-Safe Data Structures**: Strongly typed command and parameter representations
- **High Performance**: Built with Rust's performance and safety guarantees

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
koicore = "0.1.0"
```

## Quick Start

```rust
use koicore::parser::{Parser, ParserConfig, StringInputSource};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
// Create input source
let input = StringInputSource::new(r#"
#character Alice "Hello, world!"
This is regular text content.
#background Forest
"#);

// Configure parser
let config = ParserConfig::default();

// Create parser
let mut parser = Parser::new(input, config);

// Process commands
while let Some(command) = parser.next_command()? {
    println!("Command: {}", command.name());
    for param in command.params() {
        println!("  Parameter: {}", param);
    }
}

Ok(())
# }
```

## KoiLang Syntax

KoiLang uses a simple, readable syntax based on command prefixes. The core concept is separating commands (instructions) from text content (data).

### Commands
Commands start with `#` followed by the command name:

```text
#character Alice "Hello, world!"
#background Forest
#action walk direction(left) speed(5)
```

### Text Content
Plain text content without commands:

```text
This is regular text content.
It can span multiple lines.

Another paragraph of text.
```

### Annotations
Lines with multiple `#` characters are treated as annotations:
```text
## This is an annotation
### This is also an annotation
```

### Parameter Types
KoiLang supports various parameter types:

#### Basic Parameters
- **Integers**: Decimal, binary (`0b101`), and hexadecimal (`0x6CF`)
- **Floats**: Standard notation (`1.0`), scientific notation (`2e-2`)
- **Strings**: Quoted strings (`"Hello world"`)
- **Literals**: Unquoted identifiers (`string`, `__name__`)

```text
#arg_int    1 0b101 0x6CF
#arg_float  1. 2e-2 .114514
#arg_literal string __name__
#arg_string "A string"
```

#### Composite Parameters
- **Named parameters**: `name(value)`
- **Lists**: `name(item1, item2, item3)`
- **Dictionaries**: `name(key1: value1, key2: value2)`

```text
#kwargs key(value)
#keyargs_list key(item0, item1)
#kwargs_dict key(x: 11, y: 45, z: 14)
```

#### Complex Example
All parameter types can be combined:
```text
#draw Line 2 pos0(x: 0, y: 0) pos1(x: 16, y: 16) thickness(2) color(255, 255, 255)
```

### Command Names
Command names can be:
- Valid identifiers: `character`, `background`, `action`
- Numeric commands: `#114`, `#1919` (useful for numbered sequences)

### Complete Grammar
In KoiLang, files contain 'command' sections and 'text' sections:
- Command sections start with `#` and follow C-style prepared statement format
- Text sections are all other lines that don't start with `#`

The format of a single command:
```text
#command_name [param 1] [param 2] ...
```

Each command can have multiple parameters of different types, allowing for flexible and expressive command structures.

## Core Components

### Command Structure
The `Command` struct represents parsed KoiLang commands:

```rust
use koicore::command::{Command, Parameter};

# fn main() {
// Create a simple command
let cmd = Command::new("character", vec![
    Parameter::from("Alice"),
    Parameter::from("Hello, world!")
]);

// Create text and annotation commands
let text_cmd = Command::new_text("Narrative text");
let annotation_cmd = Command::new_annotation("Annotation text");
# }
```

### Parser Configuration
Customize parsing behavior with `ParserConfig`:

```rust
use koicore::parser::ParserConfig;
# fn main() {
// Default configuration (threshold = 1)
let config = ParserConfig::default();

// Custom threshold - require 2 # characters for commands
let config = ParserConfig { 
    command_threshold: 2,
    skip_annotations: false 
};
# }
```

### Input Sources
Support for various input sources:

```rust
use koicore::parser::{StringInputSource, FileInputSource};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// Parse from string
let input = StringInputSource::new("#test command");

// Parse from file (file must exist)
# std::fs::write("temp_script.ktxt", "#test command").unwrap();
let input = FileInputSource::new("temp_script.ktxt")?;
# std::fs::remove_file("temp_script.ktxt").unwrap();
# Ok(())
# }
```

## Advanced Features

### Philosophy: Data vs Instructions

The key innovation of KoiLang is the separation of concerns:
- **KoiLang files** contain the data (commands and text content)
- **Your application** provides the instructions (how to handle those commands)

This makes KoiLang files human-readable and easy to write, while your application can implement complex logic to process them. Think of it as a simple virtual machine engine where KoiLang files are the bytecode and your application is the VM.

### Streaming Large Files
Process massive files efficiently:

```rust
use koicore::parser::{Parser, ParserConfig, StringInputSource};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// For demonstration, using string input - in practice use FileInputSource
let input = StringInputSource::new(r#"
#character Alice "Hello, world!"
#background Forest
#action walk direction(left) speed(5)
"#);
let config = ParserConfig::default();
let mut parser = Parser::new(input, config);

// Process line by line with constant memory usage
while let Some(command) = parser.next_command()? {
    // Handle each command as it's parsed
    println!("Processed command: {}", command.name());
}
# Ok(())
# }
```

### Encoding Support
Handle various text encodings:

```rust
use koicore::parser::{Parser, ParserConfig, StringInputSource};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// For demonstration, using string input with UTF-8 content
let input = StringInputSource::new("#title \"Hello World\"");
let config = ParserConfig::default();
let mut parser = Parser::new(input, config);

// Parser automatically handles UTF-8 encoding
while let Some(command) = parser.next_command()? {
    println!("Command: {}", command.name());
}
# Ok(())
# }
```

### Error Handling
Comprehensive error reporting with context:

```rust
use koicore::parser::{Parser, ParserConfig, StringInputSource};

# fn main() {
let input = StringInputSource::new("#invalid command syntax");
let mut parser = Parser::new(input, ParserConfig::default());

match parser.next_command() {
    Ok(Some(command)) => println!("Parsed: {:?}", command),
    Ok(None) => println!("End of input"),
    Err(e) => {
        println!("Parse error: {}", e);
        if let Some(line_num) = e.line() {
            println!("Error location: line {}", line_num);
        }
    }
}
# }
```

## Examples

### File Generation Example

A common use case is using KoiLang to generate multiple files from a single source. Here's a conceptual example:

```text
#file "hello.txt" encoding("utf-8")
Hello world!
And there are all my friends.

#space hello
    #file "Bob.txt"
    Hello Bob.

    #file "Alice.txt"
    Hello Alice.
#endspace
```

This pattern allows you to:
- Create multiple files from a single KoiLang source
- Organize content hierarchically
- Maintain consistent encoding and formatting

### Check the `examples/` directory for more detailed examples:

- `decode_buf_reader_example.rs` - Demonstrates encoding support and streaming capabilities
- `ktxt/example0.ktxt` - Complex narrative script example
- `ktxt/example1.ktxt` - Simple file structure example

## Performance

The parser is designed for high performance:

- **Streaming processing**: Constant memory usage regardless of file size
- **Zero-copy parsing**: Minimal string allocations during parsing
- **Efficient error handling**: Fast error detection and reporting
- **Benchmarked**: Includes performance benchmarks in the `benches/` directory

Run benchmarks:
```bash
cargo bench
```

## Relationship to Python Kola

The relationship between koicore and Python Kola represents an evolution of the KoiLang ecosystem:

1. **Kola** is the complete first-generation implementation providing parser + writer + high-level abstractions. However, it relies on legacy flex and CPython APIs, making FFI integration challenging.

2. **koicore** is the next-generation KoiLang kernel, delivering higher performance and cross-language language fundamentals (parser + writer, with writer implementation pending). New KoiLang Python bindings will be built on top of koicore.

3. **Future Evolution**: Kola will gradually adopt koicore as its underlying implementation and will be progressively replaced by the new bindings.

This transition ensures better performance, improved cross-language compatibility, and a more maintainable codebase for the KoiLang ecosystem.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## Repository

[https://github.com/Visecy/koicore](https://github.com/Visecy/koicore)
