use std::path::Path;

use koicore::parser;
use criterion::{criterion_group, criterion_main, Criterion};

fn parse_example() {
    let input = parser::FileInputSource::new(Path::new("examples/ktxt/example0.ktxt")).expect("Failed to open file");
    let mut parser = parser::Parser::new(input, parser::ParserConfig::default());
    // just test no error
    parser.process_with(|_| {
        Ok::<(), Box<parser::ParseError>>(())
    }).expect("Failed to process file");
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse_example", |b| b.iter(|| parse_example()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
