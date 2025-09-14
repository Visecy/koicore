//! Nom-based command parsing for KoiLang
//! 
//! This module provides nom parsers for KoiLang command syntax,
//! including command names, arguments, and various value types.

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_while, take_while1, take_while_m_n},
    character::complete::{char, digit1, multispace1},
    combinator::{map, map_opt, map_res, opt, recognize, value, verify},
    error::{ErrorKind, FromExternalError, ParseError},
    multi::{fold_many0, many0, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair},
    IResult, Parser,
};
use std::str::FromStr;

use super::command::{Command, Parameter, Value, CompositeValue};

/// Parse a Python-style escaped character: \n, \t, \r, \x41, \u0041, etc.
/// Also handles line continuation where \\\n should be ignored.
fn parse_escaped_char<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    preceded(
        char('\\'),
        alt((
            // Line continuation: \\\n should be ignored, but we're parsing the escape
            // so we need to handle the \n case specially
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\x0B', char('v')), // Vertical tab
            value('\x07', char('a')), // Bell
            value('\x08', char('b')), // Backspace
            value('\x0C', char('f')), // Form feed
            value('\\', char('\\')),
            value('"', char('"')),
            value('\'', char('\'')),
            
            // Hex escape: \xhh
            preceded(
                char('x'),
                map_opt(
                    take_while_m_n(2, 2, |c: char| c.is_ascii_hexdigit()),
                    |hex: &str| {
                        u32::from_str_radix(hex, 16)
                            .ok()
                            .and_then(char::from_u32)
                    }
                )
            ),
            
            // Unicode escape: \uhhhh
            preceded(
                char('u'),
                map_opt(
                    take_while_m_n(4, 4, |c: char| c.is_ascii_hexdigit()),
                    |hex: &str| {
                        u32::from_str_radix(hex, 16)
                            .ok()
                            .and_then(char::from_u32)
                    }
                )
            ),
            
            // Unicode escape: \Uhhhhhhhh
            preceded(
                char('U'),
                map_opt(
                    take_while_m_n(8, 8, |c: char| c.is_ascii_hexdigit()),
                    |hex: &str| {
                        u32::from_str_radix(hex, 16)
                            .ok()
                            .and_then(char::from_u32)
                    }
                )
            ),
            
            // Octal escape: \ooo (1-3 digits)
            map_opt(
                take_while_m_n(1, 3, |c: char| c.is_digit(8)),
                |octal: &str| {
                    u32::from_str_radix(octal, 8)
                        .ok()
                        .and_then(char::from_u32)
                }
            ),
        )),
    )
    .parse(input)
}

/// Parse a non-empty block of text that doesn't include \ or "
fn parse_string_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    let not_quote_slash = is_not("\"\\");
    verify(not_quote_slash, |s: &str| !s.is_empty()).parse(input)
}

/// A string fragment contains a fragment of a string being parsed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    /// Line continuation escape (backslash followed by newline) - should be ignored
    LineContinuation,
}

/// Parse line continuation: backslash followed by newline
fn parse_line_continuation<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, (), E> {
    map(
        (char('\\'), char('\n')),
        |_| ()
    ).parse(input)
}

/// Combine parse_string_literal, parse_line_continuation, and parse_escaped_char into a StringFragment
fn parse_string_fragment<'a, E>(input: &'a str) -> IResult<&'a str, StringFragment<'a>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    alt((
        map(parse_string_literal, StringFragment::Literal),
        map(parse_line_continuation, |_| StringFragment::LineContinuation),
        map(parse_escaped_char, StringFragment::EscapedChar),
    ))
    .parse(input)
}

/// Parse a quoted string
fn parse_string(input: &str) -> IResult<&str, Value> {
    let build_string = fold_many0(
        parse_string_fragment,
        String::new,
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::LineContinuation => {
                    // Line continuation should be ignored - do nothing
                }
            }
            string
        },
    );

    delimited(char('"'), map(build_string, Value::String), char('"')).parse(input)
}

/// Parse a decimal integer
fn parse_decimal_int(input: &str) -> IResult<&str, i64> {
    map_res(
        recognize(pair(
            opt(char('-')),
            digit1
        )),
        |s: &str| i64::from_str(s)
    ).parse(input)
}

/// Parse a hexadecimal integer (0x...)
fn parse_hex_int(input: &str) -> IResult<&str, i64> {
    preceded(
        tag("0x"),
        map_res(
            take_while1(|c: char| c.is_ascii_hexdigit()),
            |s: &str| i64::from_str_radix(s, 16)
        )
    ).parse(input)
}

/// Parse a binary integer (0b...)
fn parse_bin_int(input: &str) -> IResult<&str, i64> {
    preceded(
        tag("0b"),
        map_res(
            take_while1(|c: char| c == '0' || c == '1'),
            |s: &str| i64::from_str_radix(s, 2)
        )
    ).parse(input)
}

/// Parse any integer type (decimal, hex, binary)
fn parse_integer(input: &str) -> IResult<&str, Value> {
    alt((
        map(parse_hex_int, Value::Int),
        map(parse_bin_int, Value::Int),
        map(parse_decimal_int, Value::Int),
    )).parse(input)
}

/// Parse a float number
fn parse_float(input: &str) -> IResult<&str, Value> {
    map_res(
        recognize(
            (opt(char('-')),
                alt((
                    recognize((digit1, char('.'), digit0, opt(float_exp))),
                    recognize((char('.'), digit1, opt(float_exp))),
                    recognize((digit1, float_exp)),
                )))
        ),
        |s: &str| f64::from_str(s).map(Value::Float)
    ).parse(input)
}

/// Helper for float parsing - digits or empty
fn digit0(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_ascii_digit())(input)
}

/// Helper for float parsing - exponent part
fn float_exp(input: &str) -> IResult<&str, &str> {
    recognize((
        alt((char('e'), char('E'))),
        opt(alt((char('+'), char('-')))),
        digit1,
    )).parse(input)
}

/// Parse a literal (valid identifier)
fn parse_literal(input: &str) -> IResult<&str, Value> {
    map(
        recognize(
            pair(
                take_while1(|c: char| c.is_ascii_alphabetic() || c == '_'),
                take_while(|c: char| c.is_ascii_alphanumeric() || c == '_')
            )
        ),
        |s: &str| Value::Literal(s.to_string())
    ).parse(input)
}

/// Parse any basic value type
fn parse_basic_value(input: &str) -> IResult<&str, Value> {
    alt((
        parse_string,  // Try string first since it starts with a quote
        parse_float,
        parse_integer,
        parse_literal,
    )).parse(input)
}

/// Parse a single parameter value (not composite)
fn parse_single_param(input: &str) -> IResult<&str, Parameter> {
    map(parse_basic_value, Parameter::Basic).parse(input)
}

/// Parse a list of values in parentheses: (item1, item2, ...)
fn parse_value_list(input: &str) -> IResult<&str, Vec<Value>> {
    delimited(
        char('('),
        separated_list0(preceded(parse_whitespace_with_continuation, char(',')), 
                       preceded(parse_whitespace_with_continuation, parse_basic_value)),
        preceded(parse_whitespace_with_continuation, char(')'))
    ).parse(input)
}

/// Parse a dictionary in parentheses: (key1: value1, key2: value2, ...)
fn parse_dict(input: &str) -> IResult<&str, Vec<(String, Value)>> {
    delimited(
        char('('),
        separated_list0(
            preceded(parse_whitespace_with_continuation, char(',')),
            preceded(
                parse_whitespace_with_continuation,
                separated_pair(
                    map(parse_literal, |v| match v {
                        Value::Literal(s) => s,
                        _ => unreachable!(),
                    }),
                    preceded(parse_whitespace_with_continuation, char(':')),
                    preceded(parse_whitespace_with_continuation, parse_basic_value)
                )
            )
        ),
        preceded(parse_whitespace_with_continuation, char(')'))
    ).parse(input)
}

/// Parse composite parameters: key(value), key(item1, item2), key(x: 1, y: 2)
fn parse_composite_param(input: &str) -> IResult<&str, Parameter> {
    let (input, key) = parse_literal(input)?;
    let key_str = match key {
        Value::Literal(s) => s,
        _ => return Err(nom::Err::Error(nom::error::Error::new(input, ErrorKind::Tag))),
    };
    
    let (input, composite) = alt((
        map(parse_dict, CompositeValue::Dict),
        map(parse_value_list, |values| {
            if values.len() == 1 {
                CompositeValue::Single(values[0].clone())
            } else {
                CompositeValue::List(values)
            }
        }),
    )).parse(input)?;
    
    Ok((input, Parameter::Composite(key_str, composite)))
}

/// Parse any parameter type (basic or composite)
fn parse_parameter(input: &str) -> IResult<&str, Parameter> {
    alt((
        parse_composite_param,
        parse_single_param,
    )).parse(input)
}

/// Parse a command name (can be literal or number)
fn parse_command_name(input: &str) -> IResult<&str, String> {
    alt((
        map(parse_literal, |v| match v {
            Value::Literal(s) => s,
            _ => unreachable!(),
        }),
        map(parse_decimal_int, |n| n.to_string()),
    )).parse(input)
}

/// Parse whitespace that may include line continuations (backslash + newline)
fn parse_whitespace_with_continuation<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    recognize(
        many0(
            alt((
                multispace1,
                tag("\\\n"),
            ))
        )
    ).parse(input)
}

/// Parse a complete command line: command_name [param1] [param2] ...
pub fn parse_command_line(input: &str) -> IResult<&str, Command> {
    ((
        parse_command_name,
        many0(preceded(parse_whitespace_with_continuation, parse_parameter)),
    )).parse(input).map(|(remaining, (name, params))| {
        (remaining, Command::new(name, params))
    })
    
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_integer() {
        assert_eq!(parse_integer("123"), Ok(("", Value::Int(123))));
        assert_eq!(parse_integer("-456"), Ok(("", Value::Int(-456))));
        assert_eq!(parse_integer("0x1A"), Ok(("", Value::Int(26))));
        assert_eq!(parse_integer("0b101"), Ok(("", Value::Int(5))));
    }
    
    #[test]
    fn test_parse_float() {
        assert_eq!(parse_float("1.23"), Ok(("", Value::Float(1.23))));
        assert_eq!(parse_float("-4.56"), Ok(("", Value::Float(-4.56))));
        assert_eq!(parse_float("1e-2"), Ok(("", Value::Float(0.01))));
    }
    
    #[test]
    fn test_parse_literal() {
        assert_eq!(parse_literal("hello"), Ok(("", Value::Literal("hello".to_string()))));
        assert_eq!(parse_literal("_test_123"), Ok(("", Value::Literal("_test_123".to_string()))));
    }
    
    #[test]
    fn test_parse_command_simple() {
        let result = parse_command_line("command");
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "command");
        assert_eq!(cmd.params().len(), 0);
    }
    
    #[test]
    fn test_parse_command_with_params() {
        let result = parse_command_line("draw Line 2");
        println!("Result: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 2);
        assert_eq!(cmd.params()[0], Parameter::Basic(Value::Literal("Line".to_string())));
        assert_eq!(cmd.params()[1], Parameter::Basic(Value::Int(2)));
    }
    
    #[test]
    fn test_parse_string_parameter() {
        // Test basic value parsing with string
        let basic_result = parse_basic_value("\"Hello World\"");
        println!("Basic value parse result: {:?}", basic_result);
        assert!(basic_result.is_ok());
        
        // Test the full command
        let result = parse_command_line("say \"Hello World\"");
        println!("String parse result: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "say");
        assert_eq!(cmd.params().len(), 1);
        assert_eq!(cmd.params()[0], Parameter::Basic(Value::String("Hello World".to_string())));
        
        // Test escape sequences
        let escape_result = parse_basic_value("\"Hello\\nWorld\"");
        println!("Escape parse result: {:?}", escape_result);
        assert!(escape_result.is_ok());
        if let Ok((_, Value::String(s))) = escape_result {
            assert_eq!(s, "Hello\nWorld");
        }
        
        // Test unicode escape
        let unicode_result = parse_basic_value("\"Emoji: \\U0001F602\"");
        println!("Unicode parse result: {:?}", unicode_result);
        assert!(unicode_result.is_ok());
        if let Ok((_, Value::String(s))) = unicode_result {
            assert_eq!(s, "Emoji: 😂");
        }
        
        // Test hex escape
        let hex_result = parse_basic_value("\"Hex: \\x41\"");
        println!("Hex parse result: {:?}", hex_result);
        assert!(hex_result.is_ok());
        if let Ok((_, Value::String(s))) = hex_result {
            assert_eq!(s, "Hex: A");
        }
        
        // Test octal escape
        let octal_result = parse_basic_value("\"Octal: \\101\"");
        println!("Octal parse result: {:?}", octal_result);
        assert!(octal_result.is_ok());
        if let Ok((_, Value::String(s))) = octal_result {
            assert_eq!(s, "Octal: A");
        }
    }
    
    #[test]
    fn test_parse_composite_single() {
        let result = parse_command_line("draw thickness(2)");
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 1);
        match &cmd.params()[0] {
            Parameter::Composite(name, _) => assert_eq!(name, "thickness"),
            _ => panic!("Expected composite parameter"),
        }
    }
    
    #[test]
    fn test_parse_composite_list() {
        let result = parse_command_line("draw color(255, 255, 255)");
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 1);
    }
    
    #[test]
    fn test_parse_composite_dict() {
        let result = parse_command_line("draw pos(x: 10, y: 20)");
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 1);
    }
    
    #[test]
    fn test_parse_mixed_parameters() {
        let result = parse_command_line("draw Line 2 pos(x: 0, y: 0) thickness(2) color(255, 255, 255)");
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 5);
        assert_eq!(cmd.params()[0], Parameter::Basic(Value::Literal("Line".to_string())));
        assert_eq!(cmd.params()[1], Parameter::Basic(Value::Int(2)));
        assert_eq!(cmd.params()[2], Parameter::Composite("pos".to_string(), CompositeValue::Dict(
            vec![
                ("x".to_string(), Value::Int(0)),
                ("y".to_string(), Value::Int(0)),
            ]
        )));
        assert_eq!(cmd.params()[3], Parameter::Composite("thickness".to_string(), CompositeValue::Single(Value::Int(2))));
        assert_eq!(cmd.params()[4], Parameter::Composite("color".to_string(), CompositeValue::List(
            vec![Value::Int(255), Value::Int(255), Value::Int(255)]
        )));
    }
    
    #[test]
    fn test_parse_number_command() {
        let result = parse_command_line("114 arg1 arg2");
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "114");
        assert_eq!(cmd.params().len(), 2);
    }
    
    #[test]
    fn test_parse_complex_example() {
        // Test the example from Kola README
        let result = parse_command_line("draw Line 2 pos0(x: 0, y: 0) pos1(x: 16, y: 16) thickness(2) color(255, 255, 255)");
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 6);
    }

    #[test]
    fn test_escapes_newline() {
        let result = parse_basic_value("\"Hello\\\nWorld\"");
        println!("Escape parse result: {:?}", result);
        assert!(result.is_ok());
        if let Ok((_, Value::String(s))) = result {
            assert_eq!(s, "HelloWorld");
        }

        let result = parse_command_line("draw Line\\\n2");
        println!("Command parse result: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 2);
        assert_eq!(cmd.params()[0], Parameter::Basic(Value::Literal("Line".to_string())));
        assert_eq!(cmd.params()[1], Parameter::Basic(Value::Int(2)));
    }

    #[test]
    fn test_line_continuation_in_composite_params() {
        // Test line continuation in composite parameters
        let result = parse_command_line("draw pos(x: 10,\\\ny: 20)");
        println!("Composite dict with line continuation: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 1);
        
        // Test line continuation in list parameters
        let result = parse_command_line("draw color(255,\\\n255,\\\n255)");
        println!("Composite list with line continuation: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 1);
        
        // Test line continuation in single parameter
        let result = parse_command_line("draw thickness(\\\n2)");
        println!("Composite single with line continuation: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 1);
    }

    #[test]
    fn test_multiple_line_continuations() {
        // Test multiple line continuations
        let result = parse_command_line("draw \\\nLine \\\n2 \\\npos(x: 0,\\\ny: 0)");
        println!("Multiple line continuations: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 3);
    }
}
