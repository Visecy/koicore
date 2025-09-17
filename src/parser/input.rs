//! Input sources for KoiLang parsing
//! 
//! This module provides different input sources for the parser, including
//! file-based, string-based, and streaming input.

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use encoding_rs::Encoding;
use super::decode_buf_reader::DecodeBufReader;

/// Encoding error handling strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodingErrorStrategy {
    /// Strict mode: return an error when encoding conversion fails
    Strict,
    /// Replace mode: replace invalid characters with replacement characters
    Replace,
    /// Ignore mode: skip invalid characters during conversion
    Ignore,
}

/// Trait for text input sources
/// 
/// This trait allows the parser to work with different types of input sources
/// such as files, strings, or streaming data.
pub trait TextInputSource {
    /// Get the next line from the input source
    /// 
    /// Returns `Ok(Some(String))` if a line is available, `Ok(None)` if end of input is reached,
    /// or `Err(io::Error)` if an I/O error occurs.
    fn next_line(&mut self) -> io::Result<Option<String>>;
}

/// Input source that reads from a file with encoding support
pub struct FileInputSource {
    reader: DecodeBufReader<File>,
    encoding_strategy: EncodingErrorStrategy,
}

impl FileInputSource {
    /// Create a new file input source with automatic encoding detection
    /// 
    /// # Arguments
    /// * `path` - Path to the file to read
    /// 
    /// # Returns
    /// * `Ok(FileInputSource)` if the file was opened successfully
    /// * `Err(io::Error)` if there was an error opening the file
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::with_encoding(path, None, EncodingErrorStrategy::Replace)
    }

    /// Create a new file input source with specified encoding
    /// 
    /// # Arguments
    /// * `path` - Path to the file to read
    /// * `encoding` - The encoding to use (None for auto-detection)
    /// * `strategy` - Error handling strategy for encoding conversion
    /// 
    /// # Returns
    /// * `Ok(FileInputSource)` if the file was opened successfully
    /// * `Err(io::Error)` if there was an error opening the file
    pub fn with_encoding<P: AsRef<Path>>(
        path: P,
        encoding: Option<&'static Encoding>,
        strategy: EncodingErrorStrategy,
    ) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = if let Some(enc) = encoding {
            DecodeBufReader::with_encoding_and_strategy(file, enc, strategy)
        } else {
            DecodeBufReader::with_encoding_and_strategy(file, encoding_rs::UTF_8, strategy)
        };
        Ok(Self { reader, encoding_strategy: strategy })
    }

}

impl TextInputSource for FileInputSource {
    fn next_line(&mut self) -> io::Result<Option<String>> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                let has_err = line.contains("\u{FFFD}");
                match self.encoding_strategy {
                    EncodingErrorStrategy::Strict if has_err => {
                        // In strict mode, we should return an error for encoding issues
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Invalid encoding detected in strict mode"
                        ));
                    }
                    EncodingErrorStrategy::Replace if has_err => {
                        line = line.replace("\u{FFFD}", "?");
                    }
                    EncodingErrorStrategy::Ignore if has_err => {
                        line = line.replace("\u{FFFD}", "");
                    }
                    _ => {}
                };
                Ok(Some(line.replace("\r\n", "\n")))
            }
            Err(e) => Err(e), // Propagate I/O errors
        }
    }
}

/// Input source that reads from a string
pub struct StringInputSource {
    lines: std::vec::IntoIter<String>,
}

impl StringInputSource {
    /// Create a new string input source
    /// 
    /// # Arguments
    /// * `content` - The string content to parse
    pub fn new(content: &str) -> Self {
        let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
        Self {
            lines: lines.into_iter(),
        }
    }
}

impl TextInputSource for StringInputSource {
    fn next_line(&mut self) -> io::Result<Option<String>> {
        Ok(self.lines.next())
    }
}

/// Input source that reads from any type implementing `BufRead`
impl<R: BufRead> TextInputSource for R {
    fn next_line(&mut self) -> io::Result<Option<String>> {
        let mut line = String::new();
        match self.read_line(&mut line) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                Ok(Some(line.replace("\r\n", "\n")))
            }
            Err(e) => Err(e), // Propagate I/O errors
        }
    }
}

pub(crate) struct Input<T: TextInputSource> {
    pub source: T,
    pub line_number: usize,
}

impl<T: TextInputSource> Input<T> {
    pub fn new(source: T) -> Self {
        Self {
            source: source,
            line_number: 1,
        }
    }

    pub fn next_line(&mut self) -> io::Result<Option<(usize, String)>> {
        let mut line_cache = String::new();
        loop {
            let line_number = self.line_number;
            match self.source.next_line() {
                Ok(Some(line)) => {
                    self.line_number += 1;
                    line_cache.push_str(&line);
                    if !line.ends_with("\\\n") {
                        break Ok(Some((line_number, line_cache)));
                    }
                }
                Ok(None) => {
                    if line_cache.is_empty() {
                        break Ok(None);
                    } else {
                        self.line_number += 1;
                        break Ok(Some((line_number, line_cache)));
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }
}
