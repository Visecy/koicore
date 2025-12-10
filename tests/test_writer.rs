use koicore::{Command, Parameter, Writer, WriterConfig, FormatterOptions, parser::{Parser, ParserConfig, StringInputSource}};
use koicore::writer::{ParamFormatSelector, NumberFormat};

use std::collections::HashMap;

// Test Writer-Parser compatibility for various formatting options
#[test]
fn test_writer_parser_compatibility() {
    let test_cases = vec![
        Command::new("test1", vec![Parameter::from("string"), Parameter::from(42), Parameter::from(3.14)]),
        Command::new("test2", vec![Parameter::from("no_quotes"), Parameter::from(0xff)]),
        Command::new("test3", vec![Parameter::from("with spaces"), Parameter::from(1000)]),
    ];
    
    let config = WriterConfig::default();
    
    // Test with default options
    for command in test_cases {
        // Generate text with writer
        let mut output = Vec::new();
        let mut writer = Writer::new(&mut output, config.clone());
        writer.write_command(&command).expect("Failed to write command");
        let generated = String::from_utf8(output).unwrap();
        
        // Parse it back directly
        let input = StringInputSource::new(generated.as_str());
        let parser_config = ParserConfig::default();
        let mut parser = Parser::new(input, parser_config);
        
        let parsed = parser.next_command();
        assert!(parsed.is_ok(), "Failed to parse generated command: {}", generated);
        let parsed_command = parsed.unwrap();
        assert!(parsed_command.is_some(), "No command parsed from: {}", generated);
        let parsed_command = parsed_command.unwrap();
        
        // Check if the parsed command matches the original
        assert_eq!(parsed_command.name(), command.name());
        assert_eq!(parsed_command.params.len(), command.params.len());
    }
}

// Test Writer-Parser compatibility with parameter-specific formatting
#[test]
fn test_writer_parser_param_specific() {
    let command = Command::new("param_test", vec![
        Parameter::from(42),
        Parameter::from(255),
        Parameter::from(100)
    ]);
    
    let config = WriterConfig::default();
    let mut output = Vec::new();
    let mut writer = Writer::new(&mut output, config.clone());
    
    // Set different formats for different parameters
    let mut param_options = HashMap::new();
    
    // First parameter in hex
    let mut hex_options = FormatterOptions::default();
    hex_options.number_format = NumberFormat::Hex;
    param_options.insert(ParamFormatSelector::Position(0), hex_options);
    
    // Second parameter in binary with space before
    let mut bin_options = FormatterOptions::default();
    bin_options.number_format = NumberFormat::Binary;
    param_options.insert(ParamFormatSelector::Position(1), bin_options);
    
    // Third parameter in octal
    let mut oct_options = FormatterOptions::default();
    oct_options.number_format = NumberFormat::Octal;
    param_options.insert(ParamFormatSelector::Position(2), oct_options);
    
    writer.write_command_with_options(&command, None, Some(&param_options)).expect("Failed to write with parameter-specific options");
    let generated = String::from_utf8(output).unwrap();
    
    // Parse it back
    let input = StringInputSource::new(generated.as_str());
    let parser_config = ParserConfig::default();
    let mut parser = Parser::new(input, parser_config);
    
    let parsed = parser.next_command();
    assert!(parsed.is_ok(), "Failed to parse generated command: {}", generated);
    let parsed_command = parsed.unwrap();
    assert!(parsed_command.is_some(), "No command parsed from: {}", generated);
    let parsed_command = parsed_command.unwrap();
    
    // Check if the parsed command matches the original
    assert_eq!(parsed_command.name(), command.name());
    assert_eq!(parsed_command.params.len(), command.params.len());
}

// Test Writer-Parser compatibility with newline formatting
#[test]
fn test_writer_parser_newlines() {
    let command = Command::new("newline_test", vec![
        Parameter::from("param1"),
        Parameter::from("param2"),
        Parameter::from("param3")
    ]);
    
    let config = WriterConfig::default();
    let mut output = Vec::new();
    let mut writer = Writer::new(&mut output, config.clone());
    
    // Set newline after each parameter
    let mut param_options = HashMap::new();
    
    let mut newline_options = FormatterOptions::default();
    newline_options.newline_after_param = true;
    
    param_options.insert(ParamFormatSelector::Position(0), newline_options.clone());
    param_options.insert(ParamFormatSelector::Position(1), newline_options.clone());
    param_options.insert(ParamFormatSelector::Position(2), newline_options);
    
    writer.write_command_with_options(&command, None, Some(&param_options)).expect("Failed to write with newlines");
    let generated = String::from_utf8(output).unwrap();
    
    // Parse it back
    let input = StringInputSource::new(generated.as_str());
    let parser_config = ParserConfig::default();
    let mut parser = Parser::new(input, parser_config);
    
    // Check that we can parse the command without errors
    let parsed = parser.next_command();
    assert!(parsed.is_ok(), "Failed to parse generated command with newlines: {}", generated);
    let parsed_command = parsed.unwrap();
    assert!(parsed_command.is_some(), "No command parsed from: {}", generated);
    let parsed_command = parsed_command.unwrap();
    
    // Check if the parsed command matches the original
    assert_eq!(parsed_command.name(), command.name());
}

// Test Writer-Parser compatibility with compact formatting
#[test]
fn test_writer_parser_compact() {
    let command = Command::new("compact_test", vec![
        Parameter::from("string"),
        Parameter::from(42),
        Parameter::from(3.14)
    ]);
    
    let config = WriterConfig::default();
    let mut output = Vec::new();
    let mut writer = Writer::new(&mut output, config.clone());
    
    let mut compact_options = FormatterOptions::default();
    compact_options.compact = true;
    writer.write_command_with_options(&command, Some(&compact_options), None).expect("Failed to write compact");
    let generated = String::from_utf8(output).unwrap();
    
    // Parse it back
    let input = StringInputSource::new(generated.as_str());
    let parser_config = ParserConfig::default();
    let mut parser = Parser::new(input, parser_config);
    
    let parsed = parser.next_command();
    assert!(parsed.is_ok(), "Failed to parse generated compact command: {}", generated);
    let parsed_command = parsed.unwrap();
    assert!(parsed_command.is_some(), "No command parsed from compact output: {}", generated);
    let parsed_command = parsed_command.unwrap();
    
    // Check if the parsed command matches the original
    assert_eq!(parsed_command.name(), command.name());
    assert_eq!(parsed_command.params.len(), command.params.len());
}

// Test Writer-Parser compatibility with force quotes
#[test]
fn test_writer_parser_force_quotes() {
    let command = Command::new("quote_test", vec![
        Parameter::from("valid_name"),
        Parameter::from("valid123"),
        Parameter::from("invalid-name")
    ]);
    
    let config = WriterConfig::default();
    let mut output = Vec::new();
    let mut writer = Writer::new(&mut output, config.clone());
    
    let mut force_quote_options = FormatterOptions::default();
    force_quote_options.force_quotes_for_vars = true;
    writer.write_command_with_options(&command, Some(&force_quote_options), None).expect("Failed to write with force quotes");
    let generated = String::from_utf8(output).unwrap();
    
    // Parse it back
    let input = StringInputSource::new(generated.as_str());
    let parser_config = ParserConfig::default();
    let mut parser = Parser::new(input, parser_config);
    
    let parsed = parser.next_command();
    assert!(parsed.is_ok(), "Failed to parse generated command with force quotes: {}", generated);
    let parsed_command = parsed.unwrap();
    assert!(parsed_command.is_some(), "No command parsed from: {}", generated);
    let parsed_command = parsed_command.unwrap();
    
    // Check if the parsed command matches the original
    assert_eq!(parsed_command.name(), command.name());
    assert_eq!(parsed_command.params.len(), command.params.len());
}