//! Text vs Annotation Example
//!
//! This example demonstrates the difference between regular text content and annotations in KoiLang

use koicore::parser::{Parser, ParserConfig, StringInputSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Text vs Annotation Example ===");
    
    // Create input with text content and annotations
    let koi_script = r#"
#title "Text vs Annotation Demo"

## This is a single annotation line
### This is also an annotation line
#### And this is another annotation

Regular text content here.
This line is also regular text.

#background Forest

More text content after the background command.

#### This is still an annotation
##### And this too

More regular text content.

#character Alice "Hello!"
    "#;
    
    let input = StringInputSource::new(koi_script);
    let config = ParserConfig::default();
    let mut parser = Parser::new(input, config);
    
    let mut command_count = 0;
    let mut text_commands = 0;
    let mut annotation_commands = 0;
    
    while let Some(command) = parser.next_command()? {
        command_count += 1;
        let command_name = command.name();
        
        println!("Command #{}: {}", command_count, command_name);
        
        if command_name == "@text" {
            text_commands += 1;
            println!("  Type: Regular text content");
        } else if command_name == "@annotation" {
            annotation_commands += 1;
            println!("  Type: Annotation");
        } else {
            println!("  Type: Regular command");
        }
        
        // Print parameters if any
        if !command.params().is_empty() {
            println!("  Parameters:");
            for param in command.params() {
                println!("    {}", param);
            }
        }
    }
    
    println!("\nSummary:");
    println!("  Total commands: {}", command_count);
    println!("  Text commands: {}", text_commands);
    println!("  Annotation commands: {}", annotation_commands);
    println!("  Regular commands: {}", command_count - text_commands - annotation_commands);
    
    Ok(())
}
