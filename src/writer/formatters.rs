//! Formatting utilities for KoiLang writer
//!
//! This module contains utilities for formatting different types of values
//! and parameters in KoiLang text generation.

use super::config::{FormatterOptions, NumberFormat};
use crate::command::{CompositeValue, Parameter, Value};

/// Formatting utilities for KoiLang values
pub struct Formatters;

impl Formatters {
    /// Format a number according to the specified format.
    ///
    /// # Arguments
    ///
    /// * `num` - The integer value to format
    /// * `options` - Formatting options determining the base (decimal, hex, etc.)
    pub fn format_number(num: &i64, options: &FormatterOptions) -> String {
        match options.number_format {
            NumberFormat::Decimal | NumberFormat::Unknown => num.to_string(),
            NumberFormat::Hex => format!("0x{:x}", num),
            NumberFormat::Octal => format!("0o{:o}", num),
            NumberFormat::Binary => format!("0b{:b}", num),
        }
    }

    /// Check if a string matches variable naming rules.
    ///
    /// Variable names must start with a letter or underscore, followed by letters, numbers, or underscores.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to check
    pub fn is_valid_variable_name(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let mut chars = s.chars();
        let first_char = chars.next().unwrap();

        // First character must be a letter or underscore
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return false;
        }

        // Remaining characters must be letters, numbers, or underscores
        for c in chars {
            if !c.is_ascii_alphanumeric() && c != '_' {
                return false;
            }
        }

        true
    }

    /// Format a string value with appropriate quoting.
    ///
    /// Adds double quotes if the string is not a valid variable name or if forced by options.
    ///
    /// # Arguments
    ///
    /// * `s` - The string to format
    /// * `options` - Formatting options
    pub fn format_string(s: &str, options: &FormatterOptions) -> String {
        // Check if the string needs quotes
        let needs_quotes = options.force_quotes_for_vars || !Self::is_valid_variable_name(s);

        if needs_quotes {
            let mut result = String::with_capacity(s.len() + 2);
            result.push('"');
            for c in s.chars() {
                match c {
                    '"' => result.push_str("\\\""),
                    '\\' => result.push_str("\\\\"),
                    '\n' => result.push_str("\\n"),
                    '\r' => result.push_str("\\r"),
                    '\t' => result.push_str("\\t"),
                    c => result.push(c),
                }
            }
            result.push('"');
            result
        } else {
            s.to_string()
        }
    }

    /// Format a composite value (List or Dictionary).
    ///
    /// Recursively formats the values inside the composite structure.
    ///
    /// # Arguments
    ///
    /// * `value` - The composite value
    /// * `options` - Formatting options
    pub fn format_composite_value(value: &CompositeValue, options: &FormatterOptions) -> String {
        match value {
            CompositeValue::Single(val) => {
                format!("({})", Self::format_value(val, options))
            }
            CompositeValue::List(values) => {
                let mut result = "(".to_string();
                let mut first = true;

                for val in values {
                    if !first {
                        result.push(',');
                        if !options.compact {
                            result.push(' ');
                        }
                    }
                    result.push_str(&Self::format_value(val, options));
                    first = false;
                }

                result.push(')');
                result
            }
            CompositeValue::Dict(entries) => {
                let mut result = "(".to_string();
                let mut first = true;

                for (key, val) in entries {
                    if !first {
                        result.push(',');
                        if !options.compact {
                            result.push(' ');
                        }
                    }
                    result.push_str(key);
                    result.push(':');
                    if !options.compact {
                        result.push(' ');
                    }
                    result.push_str(&Self::format_value(val, options));
                    first = false;
                }

                result.push(')');
                result
            }
        }
    }

    /// Format a basic value (Int, Float, String).
    ///
    /// # Arguments
    ///
    /// * `value` - The basic value to format
    /// * `options` - Formatting options
    pub fn format_value(value: &Value, options: &FormatterOptions) -> String {
        match value {
            Value::Int(i) => Self::format_number(i, options),
            Value::Float(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::String(s) => Self::format_string(s, options),
        }
    }

    /// Format a parameter (Basic or Composite).
    ///
    /// # Arguments
    ///
    /// * `param` - The parameter to format
    /// * `options` - Formatting options
    pub fn format_parameter(param: &Parameter, options: &FormatterOptions) -> String {
        // Space before is now handled by generators.rs to avoid double spaces

        let param_text = match param {
            Parameter::Basic(value) => Self::format_value(value, options),
            Parameter::Composite(name, composite_value) => {
                format!(
                    "{}{}",
                    name,
                    Self::format_composite_value(composite_value, options)
                )
            }
        };

        param_text.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{CompositeValue, Parameter, Value};

    #[test]
    fn test_format_number() {
        let options = FormatterOptions::default();

        // Test decimal format (default)
        let result = Formatters::format_number(&42, &options);
        assert_eq!(result, "42");

        // Test hex format
        let options = FormatterOptions {
            number_format: NumberFormat::Hex,
            ..Default::default()
        };
        let result = Formatters::format_number(&255, &options);
        assert_eq!(result, "0xff");

        // Test octal format
        let options = FormatterOptions {
            number_format: NumberFormat::Octal,
            ..Default::default()
        };
        let result = Formatters::format_number(&63, &options);
        assert_eq!(result, "0o77");

        // Test binary format
        let options = FormatterOptions {
            number_format: NumberFormat::Binary,
            ..Default::default()
        };
        let result = Formatters::format_number(&7, &options);
        assert_eq!(result, "0b111");

        // Test negative numbers
        let options = FormatterOptions::default();
        let result = Formatters::format_number(&-42, &options);
        assert_eq!(result, "-42");

        let options = FormatterOptions {
            number_format: NumberFormat::Hex,
            ..Default::default()
        };
        let result = Formatters::format_number(&-255, &options);
        // For i64, -255 in hex is 0xffffffffffffff01
        assert_eq!(result, "0xffffffffffffff01");
    }

    #[test]
    fn test_is_valid_variable_name() {
        // Valid names
        assert!(Formatters::is_valid_variable_name("valid_name"));
        assert!(Formatters::is_valid_variable_name("_valid"));
        assert!(Formatters::is_valid_variable_name("valid123"));
        assert!(Formatters::is_valid_variable_name("a"));
        assert!(Formatters::is_valid_variable_name("A"));

        // Invalid names
        assert!(!Formatters::is_valid_variable_name("123invalid"));
        assert!(!Formatters::is_valid_variable_name("invalid-name"));
        assert!(!Formatters::is_valid_variable_name("invalid name"));
        assert!(!Formatters::is_valid_variable_name("invalid.name"));
        assert!(!Formatters::is_valid_variable_name(""));
        assert!(!Formatters::is_valid_variable_name("!invalid"));
        assert!(!Formatters::is_valid_variable_name("invalid!"));
    }

    #[test]
    fn test_format_string() {
        // Test valid variable names (no quotes needed by default)
        let options = FormatterOptions::default();
        let result = Formatters::format_string("valid_name", &options);
        assert_eq!(result, "valid_name");

        // Test invalid variable names (need quotes)
        let result = Formatters::format_string("invalid-name", &options);
        assert_eq!(result, "\"invalid-name\"");

        // Test with spaces (need quotes)
        let result = Formatters::format_string("with_spaces", &options);
        assert_eq!(result, "with_spaces");

        // Test with force_quotes_for_vars
        let options = FormatterOptions {
            force_quotes_for_vars: true,
            ..Default::default()
        };
        let result = Formatters::format_string("valid_name", &options);
        assert_eq!(result, "\"valid_name\"");
    }

    #[test]
    fn test_format_composite_value() {
        let options = FormatterOptions::default();

        // Test Single composite value
        let single_value = CompositeValue::Single(Value::Int(42));
        let result = Formatters::format_composite_value(&single_value, &options);
        assert_eq!(result, "(42)");

        // Test List composite value
        let list_value = CompositeValue::List(vec![
            Value::Int(1),
            Value::String("two".to_string()),
            Value::Int(3),
        ]);
        let result = Formatters::format_composite_value(&list_value, &options);
        assert_eq!(result, "(1, two, 3)");

        // Test List composite value in compact mode
        let options_compact = FormatterOptions {
            compact: true,
            ..Default::default()
        };
        let result = Formatters::format_composite_value(&list_value, &options_compact);
        assert_eq!(result, "(1,two,3)");

        // Test Dict composite value
        let dict_entries = vec![
            ("key1".to_string(), Value::Int(1)),
            ("key2".to_string(), Value::String("value2".to_string())),
        ];
        let dict_value = CompositeValue::Dict(dict_entries);
        let result = Formatters::format_composite_value(&dict_value, &options);
        assert_eq!(result, "(key1: 1, key2: value2)");

        // Test Dict composite value in compact mode
        let result = Formatters::format_composite_value(&dict_value, &options_compact);
        assert_eq!(result, "(key1:1,key2:value2)");
    }

    #[test]
    fn test_format_value() {
        let options = FormatterOptions::default();

        // Test Int value
        let result = Formatters::format_value(&Value::Int(42), &options);
        assert_eq!(result, "42");

        // Test Float value
        let result = Formatters::format_value(&Value::Float(3.14), &options);
        assert_eq!(result, "3.14");

        // Test String value
        let result = Formatters::format_value(&Value::String("test".to_string()), &options);
        assert_eq!(result, "test");

        // Test invalid String value (needs quotes)
        let result =
            Formatters::format_value(&Value::String("test-with-dash".to_string()), &options);
        assert_eq!(result, "\"test-with-dash\"");

        // Test negative Int
        let result = Formatters::format_value(&Value::Int(-42), &options);
        assert_eq!(result, "-42");
    }

    #[test]
    fn test_format_parameter() {
        let options = FormatterOptions::default();

        // Test Basic parameter with Int value
        let basic_param = Parameter::from(42);
        let result = Formatters::format_parameter(&basic_param, &options);
        assert_eq!(result, "42");

        // Test Basic parameter with String value
        let basic_param = Parameter::from("test");
        let result = Formatters::format_parameter(&basic_param, &options);
        assert_eq!(result, "test");

        // Test Composite parameter
        let composite_param = Parameter::Composite(
            "test_name".to_string(),
            CompositeValue::Single(Value::Int(42)),
        );
        let result = Formatters::format_parameter(&composite_param, &options);
        assert_eq!(result, "test_name(42)");

        // Test Composite parameter with List
        let composite_param = Parameter::Composite(
            "list_param".to_string(),
            CompositeValue::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]),
        );
        let result = Formatters::format_parameter(&composite_param, &options);
        assert_eq!(result, "list_param(1, 2, 3)");

        // Test Composite parameter with Dict
        let dict_entries = vec![("key".to_string(), Value::String("value".to_string()))];
        let composite_param =
            Parameter::Composite("dict_param".to_string(), CompositeValue::Dict(dict_entries));
        let result = Formatters::format_parameter(&composite_param, &options);
        assert_eq!(result, "dict_param(key: value)");
    }

    #[test]
    fn test_format_value_with_number_formats() {
        // Test different number formats for Int values
        let hex_options = FormatterOptions {
            number_format: NumberFormat::Hex,
            ..Default::default()
        };
        let result = Formatters::format_value(&Value::Int(255), &hex_options);
        assert_eq!(result, "0xff");

        let oct_options = FormatterOptions {
            number_format: NumberFormat::Octal,
            ..Default::default()
        };
        let result = Formatters::format_value(&Value::Int(63), &oct_options);
        assert_eq!(result, "0o77");

        let bin_options = FormatterOptions {
            number_format: NumberFormat::Binary,
            ..Default::default()
        };
        let result = Formatters::format_value(&Value::Int(7), &bin_options);
        assert_eq!(result, "0b111");
    }
}
