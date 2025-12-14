use std::path::Path;

use koicore::{
    command,
    parser::{self, input::EncodingErrorStrategy},
};

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
        Some(command::Command::new("hello", vec!["world".into()]))
    );
    let text = parser.next_command().unwrap();
    assert_eq!(text, Some(command::Command::new_text("This is a text.")));
}

#[test]
fn test_parse_example() {
    let input = parser::FileInputSource::new(Path::new("examples/ktxt/example0.ktxt"))
        .expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    // just test no error
    let reached_eof = parser
        .process_with(|cmd| {
            println!("{:?}", cmd);
            Ok::<bool, Box<parser::ParseError>>(true)
        })
        .expect("Failed to process file");

    // Since we're processing the entire file, we should reach EOF
    assert!(reached_eof, "Should have reached end of file");

    let input = parser::FileInputSource::with_encoding(
        Path::new("examples/ktxt/example0_gbk.ktxt"),
        Some(encoding_rs::GBK),
        EncodingErrorStrategy::Strict,
    )
    .expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    // just test no error
    let reached_eof = parser
        .process_with(|cmd| {
            println!("{:?}", cmd);
            Ok::<bool, Box<parser::ParseError>>(true)
        })
        .expect("Failed to process file");

    // Since we're processing the entire file, we should reach EOF
    assert!(reached_eof, "Should have reached end of file");

    let input = parser::FileInputSource::with_encoding(
        Path::new("examples/ktxt/example0_utf16.ktxt"),
        Some(encoding_rs::UTF_16LE),
        EncodingErrorStrategy::Replace,
    )
    .expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    // just test no error
    let reached_eof = parser
        .process_with(|cmd| {
            println!("{:?}", cmd);
            Ok::<bool, Box<parser::ParseError>>(true)
        })
        .expect("Failed to process file");

    // Since we're processing the entire file, we should reach EOF
    assert!(reached_eof, "Should have reached end of file");

    let input = parser::FileInputSource::with_encoding(
        Path::new("examples/ktxt/example0_shift_jis.ktxt"),
        Some(encoding_rs::SHIFT_JIS),
        EncodingErrorStrategy::Replace,
    )
    .expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    // just test no error
    let reached_eof = parser
        .process_with(|cmd| {
            println!("{:?}", cmd);
            Ok::<bool, Box<parser::ParseError>>(true)
        })
        .expect("Failed to process file");
    // Since we're processing the entire file, we should reach EOF
    assert!(reached_eof, "Should have reached end of file");

    let input = parser::FileInputSource::new(Path::new("examples/ktxt/example1.ktxt"))
        .expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    // just test no error
    let reached_eof = parser
        .process_with(|cmd| {
            println!("{:?}", cmd);
            Ok::<bool, Box<parser::ParseError>>(true)
        })
        .expect("Failed to process file");
    // Since we're processing the entire file, we should reach EOF
    assert!(reached_eof, "Should have reached end of file");

    let input = parser::FileInputSource::new(Path::new("examples/ktxt/example1.koi0"))
        .expect("Failed to open file");
    let mut parser = parser::Parser::new(
        input,
        parser::ParserConfig::default().with_command_threshold(0),
    );
    // just test no error
    let reached_eof = parser
        .process_with(|cmd| {
            println!("{:?}", cmd);
            Ok::<bool, Box<parser::ParseError>>(true)
        })
        .expect("Failed to process file");
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
#[test]
fn test_parser_edge_cases_unclosed() {
    let inputs = vec![
        "#cmd param(1, 2",
        "#cmd param(x: 1",
        "#cmd \"unclosed string",
        "#cmd param(",
    ];
    for input_str in inputs {
        let input = parser::StringInputSource::new(input_str);
        let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
        assert!(
            parser.next_command().is_err(),
            "Should fail for: {}",
            input_str
        );
    }
}

#[test]
fn test_parser_edge_cases_trailing_comma() {
    // Current implementation uses separated_list1 which generally strictly requires separator between items
    // and DOES NOT support trailing separator unless explicitly handled.
    // Based on command_parser.rs, it doesn't look like it handles trailing commas.
    // So we test that it FAILS.
    let input_str = "#cmd param(1, 2,)";
    let input = parser::StringInputSource::new(input_str);
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    assert!(
        parser.next_command().is_err(),
        "Should fail for trailing comma"
    );
}

#[test]
fn test_parser_duplicate_keys() {
    let input_str = "#cmd p(k:1, k:2)";
    let input = parser::StringInputSource::new(input_str);
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    let cmd = parser
        .next_command()
        .expect("Should parse duplicate keys")
        .unwrap();

    // access params
    if let command::Parameter::Composite(_, val) = &cmd.params[0] {
        if let command::CompositeValue::Dict(entries) = val {
            assert_eq!(entries.len(), 2);
            assert_eq!(entries[0].0, "k");
            assert_eq!(entries[1].0, "k");
        } else {
            panic!("Expected Dict");
        }
    } else {
        panic!("Expected Composite Parameter");
    }
}

#[test]
fn test_parser_empty_composite() {
    // Test empty list/dict if allowed?
    // separated_list1 implies at least one element. So empty list `param()` should FAIL?
    // command_parser.rs: preceded(..., cut(alt((... map(parse_dict...), map(parse_value_list...) ...))))
    // parse_value_list uses separated_list1.
    // parse_dict uses separated_list1.
    // So `param()` should likely FAIL.

    let input_str = "#cmd param()";
    let input = parser::StringInputSource::new(input_str);
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    assert!(
        parser.next_command().is_err(),
        "Should fail for empty composite"
    );
}

#[test]
fn test_parser_escape_sequences() {
    // Test various escape sequences supported by the parser
    // \n, \t, \\, \", \xhh, \uhhhh
    let input_str = r#"#cmd "line1\nline2" "\t" "\\" "\"" "\x41" "\u0042""#;
    let input = parser::StringInputSource::new(input_str);
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());

    let cmd = parser
        .next_command()
        .expect("Parsed escape sequences")
        .unwrap();
    assert_eq!(cmd.params.len(), 6);

    if let command::Parameter::Basic(command::Value::String(s)) = &cmd.params[0] {
        assert_eq!(s, "line1\nline2");
    } else {
        panic!("p0");
    }

    if let command::Parameter::Basic(command::Value::String(s)) = &cmd.params[1] {
        assert_eq!(s, "\t");
    } else {
        panic!("p1");
    }

    if let command::Parameter::Basic(command::Value::String(s)) = &cmd.params[2] {
        assert_eq!(s, "\\");
    } else {
        panic!("p2");
    }

    if let command::Parameter::Basic(command::Value::String(s)) = &cmd.params[3] {
        assert_eq!(s, "\"");
    } else {
        panic!("p3");
    }

    if let command::Parameter::Basic(command::Value::String(s)) = &cmd.params[4] {
        assert_eq!(s, "A"); // \x41
    } else {
        panic!("p4");
    }

    if let command::Parameter::Basic(command::Value::String(s)) = &cmd.params[5] {
        assert_eq!(s, "B"); // \u0042
    } else {
        panic!("p5");
    }
}
