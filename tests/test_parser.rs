use std::path::Path;

use koicore::{command, parser::{self, input::EncodingErrorStrategy}};

#[test]
fn test_early_stop() {
    let input = parser::StringInputSource::new("#cmd1\n#cmd2\n#cmd3");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    let mut count = 0;
    let reached_eof = parser
        .process_with(|_cmd| {
            count += 1;
            // stop after first command
            Ok::<bool, Box<parser::ParseError>>(false)
        })
        .expect("Failed to process");
    assert_eq!(count, 1);
    assert!(!reached_eof, "Should have stopped early");
}

#[test]
fn test_parse_hello_world() {
    let input = parser::StringInputSource::new("#hello world\nThis is a text.");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    let cmd = parser.next_command().unwrap();
    assert_eq!(
        cmd,
        Some(command::Command::new(
            "hello",
            vec!["world".into()]
        ))
    );
    let text = parser.next_command().unwrap();
    assert_eq!(
        text,
        Some(command::Command::new_text("This is a text."))
    );
}

#[test]
fn test_parse_example() {
    let input = parser::FileInputSource::new(Path::new("examples/ktxt/example0.ktxt")).expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    // just test no error
    let reached_eof = parser.process_with(|cmd| {
        println!("{:?}", cmd);
        Ok::<bool, Box<parser::ParseError>>(true)
    }).expect("Failed to process file");
    
    // Since we're processing the entire file, we should reach EOF
    assert!(reached_eof, "Should have reached end of file");
    
    let input = parser::FileInputSource::with_encoding(Path::new("examples/ktxt/example0_gbk.ktxt"), Some(encoding_rs::GBK), EncodingErrorStrategy::Strict).expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    // just test no error
    let reached_eof = parser.process_with(|cmd| {
        println!("{:?}", cmd);
        Ok::<bool, Box<parser::ParseError>>(true)
    }).expect("Failed to process file");
    
    // Since we're processing the entire file, we should reach EOF
    assert!(reached_eof, "Should have reached end of file");

    let input = parser::FileInputSource::with_encoding(Path::new("examples/ktxt/example0_utf16.ktxt"), Some(encoding_rs::UTF_16LE), EncodingErrorStrategy::Replace).expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    // just test no error
    let reached_eof = parser.process_with(|cmd| {
        println!("{:?}", cmd);
        Ok::<bool, Box<parser::ParseError>>(true)
    }).expect("Failed to process file");
    
    // Since we're processing the entire file, we should reach EOF
    assert!(reached_eof, "Should have reached end of file");

    let input = parser::FileInputSource::with_encoding(Path::new("examples/ktxt/example0_shift_jis.ktxt"), Some(encoding_rs::SHIFT_JIS), EncodingErrorStrategy::Replace).expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    // just test no error
    let reached_eof = parser.process_with(|cmd| {
        println!("{:?}", cmd);
        Ok::<bool, Box<parser::ParseError>>(true)
    }).expect("Failed to process file");
    // Since we're processing the entire file, we should reach EOF
    assert!(reached_eof, "Should have reached end of file");

    let input = parser::FileInputSource::new(Path::new("examples/ktxt/example1.ktxt")).expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    // just test no error
    let reached_eof = parser.process_with(|cmd| {
        println!("{:?}", cmd);
        Ok::<bool, Box<parser::ParseError>>(true)
    }).expect("Failed to process file");
    // Since we're processing the entire file, we should reach EOF
    assert!(reached_eof, "Should have reached end of file");
    
    let input = parser::FileInputSource::new(Path::new("examples/ktxt/example1.koi0")).expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default().with_command_threshold(0));
    // just test no error
    let reached_eof = parser.process_with(|cmd| {
        println!("{:?}", cmd);
        Ok::<bool, Box<parser::ParseError>>(true)
    }).expect("Failed to process file");
    // Since we're processing the entire file, we should reach EOF
    assert!(reached_eof, "Should have reached end of file");
}

#[test]
fn test_parse_example_with_empty_line() {
    let input = parser::StringInputSource::new(" #");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    let result = parser.next_command();
    println!("{:?}", result);
    assert!(result.is_err());
}

#[test]
fn test_parse_example_with_syntax_error() {
    let text = "#error e(";
    let input = parser::StringInputSource::new(text);
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    let result = parser.next_command();
    // println!("{:#?}", result);
    assert!(result.is_err());
    let err = result.unwrap_err();
    println!("{}", err);

    let text = "#error e() 1";
    let input = parser::StringInputSource::new(text);
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    let result = parser.next_command();
    // println!("{:#?}", result);
    assert!(result.is_err());
    let err = result.unwrap_err();
    println!("{}", err);

    let text = "#error e(1, 2 3)";
    let input = parser::StringInputSource::new(text);
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    let result = parser.next_command();
    // println!("{:#?}", result);
    assert!(result.is_err());
    let err = result.unwrap_err();
    println!("{}", err);
    
    let text = "#error 0xG";
    let input = parser::StringInputSource::new(text);
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    let result = parser.next_command();
    // println!("{:#?}", result);
    assert!(result.is_err());
    let err = result.unwrap_err();
    println!("{}", err);

    let text = "#error e(\n    1, 2 3\n)";
    let input = parser::StringInputSource::new(text);
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    let result = parser.next_command();
    // println!("{:#?}", result);
    assert!(result.is_err());
    let err = result.unwrap_err();
    println!("{}", err);
    
    let text = "  #error 0xG";
    let input = parser::StringInputSource::new(text);
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    let result = parser.next_command();
    // println!("{:#?}", result);
    assert!(result.is_err());
    let err = result.unwrap_err();
    println!("{}", err);
}
