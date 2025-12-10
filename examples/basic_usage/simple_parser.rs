//! Basic Parser Usage Example
//!
//! This example demonstrates how to use the KoiLang parser to parse simple commands and text content

use koicore::parser::{Parser, ParserConfig, StringInputSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic Parser Usage Example ===");
    
    // Create an input string containing commands and text
    let koi_script = r#"
#title "My First KoiLang Script"
#character Alice "Hello, world!"
This is regular text content.
#background Forest
#action walk speed(5)
    "#;
    
    // Create input source
    let input = StringInputSource::new(koi_script);
    
    // Create parser configuration (using default configuration)
    let config = ParserConfig::default();
    
    // Create parser
    let mut parser = Parser::new(input, config);
    
    // Parse commands one by one
    let mut command_count = 0;
    while let Some(command) = parser.next_command()? {
        command_count += 1;
        
        println!("Command #{}: {}", command_count, command.name());
        
        // Print all parameters
        for param in command.params() {
            println!("  Parameter: {}", param);
        }
    }
    
    println!("Total commands parsed: {}", command_count);
    
    Ok(())
}
