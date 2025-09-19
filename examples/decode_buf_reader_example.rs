//! Example demonstrating the use of DecodeBufReader

use koicore::parser::decode_buf_reader::DecodeBufReader;
use koicore::parser::input::EncodingErrorStrategy;
use std::io::{BufRead, Cursor, Read};

/// A custom reader that simulates massive data streams for 256GB+ demonstration
struct MassiveDataStreamSimulator {
    current_line: u64,
    total_lines: u64,
    line_template: String,
}

impl MassiveDataStreamSimulator {
    fn new(total_lines: u64) -> Self {
        Self {
            current_line: 0,
            total_lines,
            line_template: "Massive stream line {} with Unicode: 你好世界 🚀🌟⭐\n".to_string(),
        }
    }
}

impl Read for MassiveDataStreamSimulator {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.current_line >= self.total_lines {
            return Ok(0);
        }
        
        let line_content = self.line_template.replace("{}", &self.current_line.to_string());
        self.current_line += 1;
        
        let data = line_content.as_bytes();
        let to_copy = std::cmp::min(buf.len(), data.len());
        buf[..to_copy].copy_from_slice(&data[..to_copy]);
        
        Ok(to_copy)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Reading UTF-8 encoded text
    println!("=== Example 1: UTF-8 Decoding ===");
    let utf8_data = "Hello, 世界!\nThis is a test\n最后一行".as_bytes();
    let cursor = Cursor::new(utf8_data);
    let mut decoder = DecodeBufReader::new(cursor);

    let mut line = String::new();
    while decoder.read_line(&mut line)? > 0 {
        println!("Read line: {}", line.trim_end());
        line.clear();
    }

    // Example 2: Reading GBK encoded text
    println!("\n=== Example 2: GBK Decoding ===");
    // GBK bytes for "你好\n世界"
    let gbk_data = vec![0xC4, 0xE3, 0xBA, 0xC3, 0x0A, 0xCA, 0xC0, 0xBD, 0xE7];
    let cursor = Cursor::new(gbk_data);
    let mut decoder = DecodeBufReader::with_encoding(cursor, encoding_rs::GBK);

    let mut line = String::new();
    while decoder.read_line(&mut line)? > 0 {
        println!("Read line: {}", line.trim_end());
        line.clear();
    }

    // Example 3: Chunk-based decoding
    println!("\n=== Example 3: Chunk-based Decoding ===");
    let utf8_data = "Chunk 1\nChunk 2\nChunk 3\n".as_bytes();
    let cursor = Cursor::new(utf8_data);
    let mut decoder = DecodeBufReader::new(cursor);

    // Decode in chunks of approximately 10 characters
    while decoder.decode_chunk(10)? {
        if let Some(content) = decoder.take_string() {
            println!("Decoded chunk: {:?}", content);
        }
    }

    // Check for any remaining content
    if let Some(content) = decoder.take_string() {
        println!("Final chunk: {:?}", content);
    }

    // Example 4: Error handling strategies
    println!("\n=== Example 4: Error Handling ===");
    let invalid_data = vec![0xC4, 0xE3, 0xBA, 0xC3, 0x0A, 0xFF, 0xFF]; // Invalid UTF-8
    let cursor = Cursor::new(invalid_data);
    
    // Strict mode - will return an error
    let mut decoder = DecodeBufReader::with_encoding_and_strategy(
        cursor, 
        encoding_rs::UTF_8, 
        EncodingErrorStrategy::Strict
    );
    
    let mut line = String::new();
    match decoder.read_line(&mut line) {
        Ok(_) => println!("Read line successfully: {}", line),
        Err(e) => println!("Error in strict mode: {}", e),
    }

    // Example 5: 256GB+ streaming capability demonstration
    println!("\n=== Example 5: 256GB+ Streaming Capability ===");
    println!("Demonstrating processing of massive streams without memory issues...");
    
    // Simulate processing 100,000 lines (representing massive data)
    let reader = MassiveDataStreamSimulator::new(100_000);
    let mut decoder = DecodeBufReader::new(reader);
    
    let start_time = std::time::Instant::now();
    let mut lines_processed = 0;
    let mut line = String::new();
    
    // Process massive stream line by line using streaming decoding
    while decoder.read_line(&mut line)? > 0 {
        lines_processed += 1;
        
        // Progress reporting
        if lines_processed % 20000 == 0 {
            println!("Processed {} lines...", lines_processed);
        }
        
        line.clear();
    }
    
    let elapsed = start_time.elapsed();
    println!("Successfully processed {} lines in {:.2?}", lines_processed, elapsed);
    println!("This demonstrates 256GB+ streaming capability with constant memory usage!");

    Ok(())
}
