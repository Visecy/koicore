//! Writer module for KoiLang
//! 
//! This module provides functionality to generate KoiLang text from parsed commands.
//! It supports flexible formatting options and can write to any output that implements
//! the `Write` trait.

use std::collections::HashMap;
use std::io::Write;
use crate::command::Command;

// Re-export configuration types
pub use self::config::{NumberFormat, ParamFormatSelector, FormatterOptions, WriterConfig};

// Internal modules
mod config;
mod formatters;
mod generators;

/// KoiLang writer that can write to any output implementing the `Write` trait
pub struct Writer<'a, T: Write> {
    writer: &'a mut T,
    config: WriterConfig,
    current_indent: usize,
    last_was_newline: bool,
}

impl<'a, T: Write> Writer<'a, T> {
    /// Create a new KoiLang writer
    /// 
    /// # Arguments
    /// * `writer` - Output to write to
    /// * `config` - Configuration for the writer
    pub fn new(writer: &'a mut T, config: WriterConfig) -> Self {
        Self {
            writer,
            config,
            current_indent: 0,
            last_was_newline: false,
        }
    }
    
    /// Write a command using the default formatting options
    pub fn write_command(&mut self, command: &Command) -> std::io::Result<()>
    {
        self.write_command_with_options(command, None, None)
    }
    
    /// Write a command with custom formatting options, including parameter-specific options
    pub fn write_command_with_options(
        &mut self, 
        command: &Command, 
        options: Option<&FormatterOptions>,
        param_options: Option<&HashMap<ParamFormatSelector, FormatterOptions>>
    ) -> std::io::Result<()>
    {
        // Get the appropriate formatting options
        let effective_options = generators::Generators::get_effective_options(
            &command.name, 
            options,
            &self.config
        );
        
        // Write newline before if needed and not already at start of line
        if effective_options.newline_before && !self.last_was_newline {
            writeln!(self.writer)?;
            self.last_was_newline = true;
        }
        
        // Write indentation
        generators::Generators::write_indent(
            self.writer, 
            self.current_indent, 
            &effective_options
        )?;
        
        // Write the command with parameter-specific formatting
        generators::Generators::write_command_with_param_options(
            self.writer,
            command,
            &self.config,
            &effective_options,
            param_options,
            self.current_indent
        )?;
        
        // Write newline after if needed and not already at end of line
        if effective_options.newline_after {
            writeln!(self.writer)?;
            self.last_was_newline = true;
        } else {
            // Update last_was_newline based on the command content
            // For simplicity, we'll assume non-newline ending for now
            self.last_was_newline = false;
        }
        
        Ok(())
    }
    
    /// Increase the indentation level by 1
    pub fn inc_indent(&mut self) {
        self.current_indent += 1;
    }
    
    /// Decrease the indentation level by 1, but not below 0
    pub fn dec_indent(&mut self) {
        if self.current_indent > 0 {
            self.current_indent -= 1;
        }
    }
    
    /// Get the current indentation level
    pub fn get_indent(&self) -> usize {
        self.current_indent
    }

    pub fn newline(&mut self) -> std::io::Result<()> {
        writeln!(self.writer)?;
        self.last_was_newline = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{Command, Parameter};
    
    #[test]
    fn test_write_basic_command() {
        let cmd = Command::new("character", vec![
            Parameter::from("Alice"),
            Parameter::from("Hello, world!")
        ]);
        
        let config = WriterConfig::default();
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);
        
        writer.write_command(&cmd).unwrap();
        
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "#character Alice \"Hello, world!\"");
    }
    
    #[test]
    fn test_write_text_command() {
        let cmd = Command::new_text("Hello, world!");
        
        let config = WriterConfig::default();
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);
        
        writer.write_command(&cmd).unwrap();
        
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "Hello, world!");
    }
    
    #[test]
    fn test_write_annotation_command() {
        let cmd = Command::new_annotation("This is an annotation");
        
        let config = WriterConfig::default();
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);
        
        writer.write_command(&cmd).unwrap();
        
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "## This is an annotation");
    }
    
    #[test]
    fn test_write_number_command() {
        let cmd = Command::new_number(123, vec![Parameter::from("extra")]);
        
        let config = WriterConfig::default();
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);
        
        writer.write_command(&cmd).unwrap();
        
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "123 extra");
    }
    
    #[test]
    fn test_write_with_custom_options() {
        let cmd = Command::new("character", vec![Parameter::from("Alice")]);
        
        let config = WriterConfig::default();
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);
        
        let custom_options = FormatterOptions {
            newline_before: true,
            newline_after: true,
            ..Default::default()
        };
        
        writer.write_command_with_options(&cmd, Some(&custom_options), None).unwrap();
        
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "\n#character Alice\n");
    }
    
    #[test]
    fn test_write_with_force_quotes() {
        let cmd = Command::new("character", vec![
            Parameter::from("Alice"),
            Parameter::from("Bob")
        ]);
        
        let config = WriterConfig {
            global_options: FormatterOptions {
                force_quotes_for_vars: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);
        
        writer.write_command(&cmd).unwrap();
        
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "#character \"Alice\" \"Bob\"");
    }
    
    #[test]
    fn test_write_with_invalid_var_names() {
        let cmd = Command::new("character", vec![
            Parameter::from("123invalid"),
            Parameter::from("with spaces")
        ]);
        
        let config = WriterConfig::default();
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);
        
        writer.write_command(&cmd).unwrap();
        
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "#character \"123invalid\" \"with spaces\"");
    }
    
    #[test]
    fn test_write_with_number_formats() {
        let cmd = Command::new("test", vec![
            Parameter::from(42),
            Parameter::from(255),
            Parameter::from(7)
        ]);
        
        // Test parameter-specific number formats
        let mut param_options = HashMap::new();
        
        // Set first parameter to hex
        param_options.insert(ParamFormatSelector::Position(0), FormatterOptions {
            number_format: NumberFormat::Hex,
            ..Default::default()
        });
        
        // Set second parameter to octal
        param_options.insert(ParamFormatSelector::Position(1), FormatterOptions {
            number_format: NumberFormat::Octal,
            ..Default::default()
        });
        
        // Set third parameter to binary
        param_options.insert(ParamFormatSelector::Position(2), FormatterOptions {
            number_format: NumberFormat::Binary,
            ..Default::default()
        });
        
        let config = WriterConfig::default();
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);
        
        writer.write_command_with_options(&cmd, None, Some(&param_options)).unwrap();
        
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "#test 0x2a 0o377 0b111");
    }
    
    #[test]
    fn test_write_with_param_newlines() {
        let cmd = Command::new("test", vec![
            Parameter::from("param1"),
            Parameter::from("param2"),
            Parameter::from("param3")
        ]);
        
        // Test parameter-specific newlines
        let mut param_options = HashMap::new();
        
        // Add newline after first parameter
        param_options.insert(ParamFormatSelector::Position(0), FormatterOptions {
            newline_after_param: true,
            ..Default::default()
        });
        
        // Add newline before third parameter
        param_options.insert(ParamFormatSelector::Position(2), FormatterOptions {
            newline_before_param: true,
            ..Default::default()
        });
        
        let config = WriterConfig::default();
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);
        
        writer.write_command_with_options(&cmd, None, Some(&param_options)).unwrap();
        
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "#test param1\n    param2\n    param3");
    }
    
    #[test]
    fn test_write_without_repeat_newlines() {
        let cmd = Command::new("test", vec![
            Parameter::from("param1"),
            Parameter::from("param2")
        ]);
        
        // Test that consecutive newlines are not repeated
        let mut param_options = HashMap::new();
        
        // Add newline after first parameter
        param_options.insert(ParamFormatSelector::Position(0), FormatterOptions {
            newline_after_param: true,
            ..Default::default()
        });
        
        // Add newline before second parameter (should not create double newline)
        param_options.insert(ParamFormatSelector::Position(1), FormatterOptions {
            newline_before_param: true,
            ..Default::default()
        });
        
        let config = WriterConfig::default();
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);
        
        writer.write_command_with_options(&cmd, None, Some(&param_options)).unwrap();
        
        let result = String::from_utf8(buffer).unwrap();
        // Should only have one newline between parameters, not two
        assert_eq!(result, "#test param1\n    param2");
    }
    
    #[test]
    fn test_write_with_composite_param_formatting() {
        let cmd = Command::new("test", vec![
            Parameter::from("regular"),
            Parameter::from("(composite, 42)"),
            Parameter::from("another")
        ]);
        
        // Test formatting composite parameters by name
        let mut param_options = HashMap::new();
        
        // Format composite parameter to hex
        param_options.insert(ParamFormatSelector::Name("composite".to_string()), FormatterOptions {
            number_format: NumberFormat::Hex,
            ..Default::default()
        });
        
        let config = WriterConfig::default();
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);
        
        writer.write_command_with_options(&cmd, None, Some(&param_options)).unwrap();
        
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "#test regular \"(composite, 42)\" another");
    }
}