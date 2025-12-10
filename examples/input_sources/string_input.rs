//! String Input Source Example
//!
//! This example demonstrates how to use StringInputSource to parse KoiLang scripts,
//! including dynamically generated scripts and multi-segment script processing

use koicore::parser::{Parser, ParserConfig, StringInputSource};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== String Input Source Example ===");
    
    // Example 1: Basic string parsing
    println!("\n--- Example 1: Basic String Parsing ---");
    let simple_script = r#"
#title "Simple Dialogue"
#character Alice "Hello, world!"
#character Bob "Hi Alice!"
    "#;
    
    let input1 = StringInputSource::new(simple_script);
    let config = ParserConfig::default();
    let mut parser1 = Parser::new(input1, config.clone());
    
    let mut count1 = 0;
    while let Some(command) = parser1.next_command()? {
        count1 += 1;
        println!("Command {}: {}", count1, command);
    }
    
    // Example 2: Dynamically generated script
    println!("\n--- Example 2: Dynamically Generated Script ---");
    let mut dynamic_script = String::new();
    dynamic_script.push_str("#title \"Dynamically Generated Script\"\n");
    
    // Dynamically add character dialogues
    let characters = vec!["Hero", "Villain", "Sidekick", "Mentor"];
    let dialogues = vec![
        "Our adventure begins!",
        "Ha ha ha, you're doomed!",
        "Don't worry, we have a plan.",
        "Remember, power comes from within."
    ];
    
    for (i, (character, dialogue)) in characters.iter().zip(dialogues.iter()).enumerate() {
        dynamic_script.push_str(&format!("#character {} \"{}\"\n", character, dialogue));
        if i < dialogues.len() - 1 {
            dynamic_script.push_str("This is a gap text.\n");
        }
    }
    
    println!("Generated script content:");
    println!("{}", dynamic_script);
    
    let input2 = StringInputSource::new(&dynamic_script);
    let mut parser2 = Parser::new(input2, config.clone());
    
    let mut count2 = 0;
    while let Some(command) = parser2.next_command()? {
        count2 += 1;
        println!("Command {}: {}", count2, command);
    }
    
    // Example 3: Multi-segment script processing
    println!("\n--- Example 3: Multi-segment Script Processing ---");
    let script_segments = vec![
        r#"
#chapter "Chapter 1: Beginning"
#scene "Forest Entrance"
#character Hero "Is this the legendary Dark Forest?"
        "#,
        r#"
#action set_time evening
#background Dark_Forest
#character Hero "I feel uneasy..."
#sound effect("wind")
        "#,
        r#"
#character Mysterious_Voice "Welcome to my domain..."
#character Hero "Who? Who's there?"
#action show_character name("Shadow Figure")
        "#
    ];
    
    let mut total_commands = 0;
    
    for (i, segment) in script_segments.iter().enumerate() {
        println!("\nProcessing script segment {}:", i + 1);
        let input = StringInputSource::new(segment);
        let mut parser = Parser::new(input, config.clone());
        
        let mut segment_commands = 0;
        while let Some(command) = parser.next_command()? {
            segment_commands += 1;
            total_commands += 1;
            println!("  {}", command);
        }
        
        println!("Segment {} contains {} commands", i + 1, segment_commands);
    }
    
    println!("\nTotal processed commands: {}", total_commands);
    
    // Example 4: Special characters and escaping
    println!("\n--- Example 4: Special Character Handling ---");
    let special_script = r#"
#title "Script with Special Characters"
#character Hero "This is a dialogue containing \"quotes\"."
#action escape_test "String containing \\ backslash"
#special_chars "Newline test\nThis is the second line" "Tab test\tThis is the second column"
    "#;
    
    let input4 = StringInputSource::new(special_script);
    let mut parser4 = Parser::new(input4, config);
    
    while let Some(command) = parser4.next_command()? {
        println!("Special character command: {}", command);
    }
    
    println!("\n=== String Input Source Summary ===");
    println!("StringInputSource is suitable for:");
    println!("- Dynamically generated script content");
    println!("- Quick testing of small scripts");
    println!("- Embedded script content");
    println!("- Scenarios requiring real-time script modification");
    
    Ok(())
}
