//! Command structures for KoiLang parsing
//!
//! This module defines the data structures used to represent parsed commands
//! and their arguments in a unified format. Commands are the fundamental building
//! blocks of KoiLang files, representing actions, text content, and annotations.
//!
//! ## Core Types
//!
//! - [`Value`] - Basic value types (integers, floats, strings)
//! - [`CompositeValue`] - Complex value types (lists, dictionaries)
//! - [`Parameter`] - Command parameters that can be basic or composite
//! - [`Command`] - Complete commands with name and parameters
//!
//! ## Examples
//!
//! ```rust
//! use koicore::command::{Command, Parameter, Value, CompositeValue};
//!
//! // Create a simple command
//! let cmd = Command::new("character", vec![
//!     Parameter::from("Alice"),
//!     Parameter::from("Hello, world!")
//! ]);
//!
//! // Create a command with composite parameters
//! let cmd = Command::new("action", vec![
//!     Parameter::from(("type", "walk")),
//!     Parameter::from(("direction", "left")),
//!     Parameter::Composite("speed".to_string(), CompositeValue::Single(Value::Int(5)))
//! ]);
//!
//! // Create text and annotation commands
//! let text_cmd = Command::new_text("Hello, world!");
//! let annotation_cmd = Command::new_annotation("This is an annotation");
//! ```

use std::{collections::HashMap, fmt};

/// Basic value types supported by KoiLang
///
/// Represents the fundamental data types that can appear as command parameters
/// or within composite values.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Integer values (64-bit signed)
    Int(i64),
    /// Floating-point values (64-bit)
    Float(f64),
    /// Boolean values
    Bool(bool),
    /// String values (UTF-8 encoded)
    String(String),
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&'_ str> for Value {
    fn from(s: &'_ str) -> Self {
        Self::String(s.to_string())
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(s) => {
                // Check if the string needs to be quoted
                // It needs quoting if:
                // 1. It is empty
                // 2. It contains characters that are not allowed in unquoted identifiers (alphanumeric + '_')
                // 3. It starts with a digit (which would be parsed as a number)
                let needs_quotes = s.is_empty()
                    || !s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    || s.chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false);

                if needs_quotes {
                    write!(f, "\"")?;
                    for c in s.chars() {
                        match c {
                            '"' => write!(f, "\\\"")?,
                            '\\' => write!(f, "\\\\")?,
                            '\n' => write!(f, "\\n")?,
                            '\r' => write!(f, "\\r")?,
                            '\t' => write!(f, "\\t")?,
                            // We don't strictly need to escape other control chars for valid parsing,
                            // but we could. For now, just basic text escapes.
                            c => write!(f, "{}", c)?,
                        }
                    }
                    write!(f, "\"")
                } else {
                    write!(f, "{}", s)
                }
            }
        }
    }
}

/// Composite value types that can contain multiple basic values
///
/// Represents complex data structures that can appear as command parameters,
/// including lists and dictionaries.
#[derive(Debug, Clone, PartialEq)]
pub enum CompositeValue {
    /// Single basic value
    Single(Value),
    /// List of basic values
    List(Vec<Value>),
    /// Dictionary mapping strings to values
    Dict(Vec<(String, Value)>),
}

impl<T: Into<Value>> From<T> for CompositeValue {
    fn from(v: T) -> Self {
        Self::Single(v.into())
    }
}

impl<T: Into<Value>> From<Vec<T>> for CompositeValue {
    fn from(v: Vec<T>) -> Self {
        Self::List(v.into_iter().map(|item| item.into()).collect())
    }
}

impl<T: Into<Value>> From<HashMap<String, T>> for CompositeValue {
    fn from(v: HashMap<String, T>) -> Self {
        Self::Dict(v.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl<T: Into<Value>> FromIterator<T> for CompositeValue {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::List(iter.into_iter().map(|item| item.into()).collect())
    }
}

impl<T: Into<Value>> FromIterator<(String, T)> for CompositeValue {
    fn from_iter<I: IntoIterator<Item = (String, T)>>(iter: I) -> Self {
        Self::Dict(iter.into_iter().map(|(k, v)| (k, v.into())).collect())
    }
}

impl fmt::Display for CompositeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompositeValue::Single(value) => write!(f, "{}", value),
            CompositeValue::List(values) => {
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                Ok(())
            }
            CompositeValue::Dict(entries) => {
                for (i, (key, value)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                Ok(())
            }
        }
    }
}

/// Command parameter types
///
/// Parameters can be either basic values or composite values with names.
/// This allows for flexible command structures in KoiLang.
#[derive(Debug, Clone, PartialEq)]
pub enum Parameter {
    /// Basic parameter containing only a value
    Basic(Value),
    /// Named composite parameter (e.g., `name(value)` or `name(list)`)
    Composite(String, CompositeValue),
}

impl<T: Into<Value>> From<T> for Parameter {
    fn from(v: T) -> Self {
        Self::Basic(v.into())
    }
}

impl<K: Into<String>, V: Into<CompositeValue>> From<(K, V)> for Parameter {
    fn from(v: (K, V)) -> Self {
        Self::Composite(v.0.into(), v.1.into())
    }
}

impl fmt::Display for Parameter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Parameter::Basic(value) => write!(f, "{}", value),
            Parameter::Composite(name, value) => write!(f, "{}({})", name, value),
        }
    }
}

/// Represents a complete KoiLang command
///
/// Commands are the fundamental units of KoiLang files, consisting of a name
/// and zero or more parameters. They can represent actions, text content, or annotations.
#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    /// The command name (e.g., "character", "background", "@text")
    pub name: String,
    /// List of command parameters
    pub params: Vec<Parameter>,
}

impl Command {
    /// Create a new command with the specified name and parameters
    ///
    /// # Arguments
    /// * `name` - The command name (can be `&str` or `String`)
    /// * `params` - Vector of command parameters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::command::{Command, Parameter, Value};
    ///
    /// // Using &str
    /// let cmd = Command::new("character", vec![
    ///     Parameter::from("Alice"),
    ///     Parameter::from("Hello!")
    /// ]);
    ///
    /// // Using String
    /// let cmd = Command::new("character".to_string(), vec![
    ///     Parameter::from("Alice"),
    ///     Parameter::from("Hello!")
    /// ]);
    /// ```
    pub fn new(name: impl Into<String>, params: Vec<Parameter>) -> Self {
        Self {
            name: name.into(),
            params,
        }
    }

    /// Create a text command representing regular content
    ///
    /// Text commands are created for lines that don't start with the command prefix.
    /// They use the special "@text" command name.
    ///
    /// # Arguments
    /// * `content` - The text content (can be `&str` or `String`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::command::Command;
    ///
    /// // Using &str
    /// let text_cmd = Command::new_text("Hello, world!");
    ///
    /// // Using String
    /// let text_cmd = Command::new_text("Hello, world!".to_string());
    /// ```
    pub fn new_text(content: impl Into<String>) -> Self {
        Self::new("@text", vec![Parameter::from(content.into())])
    }

    /// Create an annotation command
    ///
    /// Annotation commands are created for lines with more `#` characters than
    /// the command threshold. They use the special "@annotation" command name.
    ///
    /// # Arguments
    /// * `content` - The annotation content (can be `&str` or `String`)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::command::Command;
    ///
    /// // Using &str
    /// let annotation_cmd = Command::new_annotation("This is an annotation");
    ///
    /// // Using String
    /// let annotation_cmd = Command::new_annotation("This is an annotation".to_string());
    /// ```
    pub fn new_annotation(content: impl Into<String>) -> Self {
        Self::new("@annotation", vec![Parameter::from(content.into())])
    }

    /// Create a number command with integer value and additional parameters
    ///
    /// This is a convenience method for creating commands that start with a number.
    ///
    /// # Arguments
    /// * `value` - The integer value
    /// * `args` - Additional parameters
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::command::{Command, Parameter};
    ///
    /// let num_cmd = Command::new_number(114, vec![]);
    /// let num_cmd_with_args = Command::new_number(42, vec![Parameter::from("extra")]);
    /// ```
    pub fn new_number(value: i64, args: Vec<Parameter>) -> Self {
        let mut all_args = vec![Parameter::from(value)];
        all_args.extend(args);
        Self::new("@number", all_args)
    }

    /// Get the command name
    ///
    /// Returns a reference to the command name string.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the command parameters
    ///
    /// Returns a slice of all parameters associated with this command.
    pub fn params(&self) -> &[Parameter] {
        &self.params
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        for param in self.params.iter() {
            write!(f, " ")?;
            write!(f, "{}", param)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_display() {
        let cmd = Command::new("hello", vec![Parameter::Basic("world".to_string().into())]);
        assert_eq!(format!("{}", cmd), "hello world");
    }

    #[test]
    fn test_command_display_text() {
        let cmd = Command::new_text("hello world");
        assert_eq!(format!("{}", cmd), "@text \"hello world\"");
    }

    #[test]
    fn test_command_display_annotation() {
        let cmd = Command::new_annotation("hello world".to_string());
        assert_eq!(format!("{}", cmd), "@annotation \"hello world\"");
    }

    #[test]
    fn test_convert_value() {
        let cv = Parameter::from(10);
        assert_eq!(format!("{}", cv), "10");
        let cv = Parameter::from(("a", 10));
        assert_eq!(format!("{}", cv), "a(10)");
    }

    #[test]
    fn test_value_display_escaping() {
        let v = Value::String("quote \" and backslash \\".to_string());
        assert_eq!(format!("{}", v), "\"quote \\\" and backslash \\\\\"");

        let v = Value::String("newline \n and tab \t".to_string());
        assert_eq!(format!("{}", v), "\"newline \\n and tab \\t\"");
    }

    #[test]
    fn test_float_display() {
        let v = Value::Float(1.23);
        assert_eq!(format!("{}", v), "1.23");
    }

    #[test]
    fn test_composite_value_conversions() {
        // Test From<Vec<T>>
        let vec_int = vec![1, 2, 3];
        let cv: CompositeValue = CompositeValue::from(vec_int);
        if let CompositeValue::List(list) = cv {
            assert_eq!(list.len(), 3);
            assert_eq!(list[0], Value::Int(1));
        } else {
            panic!("Expected List");
        }

        // Test FromIterator
        let iter = vec![4, 5, 6].into_iter();
        let cv: CompositeValue = iter.collect();
        if let CompositeValue::List(list) = cv {
            assert_eq!(list.len(), 3);
            assert_eq!(list[0], Value::Int(4));
        } else {
            panic!("Expected List");
        }

        // Test From<HashMap> - HashMap iteration order is random, so check existence
        let mut map = HashMap::new();
        map.insert("k1".to_string(), 1);
        let cv: CompositeValue = CompositeValue::from(map);
        if let CompositeValue::Dict(entries) = cv {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].0, "k1");
            assert_eq!(entries[0].1, Value::Int(1));
        } else {
            panic!("Expected Dict");
        }

        // Test FromIterator for Dict
        let map_iter = vec![("k2".to_string(), 2)].into_iter();
        let cv: CompositeValue = map_iter.collect();
        if let CompositeValue::Dict(entries) = cv {
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].0, "k2");
        } else {
            panic!("Expected Dict");
        }
    }

    #[test]
    fn test_composite_value_display() {
        // Test List display
        let list = CompositeValue::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert_eq!(format!("{}", list), "1, 2, 3");

        // Test Dict display
        let dict = CompositeValue::Dict(vec![
            ("key1".to_string(), Value::Int(1)),
            ("key2".to_string(), Value::String("value".to_string())),
        ]);
        assert_eq!(format!("{}", dict), "key1: 1, key2: value");

        // Test Single display (already covered but for completeness)
        let single = CompositeValue::Single(Value::Int(42));
        assert_eq!(format!("{}", single), "42");
    }
}
