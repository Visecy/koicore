use std::path::Path;

use koicore::parser;

#[test]
fn test_parse_hello_world() {
    let input = parser::StringInputSource::new("#hello world\nThis is a text.");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    let cmd = parser.next_command().unwrap();
    assert_eq!(
        cmd,
        Some(parser::Command::new(
            "hello".to_string(),
            vec![parser::Parameter::Basic(parser::Value::Literal(
                "world".to_string()
            ))]
        ))
    );
    let text = parser.next_command().unwrap();
    assert_eq!(
        text,
        Some(parser::Command::new_text("This is a text.".to_string()))
    );
}

#[test]
fn test_parse_example() {
    let input = parser::FileInputSource::new(Path::new("examples/ktxt/example0.ktxt")).expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    // just test no error
    parser.process_with(|cmd| {
        println!("{:?}", cmd);
        Ok::<(), parser::ParseError>(())
    }).expect("Failed to process file");
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
}
