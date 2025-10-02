//! KoiLang parser module
//! 
//! This module provides the core parsing functionality for KoiLang files.
//! It supports streaming processing and handles three types of content:
//! - Commands (lines starting with # characters)
//! - Text (regular content lines)
//! - Annotations (lines with more # characters than the threshold)

pub mod command;
pub mod error;
pub mod traceback;
pub mod input;
pub mod decode_buf_reader;
mod command_parser;

pub use command::{Command, Parameter, Value};
pub use error::{ParseError, ParseResult, ErrorInfo, TracebackEntry, Traceback};
pub use input::{TextInputSource, FileInputSource, StringInputSource};

use input::Input;

/// Configuration for the line processor
#[repr(C)]
pub struct ParserConfig {
    /// The command threshold (number of # required for commands)
    pub command_threshold: usize,
}

impl Default for ParserConfig {
    fn default() -> Self {
        Self {
            command_threshold: 1,
        }
    }
}

/// Core KoiLang parser
/// 
/// The parser processes input line by line and produces `ParsedCommand` structures
/// for each logical unit (commands, text lines, annotations).
pub struct Parser<T: TextInputSource> {
    input: Input<T>,
    config: ParserConfig,
}

impl<T: TextInputSource> Parser<T> {
    /// Create a new parser with the specified threshold
    /// 
    /// # Arguments
    /// * `input_source` - The source of text input
    /// * `threshold` - Number of # characters required to identify a command line
    pub fn new(input_source: T, config: ParserConfig) -> Self {
        Self {
            input: Input::new(input_source),
            config: config,
        }
    }

    /// Get the next command from the input stream
    /// 
    /// Returns `Ok(None)` when end of input is reached.
    /// Returns `Ok(Some(Command))` when a command is successfully parsed.
    /// Returns `Err(ParseError)` when a parsing error occurs.
    pub fn next_command(&mut self) -> ParseResult<Option<Command>> {
        let (mut line_number, mut line) = match self.input.next_line() {
            Ok(Some(line_info)) => line_info,
            Ok(None) => return Ok(None),
            Err(e) => return Err(ParseError::io(e)),
        };
        let mut trimmed = line.trim();
        while trimmed.is_empty() {
            (line_number, line) = match self.input.next_line() {
                Ok(Some(line_info)) => line_info,
                Ok(None) => return Ok(None),
                Err(e) => return Err(ParseError::io(e)),
            };
            trimmed = line.trim();
        }
        
        // Count leading # characters
        let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
        if hash_count < self.config.command_threshold {
            Ok(Some(Command::new_text(trimmed.to_string())))
        } else if hash_count > self.config.command_threshold{
            Ok(Some(Command::new_annotation(trimmed.to_string())))
        } else {
            self.parse_command_line(trimmed.trim_start_matches('#').to_string(), line_number)
        }
    }

    /// Parse a command line
    pub fn parse_command_line(&self, command_text: String, line_number: usize) -> ParseResult<Option<Command>> {
        if command_text.is_empty() {
            return Err(ParseError::syntax_with_context(
                "Empty command line".to_string(), 
                line_number, 
                command_text.find('#').unwrap_or(0),
                command_text
            ));
        }
    
        let result = command_parser::parse_command_line::<nom_language::error::VerboseError<&str>>(&command_text);
            
        match result {
            Ok(("", command)) => {
                Ok(Some(command))
            }
            Ok((remaining, _)) => {
                Err(ParseError::unexpected_input(
                    remaining.to_string(),
                    line_number,
                    command_text
                ))
            }
            Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
                // Create a simple nom error for compatibility
                Err(ParseError::from_nom_error(
                    format!("Unexpected input at column {}", command_text.len()),
                    command_text.as_str(),
                    e.into(),
                ))
            }
            Err(nom::Err::Incomplete(_)) => {
                Err(ParseError::unexpected_eof(command_text, line_number))
            }
        }
    }

    /// Process all commands using a callback function
    /// 
    /// This provides a streaming interface where each parsed command is
    /// passed to the provided handler function.
    /// 
    /// # Arguments
    /// * `handler` - Function to call for each parsed command
    /// 
    /// # Returns
    /// * `Ok(())` if all commands were processed successfully
    /// * `Err(E)` if the handler returned an error
    pub fn process_with<F, E>(&mut self, mut handler: F) -> Result<(), E>
    where
        F: FnMut(Command) -> Result<(), E>,
        E: From<ParseError>,
    {
        loop {
            match self.next_command() {
                Ok(Some(command)) => handler(command)?,
                Ok(None) => break, // End of input
                Err(e) => return Err(e.into()), // Convert ParseError to E
            }
        }
        Ok(())
    }

    /// Get the current line number
    pub fn current_line(&self) -> usize {
        self.input.line_number
    }
}
