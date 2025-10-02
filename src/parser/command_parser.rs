//! Nom-based command parsing for KoiLang
//!
//! This module provides nom parsers for KoiLang command syntax,
//! including command names, arguments, and various value types.

use nom::{
    branch::alt,
    bytes::complete::{ is_not, tag, take_while, take_while1, take_while_m_n },
    character::complete::{ char, digit1, multispace1 },
    combinator::{ cut, map, map_opt, map_res, opt, recognize, value, verify },
    error::{ context, ContextError, FromExternalError, ParseError },
    multi::{ fold_many0, many0, many1, separated_list1 },
    sequence::{ delimited, pair, preceded, separated_pair },
    IResult,
    Parser,
};
use std::str::FromStr;

use super::command::{ Command, CompositeValue, Parameter, Value };

/// Parse a Python-style escaped character: \n, \t, \r, \x41, \u0041, etc.
/// Also handles line continuation where \\\n should be ignored.
fn parse_escaped_char<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, char, E> {
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
                    |hex: &str| u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
                )
            ),
            // Unicode escape: \uhhhh
            preceded(
                char('u'),
                map_opt(
                    take_while_m_n(4, 4, |c: char| c.is_ascii_hexdigit()),
                    |hex: &str| u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
                )
            ),
            // Unicode escape: \Uhhhhhhhh
            preceded(
                char('U'),
                map_opt(
                    take_while_m_n(8, 8, |c: char| c.is_ascii_hexdigit()),
                    |hex: &str| u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
                )
            ),
            // Octal escape: \ooo (1-3 digits)
            map_opt(
                take_while_m_n(1, 3, |c: char| c.is_digit(8)),
                |octal: &str| u32::from_str_radix(octal, 8).ok().and_then(char::from_u32)
            ),
        ))
    ).parse(input)
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
    map((char('\\'), char('\n')), |_| ()).parse(input)
}

/// Combine parse_string_literal, parse_line_continuation, and parse_escaped_char into a StringFragment
fn parse_string_fragment<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, StringFragment<'a>, E> {
    alt((
        map(parse_string_literal, StringFragment::Literal),
        map(parse_line_continuation, |_| { StringFragment::LineContinuation }),
        map(parse_escaped_char, StringFragment::EscapedChar),
    )).parse(input)
}

/// Parse a quoted string
fn parse_string<'a, E: ParseError<&'a str> + ContextError<&'a str>>(input: &'a str) -> IResult<&'a str, Value, E> {
    let build_string = fold_many0(parse_string_fragment, String::new, |mut string, fragment| {
        match fragment {
            StringFragment::Literal(s) => string.push_str(s),
            StringFragment::EscapedChar(c) => string.push(c),
            StringFragment::LineContinuation => {
                // Line continuation should be ignored - do nothing
            }
        }
        string
    });

    context("string", delimited(char('"'), map(build_string, Value::String), char('"'))).parse(
        input
    )
}

/// Parse a decimal integer
fn parse_decimal_int<'a, E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>>(input: &'a str) -> IResult<&'a str, i64, E> {
    map_res(recognize(pair(opt(char('-')), digit1)), |s: &str| { i64::from_str(s) }).parse(input)
}

/// Parse a hexadecimal integer (0x...)
fn parse_hex_int<'a, E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>>(input: &'a str) -> IResult<&'a str, i64, E> {
    preceded(
        tag("0x"),
        map_res(
            take_while1(|c: char| c.is_ascii_hexdigit()),
            |s: &str| { i64::from_str_radix(s, 16) }
        )
    ).parse(input)
}

/// Parse a binary integer (0b...)
fn parse_bin_int<'a, E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>>(input: &'a str) -> IResult<&'a str, i64, E> {
    preceded(
        tag("0b"),
        map_res(
            take_while1(|c: char| (c == '0' || c == '1')),
            |s: &str| { i64::from_str_radix(s, 2) }
        )
    ).parse(input)
}

/// Parse any integer type (decimal, hex, binary)
fn parse_integer<'a, E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>>(input: &'a str) -> IResult<&'a str, Value, E> {
    context(
        "integer",
        alt((
            map(parse_hex_int, Value::Int),
            map(parse_bin_int, Value::Int),
            map(parse_decimal_int, Value::Int),
        ))
    ).parse(input)
}

/// Parse a float number
fn parse_float<'a, E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, std::num::ParseFloatError>>(input: &'a str) -> IResult<&'a str, Value, E> {
    context(
        "float",
        map_res(
            recognize((
                opt(char('-')),
                alt((
                    recognize((digit1, char('.'), digit0, opt(float_exp))),
                    recognize((char('.'), digit1, opt(float_exp))),
                    recognize((digit1, float_exp)),
                )),
            )),
            |s: &str| f64::from_str(s).map(Value::Float)
        )
    ).parse(input)
}

/// Helper for float parsing - digits or empty
fn digit0<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    take_while(|c: char| c.is_ascii_digit())(input)
}

/// Helper for float parsing - exponent part
fn float_exp<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    recognize((alt((char('e'), char('E'))), opt(alt((char('+'), char('-')))), digit1)).parse(input)
}

/// Parse a literal (valid identifier)
fn parse_literal_str<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    recognize(
        pair(
            take_while1(|c: char| (c.is_ascii_alphabetic() || c == '_')),
            take_while(|c: char| (c.is_ascii_alphanumeric() || c == '_'))
        )
    ).parse(input)
}

/// Parse a literal (valid identifier)
fn parse_literal<'a, E: ParseError<&'a str> + ContextError<&'a str>>(input: &'a str) -> IResult<&'a str, Value, E> {
    context(
        "literal",
        map(parse_literal_str, |s: &str| Value::Literal(s.to_string()))
    ).parse(input)
}

/// Parse any basic value type
fn parse_basic_value<'a, E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError> + FromExternalError<&'a str, std::num::ParseFloatError>>(input: &'a str) -> IResult<&'a str, Value, E> {
    context(
        "basic_value",
        alt((
            parse_string, // Try string first since it starts with a quote
            parse_float,
            parse_integer,
            parse_literal,
        ))
    ).parse(input)
}

/// Parse a single parameter value (not composite)
fn parse_single_param<'a, E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError> + FromExternalError<&'a str, std::num::ParseFloatError>>(input: &'a str) -> IResult<&'a str, Parameter, E> {
    map(parse_basic_value, Parameter::Basic).parse(input)
}

/// Parse a list of values in parentheses: (item1, item2, ...)
fn parse_value_list<'a, E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError> + FromExternalError<&'a str, std::num::ParseFloatError>>(input: &'a str) -> IResult<&'a str, Vec<Value>, E> {
    context(
        "list",
        separated_list1(
            preceded(parse_whitespace_with_continuation, char(',')),
            preceded(parse_whitespace_with_continuation, cut(parse_basic_value))
        )
    ).parse(input)
}

/// Parse a dictionary in parentheses: (key1: value1, key2: value2, ...)
fn parse_dict<'a, E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError> + FromExternalError<&'a str, std::num::ParseFloatError>>(input: &'a str) -> IResult<&'a str, Vec<(String, Value)>, E> {
    context(
        "dictionary",
        separated_list1(
            preceded(parse_whitespace_with_continuation, char(',')),
            preceded(
                parse_whitespace_with_continuation,
                separated_pair(
                    map(parse_literal_str, |v| { v.to_string() }),
                    preceded(parse_whitespace_with_continuation, char(':')),
                    preceded(parse_whitespace_with_continuation, cut(parse_basic_value))
                )
            )
        )
    ).parse(input)
}

/// Parse composite parameters: key(value), key(item1, item2), key(x: 1, y: 2)
fn parse_composite_param<'a, E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError> + FromExternalError<&'a str, std::num::ParseFloatError>>(input: &'a str) -> IResult<&'a str, Parameter, E> {
    context("composite_parameter", (
        parse_literal_str,
        delimited(
            (char('('), parse_whitespace_with_continuation),
            cut(
                alt((
                    map(parse_dict, |dict| CompositeValue::Dict(dict)),
                    map(parse_value_list, |values| {
                        if values.len() == 1 {
                            CompositeValue::Single(values[0].clone())
                        } else {
                            CompositeValue::List(values)
                        }
                    }),
                ))
            ),
            cut((parse_whitespace_with_continuation, char(')')))
        ),
    ))
        .parse(input)
        .map(|(remaining, (key, composite))| {
            (remaining, Parameter::Composite(key.to_string(), composite))
        })
}

/// Parse any parameter type (basic or composite)
fn parse_parameter<'a, E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError> + FromExternalError<&'a str, std::num::ParseFloatError>>(input: &'a str) -> IResult<&'a str, Parameter, E> {
    context("parameter", alt((parse_composite_param, parse_single_param))).parse(input)
}

/// Parse a command name (can be literal or number)
fn parse_command_name<'a, E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>>(input: &'a str) -> IResult<&'a str, String, E> {
    context(
        "command_name",
        cut(
            alt((
                map(parse_literal_str, |v| v.to_string()),
                map(parse_decimal_int, |n| n.to_string()),
            ))
        )
    ).parse(input)
}

/// Parse whitespace that may include line continuations (backslash + newline)
fn parse_whitespace_with_continuation<'a, E: ParseError<&'a str>>(
    input: &'a str
) -> IResult<&'a str, &'a str, E> {
    recognize(many0(alt((multispace1, tag("\\\n"))))).parse(input)
}

/// Parse whitespace that must include line continuations (backslash + newline)
fn parse_whitespace_with_continuation1<'a, E: ParseError<&'a str>>(
    input: &'a str
) -> IResult<&'a str, &'a str, E> {
    recognize(many1(alt((multispace1, tag("\\\n"))))).parse(input)
}

/// Parse a complete command line: command_name [param1] [param2] ...
pub fn parse_command_line<'a, E: ParseError<&'a str> + ContextError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError> + FromExternalError<&'a str, std::num::ParseFloatError>>(input: &'a str) -> IResult<&'a str, Command, E> {
    (parse_command_name, many0(preceded(parse_whitespace_with_continuation1, cut(parse_parameter))))
        .parse(input)
        .map(|(remaining, (name, params))| (remaining, Command::new(name, params)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer() {
        assert_eq!(parse_integer::<nom::error::Error<&str>>("123"), Ok(("", Value::Int(123))));
        assert_eq!(parse_integer::<nom::error::Error<&str>>("-456"), Ok(("", Value::Int(-456))));
        assert_eq!(parse_integer::<nom::error::Error<&str>>("0x1A"), Ok(("", Value::Int(26))));
        assert_eq!(parse_integer::<nom::error::Error<&str>>("0b101"), Ok(("", Value::Int(5))));
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse_float::<nom::error::Error<&str>>("1.23"), Ok(("", Value::Float(1.23))));
        assert_eq!(parse_float::<nom::error::Error<&str>>("-4.56"), Ok(("", Value::Float(-4.56))));
        assert_eq!(parse_float::<nom::error::Error<&str>>("1e-2"), Ok(("", Value::Float(0.01))));
    }

    #[test]
    fn test_parse_literal() {
        assert_eq!(parse_literal::<nom::error::Error<&str>>("hello"), Ok(("", Value::Literal("hello".to_string()))));
        assert_eq!(parse_literal::<nom::error::Error<&str>>("_test_123"), Ok(("", Value::Literal("_test_123".to_string()))));
    }

    #[test]
    fn test_parse_command_simple() {
        let result = parse_command_line::<nom::error::Error<&str>>("command");
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "command");
        assert_eq!(cmd.params().len(), 0);
    }

    #[test]
    fn test_parse_command_with_params() {
        let result = parse_command_line::<nom::error::Error<&str>>("draw Line 2");
        println!("Result: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 2);
        assert_eq!(cmd.params()[0], Value::Literal("Line".to_string()).into());
        assert_eq!(cmd.params()[1], Value::from(2).into());
    }

    #[test]
    fn test_parse_string_parameter() {
        // Test basic value parsing with string
        let basic_result = parse_basic_value::<nom::error::Error<&str>>("\"Hello World\"");
        println!("Basic value parse result: {:?}", basic_result);
        assert!(basic_result.is_ok());

        // Test the full command
        let result = parse_command_line::<nom::error::Error<&str>>("say \"Hello World\"");
        println!("String parse result: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "say");
        assert_eq!(cmd.params().len(), 1);
        assert_eq!(cmd.params()[0], Parameter::from("Hello World"));

        // Test escape sequences
        let escape_result = parse_basic_value::<nom::error::Error<&str>>("\"Hello\\nWorld\"");
        println!("Escape parse result: {:?}", escape_result);
        assert!(escape_result.is_ok());
        if let Ok((_, Value::String(s))) = escape_result {
            assert_eq!(s, "Hello\nWorld");
        }

        // Test unicode escape
        let unicode_result = parse_basic_value::<nom::error::Error<&str>>("\"Emoji: \\U0001F602\"");
        println!("Unicode parse result: {:?}", unicode_result);
        assert!(unicode_result.is_ok());
        if let Ok((_, Value::String(s))) = unicode_result {
            assert_eq!(s, "Emoji: 😂");
        }

        // Test hex escape
        let hex_result = parse_basic_value::<nom::error::Error<&str>>("\"Hex: \\x41\"");
        println!("Hex parse result: {:?}", hex_result);
        assert!(hex_result.is_ok());
        if let Ok((_, Value::String(s))) = hex_result {
            assert_eq!(s, "Hex: A");
        }

        // Test octal escape
        let octal_result = parse_basic_value::<nom::error::Error<&str>>("\"Octal: \\101\"");
        println!("Octal parse result: {:?}", octal_result);
        assert!(octal_result.is_ok());
        if let Ok((_, Value::String(s))) = octal_result {
            assert_eq!(s, "Octal: A");
        }
    }

    #[test]
    fn test_parse_composite_single() {
        let result = parse_command_line::<nom::error::Error<&str>>("draw thickness(2)");
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
        let result = parse_command_line::<nom::error::Error<&str>>("draw color(255, 255, 255)");
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 1);
    }

    #[test]
    fn test_parse_composite_dict() {
        let result = parse_command_line::<nom::error::Error<&str>>("draw pos(x: 10, y: 20)");
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 1);
    }

    #[test]
    fn test_parse_mixed_parameters() {
        let result = parse_command_line::<nom::error::Error<&str>>(
            "draw Line 2 pos(x: 0, y: 0) thickness(2) color(255, 255, 255)"
        );
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 5);
        assert_eq!(cmd.params()[0], Value::Literal("Line".to_string()).into());
        assert_eq!(cmd.params()[1], Value::from(2).into());
        assert_eq!(
            cmd.params()[2],
            Parameter::Composite(
                "pos".to_string(),
                CompositeValue::Dict(
                    vec![
                        ("x".to_string(), Value::from(0).into()),
                        ("y".to_string(), Value::from(0).into())
                    ]
                )
            )
        );
        assert_eq!(
            cmd.params()[3],
            Parameter::Composite("thickness".to_string(), Value::from(2).into())
        );
        assert_eq!(
            cmd.params()[4],
            Parameter::Composite(
                "color".to_string(),
                CompositeValue::List(
                    vec![Value::from(255).into(), Value::from(255).into(), Value::from(255).into()]
                )
            )
        );
    }

    #[test]
    fn test_parse_number_command() {
        let result = parse_command_line::<nom::error::Error<&str>>("114 arg1 arg2");
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "114");
        assert_eq!(cmd.params().len(), 2);
    }

    #[test]
    fn test_parse_complex_example() {
        // Test the example from Kola README
        let result = parse_command_line::<nom::error::Error<&str>>(
            "draw Line 2 pos0(x: 0, y: 0) pos1(x: 16, y: 16) thickness(2) color(255, 255, 255)"
        );
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 6);
    }

    #[test]
    fn test_escapes_newline() {
        let result = parse_basic_value::<nom::error::Error<&str>>("\"Hello\\\nWorld\"");
        println!("Escape parse result: {:?}", result);
        assert!(result.is_ok());
        if let Ok((_, Value::String(s))) = result {
            assert_eq!(s, "HelloWorld");
        }

        let result = parse_command_line::<nom::error::Error<&str>>("draw Line\\\n2");
        println!("Command parse result: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 2);
        assert_eq!(cmd.params()[0], Value::Literal("Line".to_string()).into());
        assert_eq!(cmd.params()[1], Value::from(2).into());
    }

    #[test]
    fn test_line_continuation_in_composite_params() {
        // Test line continuation in composite parameters
        let result = parse_command_line::<nom::error::Error<&str>>("draw pos(x: 10,\\\ny: 20)");
        println!("Composite dict with line continuation: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 1);

        // Test line continuation in list parameters
        let result = parse_command_line::<nom::error::Error<&str>>("draw color(255,\\\n255,\\\n255)");
        println!("Composite list with line continuation: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 1);

        // Test line continuation in single parameter
        let result = parse_command_line::<nom::error::Error<&str>>("draw thickness(\\\n2)");
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
        let result = parse_command_line::<nom::error::Error<&str>>("draw \\\nLine \\\n2 \\\npos(x: 0,\\\ny: 0)");
        println!("Multiple line continuations: {:?}", result);
        assert!(result.is_ok());
        let (remaining, cmd) = result.unwrap();
        assert_eq!(remaining, "");
        assert_eq!(cmd.name(), "draw");
        assert_eq!(cmd.params().len(), 3);
    }
}
