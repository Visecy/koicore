# koicore

KoiLang 语言核心模块，提供基础语言功能。

[![License](https://img.shields.io/github/license/Visecy/koicore.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/koicore.svg)](https://crates.io/crates/koicore)
[![Documentation](https://docs.rs/koicore/badge.svg)](https://docs.rs/koicore)

[English](./README.md) | **中文**

## 概述

KoiLang 是一种专为叙事内容设计的标记语言，特别适用于视觉小说、互动小说和对话驱动的应用程序。`koicore` crate 提供了处理 KoiLang 文件所需的基本解析和数据结构。

KoiLang 的核心理念是分离数据和指令。KoiLang 文件包含数据（命令和文本），而您的应用程序提供指令（如何处理这些命令）。这使得 KoiLang 文件易于人类阅读和编写，同时足够强大以处理复杂应用程序。

## 特性

- **流式解析器**：以恒定内存使用量处理任意大小的文件
- **多种输入源**：支持从字符串、文件或自定义输入源解析
- **编码支持**：通过 `DecodeBufReader` 处理各种文本编码（UTF-8、GBK 等）
- **全面的错误处理**：带有源位置和上下文的详细错误消息
- **可配置的解析**：可自定义的命令阈值和解析规则
- **类型安全的数据结构**：强类型的命令和参数表示
- **高性能**：基于 Rust 的性能和安全保证构建
- **跨语言 FFI**：C 兼容 API，支持与 C/C++ 和其他语言集成

## 安装

将此添加到您的 `Cargo.toml`：

```toml
[dependencies]
koicore = "0.1.3"
```

## 从源码构建

```bash
# 克隆仓库
git clone https://github.com/Visecy/koicore.git
cd koicore

# 构建项目
make build

# 运行测试
make test

# 运行FFI测试
make ffi-test
```

## 快速开始

```rust
use koicore::parser::{Parser, ParserConfig, StringInputSource};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
// 创建输入源
let input = StringInputSource::new(r#"
#character Alice "Hello, world!"
This is regular text content.
#background Forest
"#);

// 配置解析器
let config = ParserConfig::default();

// 创建解析器
let mut parser = Parser::new(input, config);

// 处理命令
while let Some(command) = parser.next_command()? {
    println!("Command: {}", command.name());
    for param in command.params() {
        println!("  Parameter: {}", param);
    }
}

# Ok(())
# }
```

## KoiLang 语法

KoiLang 使用基于命令前缀的简单、可读语法。核心概念是分离命令（指令）和文本内容（数据）。

### 命令
命令以 `#` 开头，后跟命令名称和参数：
```text
#character Alice "Hello, world!"
#background Forest
#action walk direction(left) speed(5)
```

### 文本内容
常规文本内容（没有 `#` 前缀）被视为叙述文本：
```text
This is regular text content.
It can span multiple lines.
```

### 注释
包含多个 `#` 字符的行被视为注释：
```text
## This is an annotation
### This is also an annotation
```

### 参数类型
KoiLang 支持各种参数类型：

#### 基本参数
- **整数**：十进制、二进制（`0b101`）和十六进制（`0x6CF`）
- **浮点数**：标准表示法（`1.0`）、科学记数法（`2e-2`）
- **字符串**：带引号的字符串（`"Hello world"`）
- **字面量**：不带引号的标识符（`string`、`__name__`）

```text
#arg_int    1 0b101 0x6CF
#arg_float  1. 2e-2 .114514
#arg_literal string __name__
#arg_string "A string"
```

#### 复合参数
- **命名参数**：`name(value)`
- **列表**：`name(item1, item2, item3)`
- **字典**：`name(key1: value1, key2: value2)`

```text
#kwargs key(value)
#keyargs_list key(item0, item1)
#kwargs_dict key(x: 11, y: 45, z: 14)
```

#### 复杂示例
所有参数类型都可以组合：
```text
#draw Line 2 pos0(x: 0, y: 0) pos1(x: 16, y: 16) \
    thickness(2) color(255, 255, 255)
```

### 命令名称
命令名称可以是：
- 有效标识符：`character`、`background`、`action`
- 数字命令：`#114`、`#1919`（用于编号序列很有用）

### 完整语法
在 KoiLang 中，文件包含'命令'部分和'文本'部分：
- 命令部分以 `#` 开头，遵循 C 风格的预处理语句格式
- 文本部分是所有其他不以 `#` 开头的行

单个命令的格式：
```text
#command_name [param 1] [param 2] ...
```

每个命令可以有多个不同类型的参数，允许灵活且富有表现力的命令结构。

## 核心组件

### 命令结构
`Command` 结构表示解析后的 KoiLang 命令：

```rust
use koicore::command::{Command, Parameter};

# fn main() {
// 创建简单命令
let cmd = Command::new("character", vec![
    Parameter::from("Alice"),
    Parameter::from("Hello, world!")
]);

// 创建文本和注释命令
let text_cmd = Command::new_text("Narrative text");
let annotation_cmd = Command::new_annotation("Annotation text");
# }
```

### 解析器配置
使用 `ParserConfig` 自定义解析行为：

```rust
use koicore::parser::ParserConfig;

# fn main() {
// 默认配置（阈值 = 1）
let config = ParserConfig::default();

// 自定义阈值 - 需要 2 个 # 字符作为命令
let config = ParserConfig::default().with_command_threshold(2);
# }
```

### 输入源
支持各种输入源：

```rust
use koicore::parser::{StringInputSource, FileInputSource};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// 从字符串解析
let input = StringInputSource::new("#test command");

// 从文件解析
let input = FileInputSource::new("script.ktxt")?;
# Ok(())
# }
```

## 高级特性

### 理念：数据与指令的分离

KoiLang 的关键创新是关注点分离：
- **KoiLang 文件** 包含数据（命令和文本内容）
- **您的应用程序** 提供指令（如何处理这些命令）

这使得 KoiLang 文件易于人类阅读和编写，而您的应用程序可以实现复杂的逻辑来处理它们。可以将其视为一个简单的虚拟机引擎，其中 KoiLang 文件是字节码，而您的应用程序是 VM。

### 流式处理大文件
高效处理大文件：

```rust
use koicore::parser::{Parser, ParserConfig, FileInputSource};
use std::path::Path;

let input = FileInputSource::new(Path::new("large_script.ktxt"))?;
let config = ParserConfig::default();
let mut parser = Parser::new(input, config);

// 逐行处理，恒定内存使用量
while let Some(command) = parser.next_command()? {
    // 解析时处理每个命令
    process_command(command)?;
}
```

### 编码支持
支持 UTF-8 编码的内容：

```rust
use koicore::parser::{Parser, ParserConfig, StringInputSource};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let input = StringInputSource::new(r#"
#title "Hello World"
#character Protagonist "Hello!"
"#);

let mut parser = Parser::new(input, ParserConfig::default());
while let Some(command) = parser.next_command()? {
    println!("Command: {}", command.name());
}
# Ok(())
# }
```

### 错误处理
带有上下文的全面错误报告：

```rust
use koicore::parser::{Parser, ParserConfig, StringInputSource};

let input = StringInputSource::new("#invalid command syntax");
let mut parser = Parser::new(input, ParserConfig::default());

match parser.next_command() {
    Ok(Some(command)) => println!("Parsed: {:?}", command),
    Ok(None) => println!("End of input"),
    Err(e) => {
        println!("Parse error: {}", e);
        println!("Error location: line {}", e.line_number());
    }
}
```

## 示例

### 文件生成示例

一个常见用例是使用 KoiLang 从单个源生成多个文件。以下是一个概念示例：

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

这种模式允许您：
- 从单个 KoiLang 源创建多个文件
- 分层组织内容
- 保持一致的编码和格式

### 查看 `examples/` 目录获取更多详细示例：

- `decode_buf_reader_example.rs` - 演示编码支持和流式功能
- `ktxt/example0.ktxt` - 复杂叙事脚本示例
- `ktxt/example1.ktxt` - 简单文件结构示例

## 性能

解析器专为高性能设计：

- **流式处理**：无论文件大小，恒定内存使用量
- **零拷贝解析**：解析期间最小字符串分配
- **高效错误处理**：快速错误检测和报告
- **基准测试**：`benches/` 目录中包含性能基准

运行基准测试：
```bash
cargo bench
```

## 与 Python Kola 的关系

koicore 与 Python Kola 之间的关系代表了 KoiLang 生态系统的演进：

1. **Kola** 是完整的第一代实现，提供解析器 + 写入器 + 上层封装。然而，它依赖于老旧的 flex 和 CPython API，使得 FFI 集成具有挑战性。

2. **koicore** 是新一代 KoiLang 内核，提供更高性能和跨语言的语言基础功能（解析器 + 写入器，其中写入器待实现）。新的 KoiLang Python 绑定将构建在 koicore 之上。

3. **未来演进**：Kola 将逐步采用 koicore 作为底层实现，并将被新的绑定逐步取代。

这一过渡确保了 KoiLang 生态系统更好的性能、改进的跨语言兼容性和更易维护的代码库。

## 跨语言集成

对于使用 C、C++ 或其他编程语言编写的应用程序，koicore 提供了全面的外部函数接口（FFI）。FFI 模块（`koicore_ffi`）通过 C 兼容 API 公开所有核心 koicore 功能。

### 主要 FFI 特性

- **C 兼容 API**：完整的 C API，支持 C++ 命名空间包装
- **内存管理**：明确的所有权和安全的内存管理
- **完整覆盖**：访问所有解析器和命令功能
- **错误处理**：详细的错误报告，包含源位置信息
- **多种输入源**：支持字符串、文件和自定义输入回调
- **复合参数**：完整支持列表和字典

### 使用 FFI

详细的 FFI 文档可在 [`crates/koicore_ffi/README.md`](./crates/koicore_ffi/README.md) 中找到。包括：

- **构建和链接**：完整的构建说明和链接示例
- **API 参考**：完整的 C API 文档和示例
- **快速开始指南**：C/C++ 中的基本解析示例
- **高级用法**：自定义输入源、复合参数和错误处理
- **内存管理**：安全内存使用指南
- **线程安全**：并发使用的最佳实践

### 快速 FFI 示例

```c
#include "koicore.h"
#include <stdio.h>

int main() {
    // 从字符串创建输入源
    KoiInputSource* source = KoiInputSource_FromString("#character Alice \"Hello!\"");
    
    // 初始化解析器配置
    KoiParserConfig config;
    KoiParserConfig_Init(&config);
    
    // 创建解析器并解析命令
    KoiParser* parser = KoiParser_New(source, &config);
    KoiCommand* cmd = KoiParser_NextCommand(parser);
    
    if (cmd) {
        char name[256];
        KoiCommand_GetName(cmd, name, sizeof(name));
        printf("Command: %s\n", name);
        KoiCommand_Del(cmd);
    }
    
    KoiParser_Del(parser);
    return 0;
}
```

有关全面的使用示例和 API 详细信息，请参阅 [FFI 文档](./crates/koicore_ffi/README.md)。

## 许可证

本项目采用 MIT 许可证 - 有关详细信息，请参阅 [LICENSE](LICENSE) 文件。

## 贡献

欢迎贡献！请随时提交拉取请求。对于重大更改，请先打开一个问题来讨论您想要更改的内容。

## 仓库

[https://github.com/Visecy/koicore](https://github.com/Visecy/koicore)
