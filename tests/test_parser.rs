use std::path::Path;

use koicore::parser;

#[test]
fn test_parse_hello_world() {
    let input = parser::StringInputSource::new("#hello world\nThis is a text.");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    let cmd = parser.next_command();
    assert_eq!(
        cmd,
        Some(Ok(parser::Command::new(
            "hello".to_string(),
            vec![parser::Parameter::Basic(parser::Value::Literal(
                "world".to_string()
            ))]
        )))
    );
    let text = parser.next_command();
    assert_eq!(
        text,
        Some(Ok(parser::Command::new_text("This is a text.".to_string())))
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
