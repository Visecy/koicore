//! Input sources for KoiLang parsing
//! 
//! This module provides different input sources for the parser, including
//! file-based, string-based, and streaming input.

use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// Trait for text input sources
/// 
/// This trait allows the parser to work with different types of input sources
/// such as files, strings, or streaming data.
pub trait TextInputSource {
    /// Get the next line from the input source
    /// 
    /// Returns `Some(String)` if a line is available, `None` if end of input is reached.
    fn next_line(&mut self) -> Option<String>;
}

/// Input source that reads from a file
pub struct FileInputSource {
    reader: BufReader<File>,
}

impl FileInputSource {
    /// Create a new file input source
    /// 
    /// # Arguments
    /// * `path` - Path to the file to read
    /// 
    /// # Returns
    /// * `Ok(FileInputSource)` if the file was opened successfully
    /// * `Err(io::Error)` if there was an error opening the file
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(Self { reader })
    }
}

impl TextInputSource for FileInputSource {
    fn next_line(&mut self) -> Option<String> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None, // EOF
            Ok(_) => {
                Some(line.replace("\r\n", "\n"))
            }
            Err(_) => None, // Error reading line
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
    fn next_line(&mut self) -> Option<String> {
        self.lines.next()
    }
}

/// Input source that reads from any type implementing `BufRead`
pub struct StreamInputSource<R: BufRead> {
    reader: R,
}

impl<R: BufRead> StreamInputSource<R> {
    /// Create a new stream input source
    /// 
    /// # Arguments
    /// * `reader` - Any type that implements `BufRead`
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
}

impl<R: BufRead> TextInputSource for StreamInputSource<R> {
    fn next_line(&mut self) -> Option<String> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None, // EOF
            Ok(_) => {
                Some(line.replace("\r\n", "\n"))
            }
            Err(_) => None, // Error reading line
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

    pub fn next_line(&mut self) -> Option<(usize, String)> {
        let mut line_cache = String::new();
        loop {
            let line_number = self.line_number;
            let line = self.source.next_line()?;
            self.line_number += 1;
            line_cache.push_str(&line);
            if !line.ends_with("\\\n") {
                break Some((line_number, line_cache));
            }
        }
    }
}
