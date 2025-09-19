//! Streaming decoder buffer reader for KoiLang
//!
//! This module provides a `DecodeBufReader` that wraps encoding_rs's streaming
//! decoder to provide efficient, buffered decoding of text streams with various
//! encodings. Implements BufRead for seamless integration with Rust's I/O traits.

use super::input::EncodingErrorStrategy;
use encoding_rs::{Decoder, Encoding};
use std::io::{self, BufRead, Read};

const DEFAULT_BUFFER_SIZE: usize = 8192;
const DEFAULT_READ_CHUNK_SIZE: usize = 1024;

/// Options for configuring a DecodeBufReader
///
/// This struct holds the configuration options for a `DecodeBufReader`.
pub struct DecodeBufReaderOptions {
    /// The encoding to use for decoding
    pub encoding: &'static Encoding,
    /// The error handling strategy to use
    pub encoding_strategy: EncodingErrorStrategy,
    /// The size of the internal buffer to use for decoding
    pub buffer_size: usize,
    /// The size of the chunk to read from the underlying reader at a time
    pub read_chunk_size: usize,
}

impl Default for DecodeBufReaderOptions {
    fn default() -> Self {
        Self {
            encoding: encoding_rs::UTF_8,
            encoding_strategy: EncodingErrorStrategy::Replace,
            buffer_size: DEFAULT_BUFFER_SIZE,
            read_chunk_size: DEFAULT_READ_CHUNK_SIZE,
        }
    }
}

impl DecodeBufReaderOptions {
    /// Create a new DecodeBufReaderOptions with custom options
    ///
    /// # Arguments
    /// * `encoding` - The encoding to use for decoding
    /// * `encoding_strategy` - The error handling strategy to use
    /// * `buffer_size` - The size of the internal buffer to use for decoding
    /// * `read_chunk_size` - The size of the chunk to read from the underlying reader at a time
    pub fn new(
        encoding: &'static Encoding,
        encoding_strategy: EncodingErrorStrategy,
        buffer_size: usize,
        read_chunk_size: usize,
    ) -> Self {
        Self {
            encoding,
            encoding_strategy,
            buffer_size,
            read_chunk_size,
        }
    }

    /// Create a new DecodeBufReaderOptions with a specific encoding
    ///
    /// # Arguments
    /// * `encoding` - The encoding to use for decoding
    pub fn with_encoding(mut self, encoding: &'static Encoding) -> Self {
        self.encoding = encoding;
        self
    }

    /// Create a new DecodeBufReaderOptions with a specific encoding error strategy
    ///
    /// # Arguments
    /// * `encoding_strategy` - The error handling strategy to use
    pub fn with_encoding_strategy(mut self, encoding_strategy: EncodingErrorStrategy) -> Self {
        self.encoding_strategy = encoding_strategy;
        self
    }

    /// Create a new DecodeBufReaderOptions with a specific buffer size
    ///
    /// # Arguments
    /// * `buffer_size` - The size of the internal buffer to use for decoding
    pub fn with_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    /// Create a new DecodeBufReaderOptions with a specific read chunk size
    ///
    /// # Arguments
    /// * `read_chunk_size` - The size of the chunk to read from the underlying reader at a time
    pub fn with_read_chunk_size(mut self, read_chunk_size: usize) -> Self {
        self.read_chunk_size = read_chunk_size;
        self
    }
}

/// A buffered reader that decodes text streams using encoding_rs
///
/// This reader provides streaming decoding capabilities with efficient
/// buffer management and configurable error handling.
pub struct DecodeBufReader<R> {
    /// The underlying reader
    reader: R,
    /// The decoder for the specific encoding
    decoder: Decoder,
    /// Internal buffer for raw bytes from the reader
    buffer: Vec<u8>,
    /// Current position in the buffer
    buffer_pos: usize,
    /// Number of valid bytes in the buffer
    buffer_len: usize,
    /// Output buffer for decoded text
    output_buffer: String,
    /// Error handling strategy
    encoding_strategy: EncodingErrorStrategy,
    /// Size of the read chunk
    read_chunk_size: usize,
    /// Whether we've reached EOF
    finished: bool,
}

impl<R: Read> DecodeBufReader<R> {
    /// Create a new DecodeBufReader with UTF-8 encoding
    ///
    /// # Arguments
    /// * `reader` - The underlying reader to decode from
    pub fn new(reader: R) -> Self {
        Self::with_encoding(reader, encoding_rs::UTF_8)
    }

    /// Create a new DecodeBufReader with a specific encoding
    ///
    /// # Arguments
    /// * `reader` - The underlying reader to decode from
    /// * `encoding` - The encoding to use for decoding
    pub fn with_encoding(reader: R, encoding: &'static Encoding) -> Self {
        Self::with_encoding_and_strategy(reader, encoding, EncodingErrorStrategy::Replace)
    }

    /// Create a new DecodeBufReader with specific encoding and error strategy
    ///
    /// # Arguments
    /// * `reader` - The underlying reader to decode from
    /// * `encoding` - The encoding to use for decoding
    /// * `strategy` - The error handling strategy to use
    pub fn with_encoding_and_strategy(
        reader: R,
        encoding: &'static Encoding,
        strategy: EncodingErrorStrategy,
    ) -> Self {
        Self {
            reader,
            decoder: encoding.new_decoder(),
            buffer: vec![0; DEFAULT_BUFFER_SIZE], // 8KB buffer
            buffer_pos: 0,
            buffer_len: 0,
            output_buffer: String::new(),
            encoding_strategy: strategy,
            finished: false,
            read_chunk_size: DEFAULT_READ_CHUNK_SIZE,
        }
    }

    /// Create a new DecodeBufReader with custom options
    ///
    /// # Arguments
    /// * `reader` - The underlying reader to decode from
    /// * `options` - The options to use for configuring the reader
    pub fn with_options(reader: R, options: DecodeBufReaderOptions) -> Self {
        Self {
            reader,
            read_chunk_size: options.read_chunk_size,
            decoder: options.encoding.new_decoder(),
            buffer: vec![0; options.buffer_size],
            buffer_pos: 0,
            buffer_len: 0,
            output_buffer: String::new(),
            encoding_strategy: options.encoding_strategy,
            finished: false,
        }
    }

    /// Fill the internal buffer with data from the reader
    ///
    /// Returns the number of bytes read, or an error if reading failed.
    fn fill_buffer(&mut self) -> io::Result<usize> {
        if self.finished {
            return Ok(0);
        }

        // Shift remaining data to the beginning of the buffer
        if self.buffer_pos > 0 && self.buffer_pos < self.buffer_len {
            let remaining = self.buffer_len - self.buffer_pos;
            self.buffer.copy_within(self.buffer_pos..self.buffer_len, 0);
            self.buffer_len = remaining;
            self.buffer_pos = 0;
        } else if self.buffer_pos >= self.buffer_len {
            self.buffer_len = 0;
            self.buffer_pos = 0;
        }

        // Read new data into the buffer
        let bytes_read = self.reader.read(&mut self.buffer[self.buffer_len..])?;
        self.buffer_len += bytes_read;

        if bytes_read == 0 {
            self.finished = true;
        }

        Ok(bytes_read)
    }

    /// Decode a chunk of data into the output buffer
    ///
    /// # Arguments
    /// * `max_chars` - Maximum number of characters to decode (approximate)
    ///
    /// Returns `Ok(true)` if more data is available, `Ok(false)` if EOF reached,
    /// or an error if decoding failed.
    pub fn decode_chunk(&mut self, max_chars: usize) -> io::Result<bool> {
        if self.finished && self.buffer_pos >= self.buffer_len {
            return Ok(false);
        }

        // Ensure we have data to decode
        if self.buffer_pos >= self.buffer_len && !self.finished {
            self.fill_buffer()?;
        }

        if self.buffer_pos >= self.buffer_len {
            return Ok(false);
        }

        // Calculate how much data to decode
        let available_bytes = self.buffer_len - self.buffer_pos;
        let bytes_to_decode = available_bytes.min(4096); // Limit per chunk

        // Reserve space in output buffer
        self.output_buffer.reserve(max_chars);

        // Decode the data
        let (result, bytes_read, had_errors) = self.decoder.decode_to_string(
            &self.buffer[self.buffer_pos..self.buffer_pos + bytes_to_decode],
            &mut self.output_buffer,
            self.finished && (self.buffer_pos + bytes_to_decode >= self.buffer_len),
        );

        self.buffer_pos += bytes_read;

        // Handle encoding errors according to strategy
        if had_errors {
            match self.encoding_strategy {
                EncodingErrorStrategy::Strict => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Invalid encoding detected",
                    ));
                }
                EncodingErrorStrategy::Replace => {
                    // encoding_rs already replaces with U+FFFD, we can leave as-is
                    // or replace with custom replacement if needed
                }
                EncodingErrorStrategy::Ignore => {
                    // Remove replacement characters
                    if self.output_buffer.contains('\u{FFFD}') {
                        self.output_buffer = self.output_buffer.replace('\u{FFFD}', "");
                    }
                }
            }
        }

        // If we need more data and haven't finished, fill buffer again
        if result == encoding_rs::CoderResult::InputEmpty && !self.finished {
            self.fill_buffer()?;
        }

        Ok(true)
    }

    /// Get the current decoded content and clear the output buffer
    ///
    /// Returns the decoded string, or None if no content is available.
    pub fn take_string(&mut self) -> Option<String> {
        if self.output_buffer.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.output_buffer))
        }
    }

    /// Check if decoding is complete
    pub fn is_finished(&self) -> bool {
        self.finished && self.buffer_pos >= self.buffer_len
    }

    /// Get a reference to the underlying reader
    pub fn get_ref(&self) -> &R {
        &self.reader
    }

    /// Get a mutable reference to the underlying reader
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Consume the reader and return the underlying reader
    pub fn into_inner(self) -> R {
        self.reader
    }
}

impl<R: Read> Read for DecodeBufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // This is a bit tricky since we're dealing with decoded text
        // For now, we'll decode chunks until we have enough data
        while self.output_buffer.len() < buf.len() && !self.is_finished() {
            self.decode_chunk(buf.len())?;
        }

        let len = buf.len().min(self.output_buffer.len());
        if len > 0 {
            let text_bytes = self.output_buffer.as_bytes();
            buf[..len].copy_from_slice(&text_bytes[..len]);
            self.output_buffer.drain(..len);
        }
        Ok(len)
    }
}

impl<R: Read> BufRead for DecodeBufReader<R> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        // Ensure we have decoded content
        while self.output_buffer.is_empty() && !self.is_finished() {
            self.decode_chunk(self.read_chunk_size)?;
        }

        Ok(self.output_buffer.as_bytes())
    }

    fn consume(&mut self, amt: usize) {
        if amt >= self.output_buffer.len() {
            self.output_buffer.clear();
        } else {
            self.output_buffer.drain(..amt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_utf8_decoding() {
        let data = "Hello, 世界!\n测试数据".as_bytes();
        let cursor = Cursor::new(data);
        let mut decoder = DecodeBufReader::new(cursor);

        // Decode in chunks
        assert!(decoder.decode_chunk(100).unwrap());
        let result = decoder.take_string().unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_empty_input() {
        let data: &[u8] = &[];
        let cursor = Cursor::new(data);
        let mut decoder = DecodeBufReader::new(cursor);

        assert!(!decoder.decode_chunk(100).unwrap());
        assert!(decoder.take_string().is_none());
        assert!(decoder.is_finished());
    }

    #[test]
    fn test_buf_read_trait() {
        let data = "Line 1\nLine 2\nLine 3".as_bytes();
        let cursor = Cursor::new(data);
        let mut decoder = DecodeBufReader::new(cursor);

        let mut line = String::new();
        let bytes_read = decoder.read_line(&mut line).unwrap();
        assert!(bytes_read > 0);
        assert_eq!(line, "Line 1\n");
    }

    #[test]
    fn test_small_buffer_size() {
        let data =
            "Hello, 世界! This is a long text that should test buffer boundaries.".as_bytes();
        let cursor = Cursor::new(data);
        let options = DecodeBufReaderOptions::default()
            .with_buffer_size(16)
            .with_read_chunk_size(8);
        let mut decoder = DecodeBufReader::with_options(cursor, options);

        let mut all_content = String::new();
        while decoder.decode_chunk(100).unwrap() {
            if let Some(content) = decoder.take_string() {
                all_content.push_str(&content);
            }
        }
        assert!(!all_content.is_empty());
        assert!(all_content.contains("世界"));
    }

    #[test]
    fn test_multibyte_character_split() {
        let chinese_text = "这是一个很长的中文文本，用于测试多字节字符处理";
        let data = chinese_text.as_bytes();
        let cursor = Cursor::new(data);
        let options = DecodeBufReaderOptions::default().with_buffer_size(10); // 小缓冲区强制分割字符
        let mut decoder = DecodeBufReader::with_options(cursor, options);

        let mut decoded_content = String::new();
        while decoder.decode_chunk(50).unwrap() {
            if let Some(content) = decoder.take_string() {
                decoded_content.push_str(&content);
            }
        }
        assert_eq!(decoded_content, chinese_text);
    }

    #[test]
    fn test_buffer_refill_edge_cases() {
        let data = "Short\n".as_bytes();
        let cursor = Cursor::new(data);
        let options = DecodeBufReaderOptions::default()
            .with_buffer_size(4) // 比一行还小的缓冲区
            .with_read_chunk_size(2);
        let mut decoder = DecodeBufReader::with_options(cursor, options);

        let mut buffer = vec![0u8; 0];
        let bytes_read = decoder.read_until(b'\n', &mut buffer).unwrap();
        assert_eq!(bytes_read, 6);
        assert_eq!(&buffer[..bytes_read], b"Short\n");
    }

    #[test]
    fn test_encoding_error_replace() {
        let mixed_data = vec![b'H', b'e', b'l', b'l', b'o', 0xFF, 0xFE, b'!', b'\n'];
        let cursor = Cursor::new(mixed_data);
        let options = DecodeBufReaderOptions::default()
            .with_encoding_strategy(EncodingErrorStrategy::Replace);
        let mut decoder = DecodeBufReader::with_options(cursor, options);

        assert!(decoder.decode_chunk(100).unwrap());
        let content = decoder.take_string().unwrap();
        assert!(content.contains('�'));
        assert!(content.contains("Hello"));
        assert!(content.contains("!"));
    }

    #[test]
    fn test_encoding_error_ignore() {
        let mixed_data = vec![b'H', b'e', b'l', b'l', b'o', 0xFF, 0xFE, b'!', b'\n'];
        let cursor = Cursor::new(mixed_data);
        let options =
            DecodeBufReaderOptions::default().with_encoding_strategy(EncodingErrorStrategy::Ignore);
        let mut decoder = DecodeBufReader::with_options(cursor, options);

        assert!(decoder.decode_chunk(100).unwrap());
        let content = decoder.take_string().unwrap();
        assert!(!content.contains('�'));
        assert!(content.contains("Hello"));
        assert!(content.contains("!"));
    }

    #[test]
    fn test_large_data_processing() {
        let large_text = "Large data test. ".repeat(1000);
        let data = large_text.as_bytes();
        let cursor = Cursor::new(data);
        let mut decoder = DecodeBufReader::new(cursor);

        let mut total_decoded = String::new();
        while decoder.decode_chunk(500).unwrap() {
            if let Some(content) = decoder.take_string() {
                total_decoded.push_str(&content);
            }
        }
        assert_eq!(total_decoded.len(), large_text.len());
        assert_eq!(total_decoded, large_text);
    }

    #[test]
    fn test_chunk_boundary_handling() {
        let boundary_text = "A".repeat(100) + &"B".repeat(50) + &"C".repeat(25);
        let data = boundary_text.as_bytes();
        let cursor = Cursor::new(data);
        let options = DecodeBufReaderOptions::default().with_read_chunk_size(30); // 特定的块大小
        let mut decoder = DecodeBufReader::with_options(cursor, options);

        let mut all_content = String::new();
        while decoder.decode_chunk(40).unwrap() {
            if let Some(content) = decoder.take_string() {
                all_content.push_str(&content);
            }
        }
        assert_eq!(all_content, boundary_text);
    }

    #[test]
    fn test_empty_chunks() {
        let data = "Test\n\n\nData".as_bytes();
        let cursor = Cursor::new(data);
        let mut decoder = DecodeBufReader::new(cursor);

        let mut results = Vec::new();
        while decoder.decode_chunk(10).unwrap() {
            if let Some(content) = decoder.take_string() {
                results.push(content);
            }
        }

        let final_content = results.join("");
        assert!(final_content.contains("Test"));
        assert!(final_content.contains("Data"));
    }

    #[test]
    fn test_read_trait_edge_cases() {
        let data = "Short test data".as_bytes();
        let cursor = Cursor::new(data);
        let mut decoder = DecodeBufReader::new(cursor);

        let mut buf = vec![0; 4];
        let mut total_read = 0;

        loop {
            match decoder.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => total_read += n,
                Err(e) => panic!("Read error: {}", e),
            }
        }
        assert!(total_read > 0);
    }

    #[test]
    fn test_bufread_line_splitting() {
        let line_data = "Line1\nLine2\r\nLine3\n".as_bytes();
        let cursor = Cursor::new(line_data);
        let mut decoder = DecodeBufReader::new(cursor);

        let mut lines = Vec::new();
        let mut line = String::new();

        while decoder.read_line(&mut line).unwrap() > 0 {
            lines.push(line.clone());
            line.clear();
        }

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line1\n");
        assert_eq!(lines[1], "Line2\r\n");
        assert_eq!(lines[2], "Line3\n");
    }

    #[test]
    fn test_zero_sized_operations() {
        let data = "Test data".as_bytes();
        let cursor = Cursor::new(data);
        let mut decoder = DecodeBufReader::new(cursor);

        assert!(decoder.decode_chunk(0).unwrap());

        let mut zero_buf = vec![0; 0];
        assert_eq!(decoder.read(&mut zero_buf).unwrap(), 0);

        assert!(decoder.decode_chunk(10).unwrap());
        assert!(decoder.take_string().is_some());
    }
}
