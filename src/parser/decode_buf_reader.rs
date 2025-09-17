//! Streaming decoder buffer reader for KoiLang
//! 
//! This module provides a `DecodeBufReader` that wraps encoding_rs's streaming
//! decoder to provide efficient, buffered decoding of text streams with various
//! encodings. Implements BufRead for seamless integration with Rust's I/O traits.

use std::io::{self, Read, BufRead};
use encoding_rs::{Decoder, Encoding};
use super::input::EncodingErrorStrategy;

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
            buffer: vec![0; 8192], // 8KB buffer
            buffer_pos: 0,
            buffer_len: 0,
            output_buffer: String::new(),
            encoding_strategy: strategy,
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
            self.decode_chunk(1024)?;
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

    fn read_line(&mut self, buf: &mut String) -> io::Result<usize> {
        let mut total_read = 0;
        
        loop {
            let buffer = self.fill_buf()?;
            if buffer.is_empty() {
                break;
            }
            
            // Look for newline in current buffer using standard library
            let buffer_str = std::str::from_utf8(buffer)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;
            
            if let Some(pos) = buffer_str.find('\n') {
                // Include the newline in the result
                let line_end = pos + 1;
                buf.push_str(&buffer_str[..line_end]);
                self.consume(line_end);
                total_read += line_end;
                break;
            } else {
                // No newline found, consume entire buffer
                let len = buffer_str.len();
                buf.push_str(buffer_str);
                self.consume(len);
                total_read += len;
            }
        }
        
        Ok(total_read)
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
}
