//! Command structures for KoiLang parsing
//! 
//! This module defines the data structures used to represent parsed commands
//! and their arguments in a unified format.

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Literal(String),
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

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Literal(s) => write!(f, "{}", s),
            Value::String(s) => write!(f, "\"{}\"", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CompositeValue {
    Single(Value),
    List(Vec<Value>),
    Dict(Vec<(String, Value)>),
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
    name: String,
    params: Vec<Parameter>,
}

impl Command {
    pub fn new(name: String, params: Vec<Parameter>) -> Self {
        Self { name, params }
    }

    pub fn new_text(content: String) -> Self {
        Self::new("@text".to_string(), vec![Parameter::Basic(Value::String(content))])
    }

    pub fn new_annotation(content: String) -> Self {
        Self::new("@annotation".to_string(), vec![Parameter::Basic(Value::String(content))])
    }

    pub fn new_number(value: i64, args: Vec<Parameter>) -> Self {
        let mut all_args = vec![Parameter::Basic(Value::Int(value))];
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
