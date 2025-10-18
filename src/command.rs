//! Command structures for KoiLang parsing
//! 
//! This module defines the data structures used to represent parsed commands
//! and their arguments in a unified format.

use std::{collections::HashMap, fmt};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
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
            Value::String(s) => {
                // Check if the string needs to be quoted (contains spaces or special characters)
                if s.contains(' ') || s.contains('\t') || s.contains('\n') || s.is_empty() {
                    write!(f, "\"{}\"", s)
                } else {
                    write!(f, "{}", s)
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompositeValue {
    Single(Value),
    List(Vec<Value>),
    Dict(Vec<(String, Value)>),
}

impl<T: Into<Value>> From<T> for CompositeValue {
    fn from(v: T) -> Self {
        Self::Single(v.into())
    }
}

impl From<Vec<Value>> for CompositeValue {
    fn from(v: Vec<Value>) -> Self {
        Self::List(v)
    }
}

impl From<Vec<(String, Value)>> for CompositeValue {
    fn from(v: Vec<(String, Value)>) -> Self {
        Self::Dict(v)
    }
}

impl From<HashMap<String, Value>> for CompositeValue {
    fn from(v: HashMap<String, Value>) -> Self {
        Self::Dict(v.into_iter().collect())
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


#[derive(Debug, Clone, PartialEq)]
pub enum Parameter {
    Basic(Value),
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

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub name: String,
    pub params: Vec<Parameter>,
}

impl Command {
    pub fn new(name: String, params: Vec<Parameter>) -> Self {
        Self { name, params }
    }

    pub fn new_text(content: String) -> Self {
        Self::new("@text".to_string(), vec![Parameter::from(content)])
    }

    pub fn new_annotation(content: String) -> Self {
        Self::new("@annotation".to_string(), vec![Parameter::from(content)])
    }

    pub fn new_number(value: i64, args: Vec<Parameter>) -> Self {
        let mut all_args = vec![Parameter::from(value)];
        all_args.extend(args);
        Self::new("@number".to_string(), all_args)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

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
        let cmd = Command::new("hello".to_string(), vec![Parameter::Basic("world".to_string().into())]);
        assert_eq!(format!("{}", cmd), "hello world");
    }

    #[test]
    fn test_command_display_text() {
        let cmd = Command::new_text("hello world".to_string());
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
}
