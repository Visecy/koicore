//! File Input Source Example
//! 
//! This example demonstrates how to use FileInputSource to parse KoiLang scripts from files,
//! including different encodings and error handling strategies.

use koicore::parser::{Parser, ParserConfig, FileInputSource};
use koicore::parser::input::EncodingErrorStrategy;
use encoding_rs::{GBK, SHIFT_JIS};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== File Input Source Example ===");
    
    // Example 1: Basic file parsing (UTF-8)
    println!("\n--- Example 1: Basic UTF-8 File Parsing ---");
    let input1 = FileInputSource::new("examples/ktxt/example0.ktxt")?;
    let config = ParserConfig::default();
    let mut parser1 = Parser::new(input1, config.clone());
    
    let mut count1 = 0;
    while let Some(command) = parser1.next_command()? {
        count1 += 1;
        if count1 <= 5 { // Only show first 5 commands
            println!("Command {}: {}", count1, command);
        }
    }
    println!("Total commands parsed: {}", count1);
    
    // Example 2: Reading GBK encoded file
    println!("\n--- Example 2: GBK Encoded File ---");
    let input2 = FileInputSource::with_encoding(
        "examples/ktxt/example0_gbk.ktxt", 
        Some(GBK),
        EncodingErrorStrategy::Replace
    )?;
    let mut parser2 = Parser::new(input2, config.clone());
    
    let mut count2 = 0;
    while let Some(command) = parser2.next_command()? {
        count2 += 1;
        if count2 <= 5 { // Only show first 5 commands
            println!("Command {}: {}", count2, command);
        }
    }
    println!("Total commands parsed from GBK file: {}", count2);
    
    // Example 3: Reading Shift-JIS encoded file
    println!("\n--- Example 3: Shift-JIS Encoded File ---");
    let input3 = FileInputSource::with_encoding(
        "examples/ktxt/example0_shift_jis.ktxt", 
        Some(SHIFT_JIS),
        EncodingErrorStrategy::Replace
    )?;
    let mut parser3 = Parser::new(input3, config.clone());
    
    let mut count3 = 0;
    while let Some(command) = parser3.next_command()? {
        count3 += 1;
        if count3 <= 5 { // Only show first 5 commands
            println!("Command {}: {}", count3, command);
        }
    }
    println!("Total commands parsed from Shift-JIS file: {}", count3);
    
    // Example 4: Different encoding error strategies
    println!("\n--- Example 4: Encoding Error Strategies ---");
    
    // Create a simple test file with mixed encoding
    let test_file_content = "#title \"Mixed Encoding Test\"
#character Alice \"Hello\"
#character Bob \"こんにちは\"  # Japanese
#character Charlie \"你好\"  # Chinese\n";
    
    // Write test file (in practice, you'd use an existing file)
    std::fs::write("test_encoding.txt", test_file_content)?;
    
    // Test with different error strategies
    let strategies = [
        (EncodingErrorStrategy::Strict, "Strict"),
        (EncodingErrorStrategy::Replace, "Replace"),
        (EncodingErrorStrategy::Ignore, "Ignore"),
    ];
    
    for (strategy, name) in strategies {
        println!("\nUsing {} strategy:", name);
        let input = FileInputSource::with_encoding(
            "test_encoding.txt",
            Some(SHIFT_JIS), // Deliberately use wrong encoding to test error handling
            strategy
        )?;
        let mut parser = Parser::new(input, config.clone());
        
        match parser.next_command() {
            Ok(Some(command)) => println!("  First command: {}", command),
            Ok(None) => println!("  No commands found"),
            Err(e) => println!("  Error: {}", e),
        }
    }
    
    // Clean up test file
    std::fs::remove_file("test_encoding.txt")?;
    
    // Example 5: Reading a small example file
    println!("\n--- Example 5: Small Example File ---");
    let input5 = FileInputSource::new("examples/ktxt/example1.ktxt")?;
    let mut parser5 = Parser::new(input5, config);
    
    while let Some(command) = parser5.next_command()? {
        println!("Command: {}", command);
    }
    
    println!("\n=== File Input Source Summary ===");
    println!("FileInputSource is suitable for:");
    println!("- Reading KoiLang scripts from actual files");
    println!("- Handling various text encodings");
    println!("- Processing large files with streaming support");
    println!("- Customizable encoding error handling");
    
    Ok(())
}