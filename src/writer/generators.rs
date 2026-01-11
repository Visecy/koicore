//! Command and parameter generation for KoiLang writer
//!
//! This module contains the core logic for generating KoiLang text from
//! commands, including handling parameter-specific formatting options and
//! intelligent newline management.

use super::config::{FloatFormat, FormatterOptions, ParamFormatSelector, WriterConfig};
use super::formatters::Formatters;
use crate::command::{Command, Parameter, Value};
use crate::writer::NumberFormat;
use std::collections::HashMap;
use std::io::Write;

/// Command generation utilities
pub struct Generators;

impl Generators {
    /// Write a command with parameter-specific formatting options.
    ///
    /// This function handles the core logic of writing a command to the output, including:
    /// - Handling special command types (`@text`, `@annotation`, `@number`)
    /// - Applying global and command-specific configuration
    /// - Formatting parameters according to their specific options
    /// - Managing indentation and newlines
    ///
    /// # Arguments
    ///
    /// * `writer` - The output writer implementing `Write` trait
    /// * `command` - The command to write
    /// * `config` - The overall writer configuration
    /// * `options` - The effective formatting options for this command
    /// * `param_options` - Optional map of parameter-specific formatting options
    /// * `current_indent` - The current indentation level (number of steps)
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an `std::io::Error` if writing fails.
    pub fn write_command_with_param_options<T: Write>(
        writer: &mut T,
        command: &Command,
        config: &WriterConfig,
        options: &FormatterOptions,
        param_options: Option<&HashMap<ParamFormatSelector, &FormatterOptions>>,
        current_indent: usize,
    ) -> std::io::Result<()> {
        match command.name.as_str() {
            "@text" => {
                // Text command - just write the text as is
                if let Some(Parameter::Basic(Value::String(text))) = command.params.first() {
                    write!(writer, "{}", text)?;
                }
            }
            "@annotation" => {
                // Annotation command - write with extra # characters
                if let Some(Parameter::Basic(Value::String(text))) = command.params.first() {
                    let hashes = "#".repeat(config.command_threshold + 1);
                    if text.trim_start().starts_with(&hashes) {
                        // If text already has enough #, just write it
                        write!(writer, "{}", text)?;
                    } else {
                        // Otherwise, add extra #
                        write!(writer, "{} {}", hashes, text)?;
                    }
                }
            }
            "@number" => {
                // Number command - write as number with parameters
                if let Some(Parameter::Basic(Value::Int(value))) = command.params.first() {
                    let hashes = "#".repeat(config.command_threshold);
                    write!(writer, "{}{}", hashes, value)?;

                    // Add remaining parameters
                    for (i, param) in command.params.iter().skip(1).enumerate() {
                        let param_idx = i + 1;
                        // Get formatting options for this parameter
                        let param_format_opt = Self::get_param_specific_options(
                            param_idx,
                            Self::get_param_name(param),
                            options,
                            param_options,
                        );

                        // Check if we need to add a newline before this parameter
                        // based on previous parameter's newline_after_param
                        if i > 0 {
                            let prev_param = &command.params[param_idx - 1];
                            let prev_opt = Self::get_param_specific_options(
                                param_idx - 1,
                                Self::get_param_name(prev_param),
                                options,
                                param_options,
                            );

                            if prev_opt.newline_after_param || param_format_opt.newline_before_param
                            {
                                writeln!(writer)?;
                                // For non-compact mode, add one more indent level for parameters after newline
                                let indent_level = if options.compact {
                                    current_indent
                                } else {
                                    current_indent + 1
                                };
                                Self::write_indent(writer, indent_level, options)?;
                            } else {
                                // Always add a space between parameters for number commands
                                write!(writer, " ")?;
                            }
                        } else if param_format_opt.newline_before_param {
                            // First additional parameter (i=0) can have newline before
                            writeln!(writer)?;
                            // For non-compact mode, add one more indent level for parameters after newline
                            let indent_level = if options.compact {
                                current_indent
                            } else {
                                current_indent + 1
                            };
                            Self::write_indent(writer, indent_level, options)?;
                        } else {
                            // Always add a space between number and first parameter
                            write!(writer, " ")?;
                        }

                        // Write the parameter
                        write!(
                            writer,
                            "{}",
                            Formatters::format_parameter(param, &param_format_opt)
                        )?;
                    }
                }
            }
            _ => {
                // Regular command - write with # prefix
                let hashes = "#".repeat(config.command_threshold);
                write!(writer, "{}{}", hashes, command.name)?;

                // Add parameters with their specific formatting options
                for (i, param) in command.params.iter().enumerate() {
                    // Get formatting options for this parameter
                    let param_format_opt = Self::get_param_specific_options(
                        i,
                        Self::get_param_name(param),
                        options,
                        param_options,
                    );

                    // Check if we need to add a newline before this parameter
                    if i > 0 {
                        let prev_param = &command.params[i - 1];
                        let prev_opt = Self::get_param_specific_options(
                            i - 1,
                            Self::get_param_name(prev_param),
                            options,
                            param_options,
                        );

                        if prev_opt.newline_after_param || param_format_opt.newline_before_param {
                            writeln!(writer)?;
                            // For non-compact mode, add one more indent level for parameters after newline
                            let indent_level = if options.compact {
                                current_indent
                            } else {
                                current_indent + 1
                            };
                            Self::write_indent(writer, indent_level, options)?;
                        } else {
                            // Always add a space between parameters, even in compact mode
                            // This ensures the parser can distinguish between parameters
                            write!(writer, " ")?;
                        }
                    } else if param_format_opt.newline_before_param {
                        // First parameter can have newline before
                        writeln!(writer)?;
                        // For non-compact mode, add one more indent level for parameters after newline
                        let indent_level = if options.compact {
                            current_indent
                        } else {
                            current_indent + 1
                        };
                        Self::write_indent(writer, indent_level, options)?
                    } else {
                        // Always add a space between command name and first parameter
                        // This ensures the parser can distinguish between command and parameters
                        write!(writer, " ")?;
                    }

                    // Write the parameter
                    write!(
                        writer,
                        "{}",
                        Formatters::format_parameter(param, &param_format_opt)
                    )?;
                }
            }
        }

        Ok(())
    }

    /// Get the parameter name if it's a composite parameter.
    ///
    /// Returns `Some(name)` if the parameter is a `Composite` types, `None` otherwise.
    pub fn get_param_name(param: &Parameter) -> Option<String> {
        match param {
            Parameter::Composite(name, _) => Some(name.clone()),
            _ => None,
        }
    }

    /// Get specific formatting options for a parameter based on its position or name.
    ///
    /// This function resolves the formatting options for a specific parameter by checking:
    /// 1. Name-based options (if the parameter has a name)
    /// 2. Position-based options
    /// 3. Default options (fallback)
    ///
    /// # Arguments
    ///
    /// * `position` - The 0-based index of the parameter
    /// * `name` - The optional name of the parameter (for composite parameters)
    /// * `default_options` - The default options to use as a base
    /// * `param_options` - The map of parameter-specific options
    pub fn get_param_specific_options(
        position: usize,
        name: Option<String>,
        default_options: &FormatterOptions,
        param_options: Option<&HashMap<ParamFormatSelector, &FormatterOptions>>,
    ) -> FormatterOptions {
        if let Some(options_map) = param_options {
            // Try to get options by name first (for composite parameters)
            if let Some(param_name) = name
                && let Some(options) = options_map.get(&ParamFormatSelector::Name(param_name))
            {
                return Self::merge_options(default_options, options);
            }

            // Try to get options by position
            if let Some(options) = options_map.get(&ParamFormatSelector::Position(position)) {
                return Self::merge_options(default_options, options);
            }
        }

        // Fallback to default options
        default_options.clone()
    }

    /// Merge two formatting options, giving precedence to the override options.
    ///
    /// If `override_opt.should_override` is true, the `base` options are completely ignored.
    /// Otherwise, non-default values in `override_opt` will replace corresponding values in `base`.
    ///
    /// # Arguments
    ///
    /// * `base` - The base formatting options
    /// * `override_opt` - The options to merge on top of the base
    pub fn merge_options(
        base: &FormatterOptions,
        override_opt: &FormatterOptions,
    ) -> FormatterOptions {
        if override_opt.should_override {
            return override_opt.clone();
        }

        let mut merged = base.clone();

        // Merge only non-default values from override_opt
        if override_opt.indent != 0 {
            merged.indent = override_opt.indent;
        }
        if override_opt.use_tabs {
            merged.use_tabs = override_opt.use_tabs;
        }
        if override_opt.newline_before {
            merged.newline_before = override_opt.newline_before;
        }
        if override_opt.newline_after {
            merged.newline_after = override_opt.newline_after;
        }
        if override_opt.compact {
            merged.compact = override_opt.compact;
        }
        if override_opt.force_quotes_for_vars {
            merged.force_quotes_for_vars = override_opt.force_quotes_for_vars;
        }
        if override_opt.number_format != NumberFormat::Unknown {
            merged.number_format = override_opt.number_format.clone();
        } else if let NumberFormat::Custom(ref fmt) = merged.number_format {
            if fmt.is_empty() {
                merged.number_format = NumberFormat::Decimal;
            }
        }
        if override_opt.float_format != FloatFormat::Default {
            merged.float_format = override_opt.float_format.clone();
        }
        if override_opt.newline_before_param {
            merged.newline_before_param = override_opt.newline_before_param;
        }
        if override_opt.newline_after_param {
            merged.newline_after_param = override_opt.newline_after_param;
        }

        merged
    }

    /// Write indentation based on current level and options.
    ///
    /// Writes the appropriate indentation string (spaces or tabs) to the writer.
    /// If `options.compact` is true, no indentation is written.
    ///
    /// # Arguments
    ///
    /// * `writer` - The output writer
    /// * `current_indent` - The current indentation level (number of steps)
    /// * `options` - The formatting options defining indent style and size
    pub fn write_indent<T: Write>(
        writer: &mut T,
        current_indent: usize,
        options: &FormatterOptions,
    ) -> std::io::Result<()> {
        if options.compact {
            return Ok(());
        }

        let indent_chars = if options.use_tabs {
            "\t".repeat(current_indent)
        } else {
            " ".repeat(current_indent * options.indent)
        };

        write!(writer, "{}", indent_chars)?;
        Ok(())
    }

    /// Get the effective formatting options for a command.
    ///
    /// Resolves the final formatting options by merging:
    /// 1. Global options from config
    /// 2. Command-specific options from config
    /// 3. Ad-hoc options passed to the function
    ///
    /// # Arguments
    ///
    /// * `command_name` - The name of the command
    /// * `options` - Optional ad-hoc options to apply
    /// * `config` - The writer configuration
    pub fn get_effective_options(
        command_name: &str,
        options: Option<&FormatterOptions>,
        config: &WriterConfig,
    ) -> FormatterOptions {
        let global = match config.command_options.get(command_name) {
            Some(opt) => Self::merge_options(&config.global_options, opt),
            None => config.global_options.clone(),
        };
        let mut result = match options {
            Some(opt) => Self::merge_options(&global, opt),
            None => global,
        };
        if result.indent == 0 {
            result.indent = 4;
        }
        if result.number_format == NumberFormat::Unknown {
            result.number_format = NumberFormat::Decimal;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{Command, CompositeValue, Parameter, Value};
    use crate::writer::{NumberFormat, Writer};

    #[test]
    fn test_get_effective_options() {
        // Test with explicit options
        let explicit_options = FormatterOptions {
            compact: true,
            ..Default::default()
        };
        let config = WriterConfig::default();

        let result = Generators::get_effective_options("test", Some(&explicit_options), &config);
        assert_eq!(result.indent, 4);
        assert_eq!(result.compact, true);

        // Test with command-specific options
        let mut command_options = HashMap::new();
        let command_specific = FormatterOptions {
            newline_after: true,
            ..Default::default()
        };
        command_options.insert("custom_command".to_string(), command_specific.clone());

        let config = WriterConfig {
            command_options,
            ..Default::default()
        };

        let result = Generators::get_effective_options("custom_command", None, &config);
        assert_eq!(result.newline_after, true);

        // Test with global options
        let global_options = FormatterOptions {
            indent: 8,
            ..Default::default()
        };

        let config = WriterConfig {
            global_options: global_options.clone(),
            ..Default::default()
        };

        let result = Generators::get_effective_options("unknown_command", None, &config);
        assert_eq!(result.indent, 8);
        assert_eq!(result.number_format, NumberFormat::Decimal);
    }

    #[test]
    fn test_merge_options() {
        let base_options = FormatterOptions {
            indent: 4,
            compact: false,
            newline_after: false,
            ..Default::default()
        };

        let override_options = FormatterOptions {
            compact: true,
            newline_after: true,
            ..Default::default()
        };

        let merged = Generators::merge_options(&base_options, &override_options);

        // Check that non-overridden options are preserved
        assert_eq!(merged.indent, base_options.indent);

        // Check that overridden options are applied
        assert_eq!(merged.compact, override_options.compact);
        assert_eq!(merged.newline_after, override_options.newline_after);

        // Check that number_format is always overridden
        let override_with_hex = FormatterOptions {
            number_format: NumberFormat::Hex,
            ..Default::default()
        };

        let merged_hex = Generators::merge_options(&base_options, &override_with_hex);
        assert_eq!(merged_hex.number_format, NumberFormat::Hex);
    }

    #[test]
    fn test_merge_options_with_override() {
        let base_options = FormatterOptions {
            indent: 4,
            use_tabs: true,
            ..Default::default()
        };

        // Test override with replace=true (should_override=true)
        // This should ignore base options completely
        let override_full = FormatterOptions {
            indent: 2,
            should_override: true,
            ..Default::default()
        };

        let merged = Generators::merge_options(&base_options, &override_full);
        assert_eq!(merged.indent, 2);
        assert_eq!(merged.use_tabs, false); // Should be false from Default (overridden), not true from base
        assert_eq!(merged.should_override, true);

        // Test normal merge (should_override=false)
        let override_partial = FormatterOptions {
            indent: 8,
            should_override: false,
            ..Default::default()
        };

        let merged_partial = Generators::merge_options(&base_options, &override_partial);
        assert_eq!(merged_partial.indent, 8);
        assert_eq!(merged_partial.use_tabs, true); // Should be preserved from base
    }

    #[test]
    fn test_write_indent() {
        // Test with spaces (default)
        let mut options = FormatterOptions::default();
        options.indent = 4;

        let mut buffer = Vec::new();

        Generators::write_indent(&mut buffer, 2, &options).unwrap();
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "        "); // 2 levels * 4 spaces = 8 spaces

        // Test with tabs
        let options = FormatterOptions {
            use_tabs: true,
            ..Default::default()
        };
        let mut buffer = Vec::new();

        Generators::write_indent(&mut buffer, 2, &options).unwrap();
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "\t\t"); // 2 tabs

        // Test with compact mode (no indent)
        let options = FormatterOptions {
            compact: true,
            ..Default::default()
        };
        let mut buffer = Vec::new();

        Generators::write_indent(&mut buffer, 2, &options).unwrap();
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, ""); // Compact mode - no indent

        // Test with custom indent size
        let options = FormatterOptions {
            indent: 2,
            ..Default::default()
        };
        let mut buffer = Vec::new();

        Generators::write_indent(&mut buffer, 3, &options).unwrap();
        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "      "); // 3 levels * 2 spaces = 6 spaces
    }

    #[test]
    fn test_get_param_name() {
        // Test with basic parameter
        let basic_param = Parameter::from(42);
        let name = Generators::get_param_name(&basic_param);
        assert_eq!(name, None);

        // Test with composite parameter
        let composite_param = Parameter::Composite(
            "test_name".to_string(),
            CompositeValue::Single(Value::Int(42)),
        );
        let name = Generators::get_param_name(&composite_param);
        assert_eq!(name, Some("test_name".to_string()));
    }

    #[test]
    fn test_get_param_specific_options() {
        let default_options = FormatterOptions::default();

        // Test with no param options
        let result = Generators::get_param_specific_options(0, None, &default_options, None);
        assert_eq!(result, default_options);

        // Test with position-based options
        let mut param_options = HashMap::new();

        // Set first parameter to hex
        let hex_opt = FormatterOptions {
            number_format: NumberFormat::Hex,
            ..Default::default()
        };
        param_options.insert(ParamFormatSelector::Position(0), &hex_opt);

        // Set second parameter to octal
        let octal_opt = FormatterOptions {
            number_format: NumberFormat::Octal,
            ..Default::default()
        };
        param_options.insert(ParamFormatSelector::Position(1), &octal_opt);

        // Set third parameter to binary
        let binary_opt = FormatterOptions {
            number_format: NumberFormat::Binary,
            ..Default::default()
        };
        param_options.insert(ParamFormatSelector::Position(2), &binary_opt);

        let config = WriterConfig::default();
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);

        // Create a dummy command for the writer call, as the original test didn't have one
        let cmd = Command::new(
            "dummy",
            vec![Parameter::from(1), Parameter::from(2), Parameter::from(3)],
        );

        writer
            .write_command_with_options(&cmd, None, Some(&param_options))
            .unwrap();

        let output_str = String::from_utf8(buffer).unwrap();
        assert!(!output_str.is_empty());

        let res0 = Generators::get_param_specific_options(
            0, // Check for position 0
            None,
            &default_options,
            Some(&param_options),
        );
        assert_eq!(res0, hex_opt);

        let res1 = Generators::get_param_specific_options(
            1, // Check for position 1
            None,
            &default_options,
            Some(&param_options),
        );
        assert_eq!(res1, octal_opt);

        let res2 = Generators::get_param_specific_options(
            2, // Check for position 2
            None,
            &default_options,
            Some(&param_options),
        );
        assert_eq!(res2, binary_opt);

        // Test with name-based options
        let name_options = FormatterOptions {
            number_format: NumberFormat::Decimal, // Changed to Decimal to avoid conflict with existing binary_opt
            ..Default::default()
        };
        param_options.insert(
            ParamFormatSelector::Name("test_name".to_string()),
            &name_options, // Pass reference
        );

        let result = Generators::get_param_specific_options(
            1,
            Some("test_name".to_string()),
            &default_options,
            Some(&param_options),
        );
        assert_eq!(result, name_options);

        // Test that name-based options take precedence over position-based
        let conflicting_pos_options = FormatterOptions {
            number_format: NumberFormat::Octal,
            ..Default::default()
        };
        param_options.insert(ParamFormatSelector::Position(2), &conflicting_pos_options); // Pass reference

        let result = Generators::get_param_specific_options(
            2,
            Some("test_name".to_string()),
            &default_options,
            Some(&param_options),
        );
        assert_eq!(result, name_options); // Name-based should win
    }

    #[test]
    fn test_write_command_with_param_options() {
        // Test regular command
        let command = Command::new(
            "test_command",
            vec![Parameter::from(42), Parameter::from("string")],
        );
        let config = WriterConfig::default();
        let options = FormatterOptions::default();

        let mut buffer = Vec::new();
        Generators::write_command_with_param_options(
            &mut buffer,
            &command,
            &config,
            &options,
            None,
            0,
        )
        .unwrap();

        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "#test_command 42 string");

        // Test text command
        let command = Command::new_text("Hello, world!");
        let mut buffer = Vec::new();
        Generators::write_command_with_param_options(
            &mut buffer,
            &command,
            &config,
            &options,
            None,
            0,
        )
        .unwrap();

        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "Hello, world!");

        // Test annotation command
        let command = Command::new_annotation("This is an annotation");
        let mut buffer = Vec::new();
        Generators::write_command_with_param_options(
            &mut buffer,
            &command,
            &config,
            &options,
            None,
            0,
        )
        .unwrap();

        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "## This is an annotation");

        // Test number command
        let command = Command::new_number(123, vec![Parameter::from("extra")]);
        let mut buffer = Vec::new();
        Generators::write_command_with_param_options(
            &mut buffer,
            &command,
            &config,
            &options,
            None,
            0,
        )
        .unwrap();

        let result = String::from_utf8(buffer).unwrap();
        assert_eq!(result, "#123 extra");
    }

    #[test]
    fn test_write_number_command_complex() {
        // Test number command with multiple params and newlines
        let cmd = Command::new_number(
            123,
            vec![
                Parameter::from("p1"),
                Parameter::from("p2"),
                Parameter::from("p3"),
            ],
        );

        let options = FormatterOptions {
            indent: 4,
            ..Default::default()
        };

        // Define param options:
        // p1 (idx 0 of params list, but printed after number) -> Position 0 (in extra params list? No, params list includes number param?)
        // Wait, command.params has [Int(123), p1, p2, p3].
        // Loop skips 1.
        // i=0 -> param=p1 -> param_idx=1.
        // get_param_specific_options uses param_idx (1).

        let mut param_options = HashMap::new();

        // Add newline after first parameter
        let nl_after = FormatterOptions {
            newline_after_param: true,
            ..Default::default()
        };
        param_options.insert(ParamFormatSelector::Position(0), &nl_after);

        // Add newline before third parameter
        let nl_before = FormatterOptions {
            newline_before_param: true,
            ..Default::default()
        };
        param_options.insert(ParamFormatSelector::Position(2), &nl_before);

        let config = WriterConfig::default();
        let mut buffer = Vec::new();
        let mut writer = Writer::new(&mut buffer, config);

        writer
            .write_command_with_options(&cmd, Some(&options), Some(&param_options))
            .unwrap(); // Pass HashMap and options

        let result = String::from_utf8(buffer).unwrap();
        // #123 p1
        //         p2 p3
        // Indent: base indent 1 (4 spaces) + 1 (4 spaces) = 8 spaces for p2.
        // Wait, current_indent is 1. write_indent uses level * options.indent.
        // logic: current_indent + 1 = 2.
        // write_command_with_param_options does NOT write initial indent. It writes hashes then name.
        // But for newlines inside parameters, it calls write_indent.
        // So: "#123 p1\n        p2 p3"
        // Wait, initial indent is supplied as 1.

        let expected = "#123 p1\n    p2 p3\n";
        assert_eq!(result, expected);
    }
}
