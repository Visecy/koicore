//! BufRead Wrapper Example
//! 
//! This example demonstrates how to use BufReadWrapper to parse KoiLang scripts from
//! various BufRead implementations, including memory buffers and custom readers.

use koicore::parser::{Parser, ParserConfig};
use koicore::parser::input::BufReadWrapper;
use std::io::{self, BufRead, Cursor, Read};

/// A custom BufRead implementation that generates KoiLang commands dynamically
struct DynamicCommandGenerator {
    current_line: u32,
    max_lines: u32,
    buffer: Vec<u8>,
}

impl DynamicCommandGenerator {
    fn new(max_lines: u32) -> Self {
        Self {
            current_line: 0,
            max_lines,
            buffer: Vec::new(),
        }
    }
}

impl Read for DynamicCommandGenerator {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.current_line >= self.max_lines {
            return Ok(0); // End of input
        }
        
        // Generate a dynamic KoiLang command
        let line = if self.current_line == 0 {
            "#title \"Dynamically Generated Script\"\n".to_string()
        } else {
            format!("#character Character{} \"Line {} content\"\n", self.current_line, self.current_line)
        };
        
        self.current_line += 1;
        self.buffer = line.as_bytes().to_vec();
        
        let to_copy = std::cmp::min(buf.len(), self.buffer.len());
        buf[..to_copy].copy_from_slice(&self.buffer[..to_copy]);
        
        // Remove copied bytes from buffer
        self.buffer.drain(..to_copy);
        
        Ok(to_copy)
    }
}

impl BufRead for DynamicCommandGenerator {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        if self.buffer.is_empty() {
            // Generate next line if buffer is empty
            self.read(&mut [0; 4096])?;
        }
        Ok(&self.buffer)
    }
    
    fn consume(&mut self, amt: usize) {
        self.buffer.drain(..amt);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BufRead Wrapper Example ===");
    
    // Example 1: Using Cursor<Vec<u8>> (memory buffer)
    println!("\n--- Example 1: Cursor<Vec<u8>> (Memory Buffer) ---");
    let koi_content = br#"#title "Memory Buffer Test"
#character Alice "Hello from memory!"
#background Forest
This is regular text content.
#action walk speed(5)
    "#;
    
    let cursor = Cursor::new(koi_content.to_vec());
    let input1 = BufReadWrapper(cursor);
    let config = ParserConfig::default();
    let mut parser1 = Parser::new(input1, config.clone());
    
    while let Some(command) = parser1.next_command()? {
        println!("Command: {}", command);
    }
    
    // Example 2: Using custom BufRead implementation
    println!("\n--- Example 2: Custom BufRead Implementation ---");
    let dynamic_reader = DynamicCommandGenerator::new(5);
    let input2 = BufReadWrapper(dynamic_reader);
    let mut parser2 = Parser::new(input2, config.clone());
    
    while let Some(command) = parser2.next_command()? {
        println!("Command: {}", command);
    }
    
    // Example 3: Using stdin as BufRead (interactive mode simulation)
    println!("\n--- Example 3: String Buffer as BufRead ---");
    
    // Create a string buffer with KoiLang content
    let string_buffer = "#title \"String Buffer Test\"\n#character Bob \"Hi there!\"\n#action jump height(3)\n";
    
    // Convert string to Vec<u8> and wrap in Cursor
    let buffer = string_buffer.as_bytes().to_vec();
    let cursor = Cursor::new(buffer);
    let input3 = BufReadWrapper(cursor);
    let mut parser3 = Parser::new(input3, config.clone());
    
    while let Some(command) = parser3.next_command()? {
        println!("Command: {}", command);
    }
    
    // Example 4: Using BufRead for large text blocks
    println!("\n--- Example 4: Large Text Block Processing ---");
    
    // Create a large text block with multiple commands
    let mut large_content = String::new();
    large_content.push_str("#title \"Large Content Test\"\n");
    
    for i in 1..=10 {
        large_content.push_str(&format!("#character Char{} \"Message {}\"\n", i, i));
        large_content.push_str(&format!("This is text between command {}.\n", i));
    }
    
    let cursor = Cursor::new(large_content.into_bytes());
    let input4 = BufReadWrapper(cursor);
    let mut parser4 = Parser::new(input4, config);
    
    let mut command_count = 0;
    let mut text_count = 0;
    
    while let Some(command) = parser4.next_command()? {
        command_count += 1;
        if command.name() == "@text" {
            text_count += 1;
        }
    }
    
    println!("Total commands processed: {}", command_count);
    println!("Text commands: {}", text_count);
    println!("Regular commands: {}", command_count - text_count);
    
    println!("\n=== BufRead Wrapper Summary ===");
    println!("BufReadWrapper is suitable for:");
    println!("- Reading from memory buffers (Cursor<Vec<u8>>)");
    println!("- Custom BufRead implementations");
    println!("- Network streams (TcpStream, UnixStream)");
    println!("- Any other type that implements BufRead");
    println!("- Dynamic command generation");
    
    Ok(())
}