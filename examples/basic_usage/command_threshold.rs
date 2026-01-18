//! Command Threshold Example
//! 
//! This example demonstrates how the command_threshold configuration affects
//! the parsing of KoiLang scripts. The threshold determines how many '#' characters
//! are required to identify a command line.

use koicore::parser::{Parser, ParserConfig, StringInputSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Command Threshold Example ===");
    
    // Common script content used for all examples
    let koi_script = r#"#single_hash_command "This should be a command"
##double_hash_command "This could be a command or annotation"
###triple_hash_command "This could be a command or annotation"
####quadruple_hash_command "This could be a command or annotation"

Regular text content here.

#another_single_hash "Another command"
##another_double_hash "Another potential command"
    "#;
    
    println!("\nScript content being parsed:");
    println!("{}", koi_script);
    
    // Example 1: Default threshold (1)
    println!("\n--- Example 1: Default Threshold (command_threshold = 1) ---");
    println!("Rules: ");
    println!("  - # prefix = Command");
    println!("  - ##+ prefix = Annotation");
    println!("  - No # prefix = Text");
    
    let input1 = StringInputSource::new(koi_script);
    let config1 = ParserConfig::default(); // Default threshold is 1
    let mut parser1 = Parser::new(input1, config1);
    
    parse_and_display_commands(&mut parser1)?;
    
    // Example 2: Threshold = 2
    println!("\n--- Example 2: Threshold = 2 ---");
    println!("Rules: ");
    println!("  - # prefix = Text");
    println!("  - ## prefix = Command");
    println!("  - ###+ prefix = Annotation");
    
    let input2 = StringInputSource::new(koi_script);
    let config2 = ParserConfig::new(2, false, true, false, false);
    let mut parser2 = Parser::new(input2, config2);
    
    parse_and_display_commands(&mut parser2)?;
    
    // Example 3: Threshold = 3
    println!("\n--- Example 3: Threshold = 3 ---");
    println!("Rules: ");
    println!("  - # prefix = Text");
    println!("  - ## prefix = Text");
    println!("  - ### prefix = Command");
    println!("  - ####+ prefix = Annotation");
    
    let input3 = StringInputSource::new(koi_script);
    let config3 = ParserConfig::default()
        .with_command_threshold(3);
    let mut parser3 = Parser::new(input3, config3);
    
    parse_and_display_commands(&mut parser3)?;
    
    // Example 4: Threshold = 0 (Special Case)
    println!("\n--- Example 4: Threshold = 0 (Special Case) ---");
    println!("Rules: ");
    println!("  - No # prefix = Command (hash_count == 0 == threshold)");
    println!("  - #+ prefix = Annotation (hash_count > 0 == threshold)");
    println!("  - All lines are either commands or annotations, no text");
    println!("  - Non-hash lines must be valid command syntax");
    
    // Use a script with valid command syntax for threshold=0
    let script_for_threshold_0 = r#"#single_hash_command "This is an annotation"
##double_hash_command "This is also an annotation"
valid_command "This is a valid command"
another_command param1(123) param2("text")
third_command "Multiple" "parameters" "example"
    "#;
    
    let input4 = StringInputSource::new(script_for_threshold_0);
    let config4 = ParserConfig::new(0, false, true, false, false);
    let mut parser4 = Parser::new(input4, config4);
    
    parse_and_display_commands(&mut parser4)?;
    
    // Example 5: Threshold = 2 with skip_annotations = true
    println!("\n--- Example 5: Threshold = 2 with skip_annotations = true ---");
    println!("Rules: ");
    println!("  - # prefix = Text");
    println!("  - ## prefix = Command");
    println!("  - ###+ prefix = Skipped (not displayed)");
    
    let input5 = StringInputSource::new(koi_script);
    let config5 = ParserConfig::new(2, true, true, false, false);
    let mut parser5 = Parser::new(input5, config5);
    
    parse_and_display_commands(&mut parser5)?;
    
    // Example 6: Different thresholds with the same script
    println!("\n--- Example 6: Comparing Thresholds Side by Side ---");
    
    let test_script = r#"#one "Command?"
##two "Command?"
###three "Command?"
Regular text here
"#;
    
    for threshold in 0..=4 {
        let input = StringInputSource::new(test_script);
        let config = ParserConfig::new(threshold, false, true, false, false);
        let mut parser = Parser::new(input, config);
        
        print!("Threshold {}: ", threshold);
        let mut commands = Vec::new();
        while let Some(cmd) = parser.next_command()? {
            commands.push(cmd.name().to_string());
        }
        println!("{:?}", commands);
    }
    
    println!("\n=== Command Threshold Summary ===");
    println!("The command_threshold determines how KoiLang parses lines:");
    println!("  - Lines with < threshold # characters = Text");
    println!("  - Lines with = threshold # characters = Command");
    println!("  - Lines with > threshold # characters = Annotation");
    println!();
    println!("Use cases:");
    println!("  - threshold=0: Special case - all lines are commands or annotations");
    println!("  - threshold=1: Standard KoiLang syntax");
    println!("  - threshold=2: Compatible with markdown-style headings");
    println!("  - threshold=3: Strict command parsing");
    println!("  - skip_annotations=true: Ignore all annotations");
    
    Ok(())
}

/// Helper function to parse and display commands from a parser
fn parse_and_display_commands(parser: &mut Parser<StringInputSource>) -> Result<(), Box<dyn std::error::Error>> {
    let mut command_count = 0;
    let mut text_count = 0;
    let mut annotation_count = 0;
    
    while let Some(command) = parser.next_command()? {
        command_count += 1;
        let command_name = command.name();
        
        if command_name == "@text" {
            text_count += 1;
            println!("  Text: {:?}", command.params()[0]);
        } else if command_name == "@annotation" {
            annotation_count += 1;
            println!("  Annotation: {:?}", command.params()[0]);
        } else {
            println!("  Command: {} {:?}", command_name, command.params());
        }
    }
    
    println!();
    println!("  Summary: {}", format_summary(command_count, text_count, annotation_count));
    
    Ok(())
}

/// Helper function to format command count summary
fn format_summary(total: usize, text: usize, annotation: usize) -> String {
    let commands = total - text - annotation;
    format!("Total: {} | Commands: {} | Text: {} | Annotations: {}", 
           total, commands, text, annotation)
}