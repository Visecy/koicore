//! Input sources for KoiLang parsing
//!
//! This module provides different input sources for the parser, including
//! file-based, string-based, and streaming input.

use super::decode_buf_reader::DecodeBufReader;
use encoding_rs::Encoding;
use std::fmt::Debug;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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

    /// Get the source name (e.g., filename) for error reporting
    ///
    /// # Returns
    /// * The source name (e.g., filename) for error reporting
    fn source_name(&self) -> String {
        "<string>".into()
    }
}

impl<T: TextInputSource + ?Sized> TextInputSource for Box<T> {
    fn next_line(&mut self) -> io::Result<Option<String>> {
        self.as_mut().next_line()
    }

    fn source_name(&self) -> String {
        self.as_ref().source_name()
    }
}

impl<T: TextInputSource + ?Sized> TextInputSource for Arc<Mutex<T>> {
    fn next_line(&mut self) -> io::Result<Option<String>> {
        self.as_ref()
            .lock()
            .map_err(|e| io::Error::other(format!("{}", e)))?
            .next_line()
    }

    fn source_name(&self) -> String {
        self.as_ref()
            .lock()
            .map(|s| s.source_name())
            .unwrap_or("<string>".into())
    }
}

/// Input source that reads from a file with encoding support
pub struct FileInputSource {
    reader: DecodeBufReader<File>,
    encoding_strategy: EncodingErrorStrategy,
    filename: PathBuf,
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
        let filename = path.as_ref().to_path_buf();
        let file = File::open(path)?;
        let reader = if let Some(enc) = encoding {
            DecodeBufReader::with_encoding_and_strategy(file, enc, strategy)
        } else {
            DecodeBufReader::with_encoding_and_strategy(file, encoding_rs::UTF_8, strategy)
        };
        Ok(Self {
            reader,
            filename,
            encoding_strategy: strategy,
        })
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
                            "Invalid encoding detected in strict mode",
                        ));
                    }
                    EncodingErrorStrategy::Replace if has_err => {
                        line = line.replace("\u{FFFD}", "?");
                    }
                    EncodingErrorStrategy::Ignore if has_err => {
                        line = line.replace("\u{FFFD}", "");
                    }
                    _ => {}
                }
                Ok(Some(line.replace("\r\n", "\n")))
            }
            Err(e) => Err(e), // Propagate I/O errors
        }
    }

    fn source_name(&self) -> String {
        // We can enhance this to return the actual filename if needed
        self.filename.to_str().unwrap_or("<unknown>").to_owned()
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
        let lines: Vec<String> = content
            .split_inclusive('\n')
            .map(|s| s.to_string())
            .collect();
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

pub struct BufReadWrapper<R: BufRead>(pub R);

/// Input source that reads from any type implementing `BufRead`
impl<R: BufRead> TextInputSource for BufReadWrapper<R> {
    fn next_line(&mut self) -> io::Result<Option<String>> {
        let mut line = String::new();
        match self.0.read_line(&mut line) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => Ok(Some(line.replace("\r\n", "\n"))),
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
            source,
            line_number: 1,
        }
    }

    pub fn next_line(&mut self) -> io::Result<Option<(usize, String)>> {
        let mut line_cache = String::new();
        let start_line_number = self.line_number;
        loop {
            match self.source.next_line() {
                Ok(Some(line)) => {
                    self.line_number += 1;
                    line_cache.push_str(&line);
                    if !line.ends_with("\\\n") {
                        break Ok(Some((start_line_number, line_cache)));
                    }
                }
                Ok(None) => {
                    if line_cache.is_empty() {
                        break Ok(None);
                    } else {
                        // For the last chunk, we still return the start line number
                        break Ok(Some((start_line_number, line_cache)));
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }
}

impl<T: TextInputSource> AsRef<T> for Input<T> {
    fn as_ref(&self) -> &T {
        &self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::io::Write;

    #[test]
    fn test_buf_read_wrapper() {
        let data = "line1\nline2\r\nline3";
        let cursor = Cursor::new(data);
        let mut source = BufReadWrapper(cursor);

        assert_eq!(source.next_line().unwrap(), Some("line1\n".to_string()));
        assert_eq!(source.next_line().unwrap(), Some("line2\n".to_string()));
        assert_eq!(source.next_line().unwrap(), Some("line3".to_string()));
        assert_eq!(source.next_line().unwrap(), None);
    }

    #[test]
    fn test_input_line_continuation() {
        // Test backslash + newline handling
        let content = "line1\\\n continued\nline2";
        let source = StringInputSource::new(content);
        let mut input = Input::new(source);

        // Expected: "line1\\\n continued" as one logical line
        // Input::next_line returns (line_number, line_content)
        // It accumulates lines ending with \\\n

        // First call should return combined line
        let (lineno, text) = input.next_line().unwrap().unwrap();
        assert_eq!(lineno, 1);
        assert_eq!(text, "line1\\\n continued\n");
        assert_eq!(input.line_number, 3); // 1 + 1 (continued) + 1 (next) -> next will be 3?
        // Let's trace:
        // Start line_number = 1.
        // next_line() calls source.next_line() -> "line1\\\n"
        // self.line_number becomes 2.
        // ends_with("\\\n") is true -> loop continues.
        // next_line() -> " continued\n"
        // self.line_number becomes 3.
        // ends_with("\\\n") false -> break.
        // Returns (original line_number 1, accumulated text).

        let (lineno, text) = input.next_line().unwrap().unwrap();
        assert_eq!(lineno, 3);
        assert_eq!(text, "line2");

        assert!(input.next_line().unwrap().is_none());
    }

    #[test]
    fn test_file_input_source_encoding_strategies() {
        use std::env;
        use std::fs;

        // Create a temporary file with invalid UTF-8
        let mut path = env::temp_dir();
        path.push("koi_test_encoding.txt");

        let invalid_utf8 = b"Hello \xFF World\n";
        {
            let mut file = File::create(&path).unwrap();
            file.write_all(invalid_utf8).unwrap();
        }

        // Test Replace strategy (default)
        {
            let mut source = FileInputSource::new(&path).unwrap();
            let line = source.next_line().unwrap().unwrap();
            // \xFF is invalid in UTF-8, should be replaced with ? in our implementation of FileInputSource (Wait, usually it's \u{FFFD}, but code says: line = line.replace("\u{FFFD}", "?");)
            // DecodeBufReader produces \u{FFFD}. FileInputSource logic replaces it with '?'.
            assert_eq!(line, "Hello ? World\n");
        }

        // Test Ignore strategy
        {
            let mut source =
                FileInputSource::with_encoding(&path, None, EncodingErrorStrategy::Ignore).unwrap();
            let line = source.next_line().unwrap().unwrap();
            assert_eq!(line, "Hello  World\n");
        }

        // Test Strict strategy
        {
            let mut source =
                FileInputSource::with_encoding(&path, None, EncodingErrorStrategy::Strict).unwrap();
            let result = source.next_line();
            assert!(result.is_err());
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
        }

        // Cleanup
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_box_text_input_source() {
        let source = StringInputSource::new("line1\nline2");
        let mut boxed: Box<dyn TextInputSource> = Box::new(source);

        assert_eq!(boxed.source_name(), "<string>");
        assert_eq!(boxed.next_line().unwrap(), Some("line1\n".to_string()));
        assert_eq!(boxed.next_line().unwrap(), Some("line2".to_string()));
        assert_eq!(boxed.next_line().unwrap(), None);
    }

    #[test]
    fn test_arc_mutex_text_input_source() {
        let source = StringInputSource::new("line1\nline2");
        let mut arc_source: Arc<Mutex<StringInputSource>> = Arc::new(Mutex::new(source));

        assert_eq!(arc_source.source_name(), "<string>");
        assert_eq!(arc_source.next_line().unwrap(), Some("line1\n".to_string()));
        assert_eq!(arc_source.next_line().unwrap(), Some("line2".to_string()));
        assert_eq!(arc_source.next_line().unwrap(), None);
    }
}
