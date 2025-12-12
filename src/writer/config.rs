//! Configuration types for KoiLang writer
//! 
//! This module defines the configuration types used by the KoiLang writer,
//! including number formats, formatter options, and parameter selectors.

use std::collections::HashMap;

/// Number format options for numeric values
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum NumberFormat {
    /// Decimal format (default)
    #[default]
    Decimal,
    /// Hexadecimal format
    Hex,
    /// Octal format
    Octal,
    /// Binary format
    Binary,
}

/// Selector for parameter-specific formatting options
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ParamFormatSelector {
    /// Select parameter by position (0-based index)
    Position(usize),
    /// Select composite parameter by name
    Name(String),
}

/// Formatting options for KoiLang generation
#[derive(Debug, Clone, PartialEq)]
pub struct FormatterOptions {
    /// Number of spaces to use for indentation
    pub indent: usize,
    /// Whether to use tabs for indentation instead of spaces
    pub use_tabs: bool,
    /// Whether to add a newline before the command
    pub newline_before: bool,
    /// Whether to add a newline after the command
    pub newline_after: bool,
    /// Whether to use compact formatting (minimal whitespace)
    pub compact: bool,
    /// Whether to force quotes for names that match variable naming rules
    pub force_quotes_for_vars: bool,
    /// Format to use for numeric values
    pub number_format: NumberFormat,
    /// Whether to add a newline before this specific parameter
    pub newline_before_param: bool,
    /// Whether to add a newline after this specific parameter
    pub newline_after_param: bool,
}

impl Default for FormatterOptions {
    fn default() -> Self {
        Self {
            indent: 4,
            use_tabs: false,
            newline_before: false,
            newline_after: false,
            compact: false,
            force_quotes_for_vars: false,
            number_format: NumberFormat::Decimal,
            newline_before_param: false,
            newline_after_param: false,
        }
    }
}

/// Configuration for the KoiLang writer
#[derive(Debug, Clone)]
pub struct WriterConfig {
    /// Global formatting options
    pub global_options: FormatterOptions,
    /// Command-specific formatting options
    pub command_options: HashMap<String, FormatterOptions>,
    /// Command threshold (number of # required for commands)
    pub command_threshold: usize,
}

impl Default for WriterConfig {
    fn default() -> Self {
        Self {
            global_options: FormatterOptions::default(),
            command_options: HashMap::new(),
            command_threshold: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_number_format_default() {
        let default = NumberFormat::default();
        assert_eq!(default, NumberFormat::Decimal);
    }
    
    #[test]
    fn test_param_format_selector() {
        let pos_selector = ParamFormatSelector::Position(0);
        let name_selector = ParamFormatSelector::Name("test".to_string());
        
        // Test that different selectors are not equal
        assert_ne!(pos_selector, name_selector);
        
        // Test that same position selectors are equal
        let pos_selector2 = ParamFormatSelector::Position(0);
        assert_eq!(pos_selector, pos_selector2);
        
        // Test that same name selectors are equal
        let name_selector2 = ParamFormatSelector::Name("test".to_string());
        assert_eq!(name_selector, name_selector2);
    }
    
    #[test]
    fn test_formatter_options_default() {
        let default = FormatterOptions::default();
        
        assert_eq!(default.indent, 4);
        assert_eq!(default.use_tabs, false);
        assert_eq!(default.newline_before, false);
        assert_eq!(default.newline_after, false);
        assert_eq!(default.compact, false);
        assert_eq!(default.force_quotes_for_vars, false);
        assert_eq!(default.number_format, NumberFormat::Decimal);
        assert_eq!(default.newline_before_param, false);
        assert_eq!(default.newline_after_param, false);
    }
    
    #[test]
    fn test_writer_config_default() {
        let default = WriterConfig::default();
        
        // Test default global options
        assert_eq!(default.global_options.indent, 4);
        assert_eq!(default.global_options.compact, false);
        
        // Test default command options
        assert!(default.command_options.is_empty());
        
        // Test default command threshold
        assert_eq!(default.command_threshold, 1);
    }
    
    #[test]
    fn test_writer_config_with_custom_command_options() {
        let mut command_options = HashMap::new();
        let custom_options = FormatterOptions {
            compact: true,
            newline_after: true,
            ..Default::default()
        };
        command_options.insert("custom_command".to_string(), custom_options.clone());
        
        let config = WriterConfig {
            command_options,
            ..Default::default()
        };
        
        // Test that custom command options are stored correctly
        assert_eq!(config.command_options.len(), 1);
        assert_eq!(config.command_options.get("custom_command"), Some(&custom_options));
    }
    
    #[test]
    fn test_writer_config_with_custom_threshold() {
        let config = WriterConfig {
            command_threshold: 2,
            ..Default::default()
        };
        
        assert_eq!(config.command_threshold, 2);
    }
}
