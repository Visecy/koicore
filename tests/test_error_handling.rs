//! Tests for improved error handling in KoiLang parsing

use koicore::parser::{Parser, ParserConfig, StringInputSource, ParseError};

#[test]
fn test_syntax_error_with_context() {
    // Test empty command line error
    let input = StringInputSource::new("#");
    let mut parser = Parser::new(input, ParserConfig::default());
    
    let result = parser.next_command();
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    match error {
        ParseError::SyntaxError { message, line, column, context, .. } => {
            assert_eq!(line, 1);
            assert_eq!(column, 0);
            assert_eq!(context, "");
            assert!(message.contains("Empty command line"));
        }
        _ => panic!("Expected SyntaxError, got {:?}", error),
    }
}

#[test]
fn test_unexpected_input_error() {
    // Test unexpected input error with precise column tracking
    let input = StringInputSource::new("#command $$$ invalid");
    let mut parser = Parser::new(input, ParserConfig::default());
    
    let result = parser.next_command();
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    match error {
        ParseError::SyntaxError { message, line, column, context, .. } => {
            assert_eq!(line, 1);
            assert!(column > 0); // Should have precise column information
            assert_eq!(context, "command $$$ invalid");
            assert!(message.contains("Unexpected input"));
        }
        _ => panic!("Expected SyntaxError, got {:?}", error),
    }
}

#[test]
fn test_error_position_tracking() {
    // Test that errors provide accurate position information
    let test_cases = vec![
        ("#", "Empty command line"),
        ("#cmd $$$", "Unexpected input"),
    ];
    
    for (input_text, _expected_error_type) in test_cases {
        let input = StringInputSource::new(input_text);
        let mut parser = Parser::new(input, ParserConfig::default());
        
        let result = parser.next_command();
        if let Err(error) = result {
            // All errors should provide position information
            let position = error.position();
            assert!(position.is_some(), "Error should provide position: {:?}", error);
            
            let (line, _column) = position.unwrap();
            assert_eq!(line, 1, "Error should be on line 1: {:?}", error);
        }
    }
}

#[test]
fn test_error_display_formatting() {
    // Test that errors display with proper formatting and context
    let input = StringInputSource::new("#");
    let mut parser = Parser::new(input, ParserConfig::default());
    
    let result = parser.next_command();
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    let error_string = format!("{}", error);
    
    // Error should contain line and column information
    assert!(error_string.contains("line 1"));
    assert!(error_string.contains("column 0"));
    assert!(error_string.contains("Empty command line"));
    
    println!("Error display: {}", error_string);
}
