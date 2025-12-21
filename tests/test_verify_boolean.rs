use koicore::{
    command::{Parameter, Value},
    parser::{Parser, ParserConfig, StringInputSource},
    writer::{Writer, WriterConfig},
};
use std::sync::{Arc, Mutex};

#[test]
fn test_boolean_parsing_and_writing() {
    let input = "#test_bool p1(true) p2(false)";
    // Input source expects &str
    let source = Arc::new(Mutex::new(StringInputSource::new(input)));
    // ParserConfig expects 3 args: threshold, skip_annotations, convert_number_command
    let config = ParserConfig::new(1, false, false);
    let mut parser = Parser::new(source, config);

    let cmd = parser
        .next_command()
        .expect("Should parse command")
        .expect("Should verify no error");

    assert_eq!(cmd.name, "test_bool");

    // Check p1(true)
    let p1_param = cmd.params.iter().find(|p| match p {
        Parameter::Composite(name, _) => name == "p1",
        _ => false,
    });

    if let Some(Parameter::Composite(_, val)) = p1_param {
        match val {
            koicore::command::CompositeValue::Single(Value::Bool(b)) => assert_eq!(*b, true),
            _ => panic!("p1 should be Single(Bool(true))"),
        }
    } else {
        panic!("p1 parameter not found or not composite");
    }

    // Check p2(false)
    let p2_param = cmd.params.iter().find(|p| match p {
        Parameter::Composite(name, _) => name == "p2",
        _ => false,
    });

    if let Some(Parameter::Composite(_, val)) = p2_param {
        match val {
            koicore::command::CompositeValue::Single(Value::Bool(b)) => assert_eq!(*b, false),
            _ => panic!("p2 should be Single(Bool(false))"),
        }
    } else {
        panic!("p2 parameter not found");
    }

    // Parsing List with boolean l(true, false)
    let input_list = "#test_list l(true, false)";
    let source = Arc::new(Mutex::new(StringInputSource::new(input_list)));
    let mut parser = Parser::new(source, ParserConfig::new(1, false, false));
    let cmd = parser.next_command().unwrap().unwrap();

    // Formatting check
    let writer_config = WriterConfig::default();
    let mut output = Vec::new();
    let mut writer = Writer::new(&mut output, writer_config);
    writer.write_command(&cmd).unwrap();

    let output_str = String::from_utf8(output).unwrap();
    // Normalize newlines
    let output_str = output_str.replace("\r\n", "\n");

    // We expect `l(true, false)`
    assert!(output_str.contains("true"));
    assert!(output_str.contains("false"));
    assert!(output_str.contains("l("));
}
