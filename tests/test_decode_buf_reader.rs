//! Tests for the DecodeBufReader functionality

use koicore::parser::decode_buf_reader::DecodeBufReader;
use koicore::parser::input::EncodingErrorStrategy;
use std::io::{BufRead, Cursor};

#[test]
fn test_decode_buf_reader_utf8() {
    let data = "Hello, 世界!\n测试数据\nLine 3".as_bytes();
    let cursor = Cursor::new(data);
    let mut decoder = DecodeBufReader::new(cursor);

    // Test reading lines
    let mut line = String::new();
    let bytes_read = decoder.read_line(&mut line).unwrap();
    assert!(bytes_read > 0);
    assert_eq!(line, "Hello, 世界!\n");

    line.clear();
    let bytes_read = decoder.read_line(&mut line).unwrap();
    assert!(bytes_read > 0);
    assert_eq!(line, "测试数据\n");

    line.clear();
    let bytes_read = decoder.read_line(&mut line).unwrap();
    assert!(bytes_read > 0);
    assert_eq!(line, "Line 3");

    // Test EOF
    line.clear();
    let bytes_read = decoder.read_line(&mut line).unwrap();
    assert_eq!(bytes_read, 0);
    assert_eq!(line, "");
}

#[test]
fn test_decode_buf_reader_chunk_decoding() {
    let data = "Hello, 世界!\n测试数据\nLine 3".as_bytes();
    let cursor = Cursor::new(data);
    let mut decoder = DecodeBufReader::new(cursor);

    // Test chunk decoding
    assert!(decoder.decode_chunk(100).unwrap());
    let result = decoder.take_string().unwrap();
    assert!(!result.is_empty());
    assert!(result.contains("Hello"));
}

#[test]
fn test_decode_buf_reader_empty_input() {
    let data: &[u8] = &[];
    let cursor = Cursor::new(data);
    let mut decoder = DecodeBufReader::new(cursor);

    let mut line = String::new();
    let bytes_read = decoder.read_line(&mut line).unwrap();
    assert_eq!(bytes_read, 0);
    assert_eq!(line, "");
}

#[test]
fn test_decode_buf_reader_with_encoding() {
    // Test with GBK encoding
    let data = vec![0xC4, 0xE3, 0xBA, 0xC3, 0x0A, 0xCA, 0xC0, 0xBD, 0xE7]; // "你好\n世界" in GBK
    let cursor = Cursor::new(data);
    let mut decoder = DecodeBufReader::with_encoding(cursor, encoding_rs::GBK);

    let mut line = String::new();
    let bytes_read = decoder.read_line(&mut line).unwrap();
    assert!(bytes_read > 0);
    assert!(line.contains("好")); // Should contain "你好\n"
}

#[test]
fn test_decode_buf_reader_error_handling() {
    // Test with strict error handling
    let data = vec![0xC4, 0xE3, 0xBA, 0xC3, 0x0A, 0xFF, 0xFF]; // Invalid UTF-8
    let cursor = Cursor::new(data);
    let mut decoder = DecodeBufReader::with_encoding_and_strategy(
        cursor, 
        encoding_rs::UTF_8, 
        EncodingErrorStrategy::Strict
    );

    let mut line = String::new();
    let result = decoder.read_line(&mut line);
    // Should fail with strict mode
    assert!(result.is_err());
}
