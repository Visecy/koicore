//! KoiLang parser module
//!
//! This module provides the core parsing functionality for KoiLang files.
//!
//! ## Features
//!
//! - **Streaming Processing**: Parse files of any size with constant memory usage
//! - **Multiple Input Sources**: Parse from strings, files, or custom sources
//! - **Encoding Support**: Handle various text encodings through `DecodeBufReader`
//! - **Comprehensive Error Handling**: Detailed error messages with source locations
//! - **Configurable Parsing**: Customizable command thresholds and parsing rules
//!
//! ## Usage
//!
//! ```rust
//! use koicore::parser::{Parser, ParserConfig, StringInputSource};
//!
//! let input = StringInputSource::new("#name \"Test\"\nHello World");
//! let config = ParserConfig::default();
//! let mut parser = Parser::new(input, config);
//!
//! while let Some(command) = parser.next_command()? {
//!     println!("Command: {}", command.name());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod command_parser;
pub mod decode_buf_reader;
pub mod error;
pub mod input;
pub mod traceback;

use super::command::Command;
pub use error::{ErrorInfo, ParseError, ParseResult};
pub use input::{BufReadWrapper, FileInputSource, StringInputSource, TextInputSource};
use nom::Offset;
pub use traceback::TracebackEntry;

use input::Input;
use traceback::NomErrorNode;

/// Configuration for the line processor
///
/// Controls how the parser interprets different types of lines in the input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserConfig {
    /// The command threshold (number of # required for commands)
    ///
    /// Lines with fewer # characters than this threshold are treated as text.
    /// Lines with exactly this many # characters are treated as commands.
    /// Lines with more # characters are treated as annotations.
    pub command_threshold: usize,
    /// Whether to skip annotation lines (lines starting with #)
    ///
    /// If set to true, annotation lines will be skipped and not processed as commands.
    /// If set to false, annotation lines will be included in the output as special commands.
    pub skip_annotations: bool,
    /// Whether to convert number commands to special commands
    ///
    /// If set to true, commands with names that are valid integers will be converted
    /// to special number commands. If set to false, they will be treated as regular commands.
    pub convert_number_command: bool,
    /// Whether to preserve indentation in text and annotation lines
    ///
    /// If set to true, leading whitespace (indentation) will be preserved in text and
    /// annotation content. If set to false, leading whitespace will be trimmed.
    pub preserve_indent: bool,
    /// Whether to preserve empty lines as text commands
    ///
    /// If set to true, empty lines will be preserved and returned as empty text commands.
    /// If set to false, empty lines will be skipped.
    pub preserve_empty_lines: bool,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            command_threshold: 1,
            skip_annotations: false,
            convert_number_command: true,
            preserve_indent: false,
            preserve_empty_lines: false,
        }
    }
}

impl ParserConfig {
    /// Create a new parser configuration with the specified command threshold
    ///
    /// # Arguments
    /// * `threshold` - Number of # characters required to identify a command line
    /// * `skip_annotations` - Whether to skip annotation lines
    /// * `convert_number_command` - Whether to convert number commands
    /// * `preserve_indent` - Whether to preserve leading whitespace in text/annotations
    /// * `preserve_empty_lines` - Whether to preserve empty lines as text commands
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::parser::ParserConfig;
    ///
    /// // Default configuration (threshold = 1)
    /// let config = ParserConfig::default();
    ///
    /// // Custom configuration
    /// let config = ParserConfig::new(2, true, true, false, false);
    /// ```
    pub fn new(
        threshold: usize,
        skip_annotations: bool,
        convert_number_command: bool,
        preserve_indent: bool,
        preserve_empty_lines: bool,
    ) -> Self {
        Self {
            command_threshold: threshold,
            skip_annotations,
            convert_number_command,
            preserve_indent,
            preserve_empty_lines,
        }
    }

    /// Set the command threshold for this configuration
    ///
    /// # Arguments
    /// * `threshold` - New number of # characters required to identify a command line
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::parser::ParserConfig;
    ///
    /// let config = ParserConfig::default().with_command_threshold(2);
    /// ```
    pub fn with_command_threshold(mut self, threshold: usize) -> Self {
        self.command_threshold = threshold;
        self
    }

    /// Set whether to skip annotation lines for this configuration
    ///
    /// # Arguments
    /// * `skip` - Whether to skip annotation lines (true) or include them (false)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::parser::ParserConfig;
    ///
    /// let config = ParserConfig::default().with_skip_annotations(true);
    /// ```
    pub fn with_skip_annotations(mut self, skip: bool) -> Self {
        self.skip_annotations = skip;
        self
    }

    /// Set whether to convert number command name into @number
    ///
    /// # Arguments
    /// * `convert` - Whether to convert number command name into @number
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::parser::ParserConfig;
    ///
    /// let config = ParserConfig::default().with_convert_number_command(true);
    /// ```
    pub fn with_convert_number_command(mut self, convert: bool) -> Self {
        self.convert_number_command = convert;
        self
    }

    /// Set whether to preserve indentation in text and annotation lines
    ///
    /// # Arguments
    /// * `preserve` - Whether to preserve leading whitespace in text/annotations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::parser::ParserConfig;
    ///
    /// let config = ParserConfig::default().with_preserve_indent(true);
    /// ```
    pub fn with_preserve_indent(mut self, preserve: bool) -> Self {
        self.preserve_indent = preserve;
        self
    }

    /// Set whether to preserve empty lines as text commands
    ///
    /// # Arguments
    /// * `preserve` - Whether to preserve empty lines as text commands
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::parser::ParserConfig;
    ///
    /// let config = ParserConfig::default().with_preserve_empty_lines(true);
    /// ```
    pub fn with_preserve_empty_lines(mut self, preserve: bool) -> Self {
        self.preserve_empty_lines = preserve;
        self
    }
}

/// Core KoiLang parser
///
/// The parser processes input line by line and produces `ParsedCommand` structures
/// for each logical unit (commands, text lines, annotations). It supports streaming
/// processing for memory-efficient parsing of large files.
pub struct Parser<T: TextInputSource> {
    input: Input<T>,
    config: ParserConfig,
}

impl<T: TextInputSource> Parser<T> {
    /// Create a new parser with the specified configuration
    ///
    /// # Arguments
    /// * `input_source` - The source of text input
    /// * `config` - Parser configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::parser::{Parser, ParserConfig, StringInputSource};
    ///
    /// let input = StringInputSource::new("#name \"Test\"");
    /// let config = ParserConfig::default();
    /// let mut parser = Parser::new(input, config);
    /// ```
    pub fn new(input_source: T, config: ParserConfig) -> Self {
        Self {
            input: Input::new(input_source),
            config,
        }
    }

    /// Get the next command from the input stream
    ///
    /// Returns `Ok(None)` when end of input is reached.
    /// Returns `Ok(Some(Command))` when a command is successfully parsed.
    /// Returns `Err(ParseError)` when a parsing error occurs.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::parser::{Parser, ParserConfig, StringInputSource};
    ///
    /// let input = StringInputSource::new("#name \"Test\"");
    /// let config = ParserConfig::default();
    /// let mut parser = Parser::new(input, config);
    ///
    /// while let Some(command) = parser.next_command()? {
    ///     println!("Command: {}", command.name());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn next_command(&mut self) -> ParseResult<Option<Command>> {
        loop {
            let (lineno, line_text) = match self.input.next_line() {
                Ok(Some(line_info)) => line_info,
                Ok(None) => {
                    return Ok(None);
                }
                Err(e) => {
                    return Err(ParseError::io(e).with_source(
                        &self.input,
                        self.input.line_number,
                        "".to_owned(),
                    ));
                }
            };
            let trimmed = line_text.trim();
            if trimmed.is_empty() {
                if self.config.preserve_empty_lines {
                    break Ok(Some(Command::new_text("")));
                }
                continue;
            }

            // Count leading # characters
            let hash_count = trimmed.chars().take_while(|&c| c == '#').count();

            if hash_count < self.config.command_threshold {
                let text_content = if self.config.preserve_indent {
                    line_text.trim_end().to_string()
                } else {
                    trimmed.to_string()
                };
                break Ok(Some(Command::new_text(text_content)));
            } else if hash_count > self.config.command_threshold {
                if self.config.skip_annotations {
                    continue;
                }
                let annotation_content = if self.config.preserve_indent {
                    line_text.trim_end().to_string()
                } else {
                    let content: String = trimmed.chars().skip(hash_count).collect();
                    content.trim().to_string()
                };
                break Ok(Some(Command::new_annotation(annotation_content)));
            } else {
                // hash_count == self.config.command_threshold
                let column = line_text.offset(trimmed) + hash_count;
                let command_str: String = trimmed.chars().skip(hash_count).collect();
                break self
                    .parse_command_line(command_str, lineno, column)
                    .map_err(|e| e.with_source(&self.input, lineno, line_text));
            }
        }
    }

    /// Parse a command line
    ///
    /// This is an internal method that handles the actual parsing of command syntax.
    /// It processes the text after the command prefix (#) and creates the appropriate
    /// Command structure.
    ///
    /// # Arguments
    /// * `command_text` - The text content of the command (without the # prefix)
    /// * `lineno` - The line number in the source file
    /// * `column` - The column number in the source file
    pub fn parse_command_line(
        &self,
        command_text: String,
        lineno: usize,
        column: usize,
    ) -> ParseResult<Option<Command>> {
        if command_text.is_empty() {
            return Err(ParseError::syntax_with_context(
                "Empty command line".to_string(),
                lineno,
                column,
                command_text,
            ));
        }

        let result = command_parser::parse_command_line::<NomErrorNode<&str>>(&command_text);

        match result {
            Ok(("", command)) => {
                let num_name = command.name().parse();
                match num_name {
                    Result::Err(_) => Ok(Some(command)),
                    Result::Ok(num) => {
                        if !self.config.convert_number_command {
                            Ok(Some(command))
                        } else {
                            Ok(Some(Command::new_number(num, command.params)))
                        }
                    }
                }
            }
            Ok((remaining, _)) => Err(ParseError::unexpected_input(
                remaining.to_string(),
                lineno,
                column,
                command_text,
            )),
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                // Create a simple nom error for compatibility
                Err(ParseError::from_nom_error(
                    "Command parsing error".to_string(),
                    command_text.as_str(),
                    lineno,
                    column,
                    e,
                ))
            }
            Err(nom::Err::Incomplete(_)) => {
                Err(ParseError::unexpected_eof(command_text, lineno, column))
            }
        }
    }

    /// Process all commands using a callback function
    ///
    /// This provides a streaming interface where each parsed command is
    /// passed to the provided handler function. This is the most memory-efficient
    /// way to process large files.
    ///
    /// # Arguments
    /// * `handler` - Function to call for each parsed command. Should return:
    ///   * `Ok(true)` to continue processing
    ///   * `Ok(false)` to stop processing
    ///   * `Err(e)` to propagate an error
    ///
    /// # Returns
    /// * `Ok(true)` if processing completed and reached EOF
    /// * `Ok(false)` if processing was stopped early by the handler
    /// * `Err(E)` if the handler returned an error or a parse error occurred
    ///
    /// # Examples
    ///
    /// ```rust
    /// use koicore::parser::{Parser, ParserConfig, StringInputSource};
    ///
    /// let input = StringInputSource::new("#name \"Test\"\nHello");
    /// let config = ParserConfig::default();
    /// let mut parser = Parser::new(input, config);
    ///
    /// let reached_eof = parser.process_with(|command| -> Result<bool, Box<dyn std::error::Error>> {
    ///     match command.name() {
    ///         "@text" => {
    ///             println!("Text: {}", command);
    ///             Ok(true) // Continue processing
    ///         },
    ///         "@annotation" => {
    ///             println!("Annotation: {}", command);
    ///             Ok(false) // Stop processing after this command
    ///         },
    ///         _ => {
    ///             println!("Command: {}", command);
    ///             Ok(true) // Continue processing
    ///         },
    ///     }
    /// })?;
    ///
    /// assert!(reached_eof, "Should have reached end of file");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn process_with<F, E>(&mut self, mut handler: F) -> Result<bool, E>
    where
        F: FnMut(Command) -> Result<bool, E>,
        E: From<Box<ParseError>>,
    {
        loop {
            match self.next_command() {
                Ok(Some(command)) => {
                    let should_continue = handler(command)?;
                    if !should_continue {
                        return Ok(false); // Stopped early by handler
                    }
                }
                Ok(None) => {
                    return Ok(true); // Reached EOF
                } // End of input
                Err(e) => {
                    return Err(e.into());
                } // Convert ParseError to E
            }
        }
    }

    /// Get the current line number
    ///
    /// Returns the line number that the parser is currently processing.
    /// This is useful for error reporting and progress tracking.
    pub fn current_line(&self) -> usize {
        self.input.line_number
    }
}

impl<T: TextInputSource> AsRef<T> for Parser<T> {
    fn as_ref(&self) -> &T {
        &self.input.source
    }
}

impl<T: TextInputSource> AsMut<T> for Parser<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.input.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::error::ParseError;

    #[test]
    fn test_parser_config() {
        let config = ParserConfig::default();
        assert_eq!(config.command_threshold, 1);
        assert!(!config.skip_annotations);
        assert!(config.convert_number_command);
        assert!(!config.preserve_indent);
        assert!(!config.preserve_empty_lines);

        let config = ParserConfig::new(2, true, false, true, true);
        assert_eq!(config.command_threshold, 2);
        assert!(config.skip_annotations);
        assert!(!config.convert_number_command);
        assert!(config.preserve_indent);
        assert!(config.preserve_empty_lines);

        let config = ParserConfig::default()
            .with_command_threshold(3)
            .with_skip_annotations(true)
            .with_convert_number_command(false)
            .with_preserve_indent(true)
            .with_preserve_empty_lines(true);
        assert_eq!(config.command_threshold, 3);
        assert!(config.skip_annotations);
        assert!(!config.convert_number_command);
        assert!(config.preserve_indent);
        assert!(config.preserve_empty_lines);
    }

    #[test]
    fn test_preserve_indent() {
        let input = StringInputSource::new("  indented text\nnormal text");
        
        let config = ParserConfig::default();
        let mut parser = Parser::new(input, config);
        let cmd = parser.next_command().unwrap().unwrap();
        assert_eq!(cmd.name(), "@text");
        assert_eq!(cmd.params()[0].to_string(), "\"indented text\"");
        
        let input = StringInputSource::new("  indented text\nnormal text");
        let config = ParserConfig::default().with_preserve_indent(true);
        let mut parser = Parser::new(input, config);
        let cmd = parser.next_command().unwrap().unwrap();
        assert_eq!(cmd.name(), "@text");
        assert_eq!(cmd.params()[0].to_string(), "\"  indented text\"");
    }

    #[test]
    fn test_preserve_empty_lines() {
        let input = StringInputSource::new("text1\n\ntext2");
        
        let config = ParserConfig::default();
        let mut parser = Parser::new(input, config);
        let cmd1 = parser.next_command().unwrap().unwrap();
        assert_eq!(cmd1.name(), "@text");
        assert_eq!(cmd1.params()[0].to_string(), "text1");
        let cmd2 = parser.next_command().unwrap().unwrap();
        assert_eq!(cmd2.name(), "@text");
        assert_eq!(cmd2.params()[0].to_string(), "text2");
        
        let input = StringInputSource::new("text1\n\ntext2");
        let config = ParserConfig::default().with_preserve_empty_lines(true);
        let mut parser = Parser::new(input, config);
        let cmd1 = parser.next_command().unwrap().unwrap();
        assert_eq!(cmd1.name(), "@text");
        assert_eq!(cmd1.params()[0].to_string(), "text1");
        let cmd_empty = parser.next_command().unwrap().unwrap();
        assert_eq!(cmd_empty.name(), "@text");
        assert_eq!(cmd_empty.params()[0].to_string(), "\"\"");
        let cmd2 = parser.next_command().unwrap().unwrap();
        assert_eq!(cmd2.name(), "@text");
        assert_eq!(cmd2.params()[0].to_string(), "text2");
    }

    #[test]
    fn test_preserve_indent_annotation() {
        let input = StringInputSource::new("##  annotation text");
        
        let config = ParserConfig::default();
        let mut parser = Parser::new(input, config);
        let cmd = parser.next_command().unwrap().unwrap();
        assert_eq!(cmd.name(), "@annotation");
        assert_eq!(cmd.params()[0].to_string(), "\"annotation text\"");
        
        let input = StringInputSource::new("##  annotation text");
        let config = ParserConfig::default().with_preserve_indent(true);
        let mut parser = Parser::new(input, config);
        let cmd = parser.next_command().unwrap().unwrap();
        assert_eq!(cmd.name(), "@annotation");
        assert_eq!(cmd.params()[0].to_string(), "\"##  annotation text\"");
    }

    #[test]
    fn test_parser_process_with() {
        let input = StringInputSource::new("#cmd1\n#cmd2");
        let config = ParserConfig::default();
        let mut parser = Parser::new(input, config);

        let mut commands = Vec::new();
        // Explicitly specify the Error type for the result
        let result: Result<bool, Box<ParseError>> = parser.process_with(|cmd| {
            commands.push(cmd.name().to_string());
            Ok(true)
        });

        assert!(result.is_ok());
        assert!(result.unwrap()); // EOF reached
        assert_eq!(commands, vec!["cmd1", "cmd2"]);
    }

    #[test]
    fn test_parser_process_with_early_stop() {
        let input = StringInputSource::new("#cmd1\n#cmd2");
        let config = ParserConfig::default();
        let mut parser = Parser::new(input, config);

        let mut commands = Vec::new();
        let result: Result<bool, Box<ParseError>> = parser.process_with(|cmd| {
            commands.push(cmd.name().to_string());
            Ok(false) // Stop after first command
        });

        assert!(result.is_ok());
        assert!(!result.unwrap()); // Stopped early
        assert_eq!(commands, vec!["cmd1"]);

        // Next command should be available
        let next = parser.next_command().unwrap();
        assert!(next.is_some());
        assert_eq!(next.unwrap().name(), "cmd2");
    }

    #[test]
    fn test_parser_current_line() {
        let input = StringInputSource::new("#cmd1\n#cmd2");
        let config = ParserConfig::default();
        let mut parser = Parser::new(input, config);

        assert_eq!(parser.current_line(), 1);
        parser.next_command().unwrap();
        assert_eq!(parser.current_line(), 2);
    }
}
