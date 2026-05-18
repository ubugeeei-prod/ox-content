//! Benchmarks for the documentation extractor.
//!
//! The benchmark focuses on the JSDoc parsing path, which dominates extraction
//! cost for files dense with documentation comments.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use ox_content_docs::DocExtractor;
use oxc_span::SourceType;

const SMALL_TS: &str = r"
/**
 * Adds two numbers together.
 * @param a - The first number
 * @param b - The second number
 * @returns The sum of a and b
 */
export function add(a: number, b: number): number {
    return a + b;
}

/**
 * User interface.
 */
export interface User {
    /** User's name */
    name: string;
    /** User's age */
    age: number;
}
";

fn generate_large_ts(item_count: usize) -> String {
    use std::fmt::Write;
    let mut buf = String::new();
    for index in 0..item_count {
        write!(
            buf,
            r#"
/**
 * Function number {index}.
 *
 * @param {{string}} input - Input value for entry {index}
 * @param {{number}} [count=1] - Optional repetition count
 * @param {{boolean}} [flag] - Optional flag
 * @returns {{string}} Formatted output for entry {index}
 * @example
 *   handler{index}("hello")
 *   //=> "hello"
 */
export function handler{index}(input: string, count: number = 1, flag?: boolean): string {{
    return input.repeat(count);
}}

/**
 * Interface number {index}.
 */
export interface Entry{index} {{
    /** Unique identifier */
    id: string;
    /** Display label */
    label: string;
    /** Optional metadata */
    meta?: Record<string, unknown>;
}}
"#
        )
        .unwrap();
    }
    buf
}

fn bench_extract_small(c: &mut Criterion) {
    let extractor = DocExtractor::new();
    let mut group = c.benchmark_group("extract_small");
    group.throughput(Throughput::Bytes(SMALL_TS.len() as u64));

    group.bench_function("small_ts", |b| {
        b.iter(|| {
            let _ = extractor
                .extract_source(black_box(SMALL_TS), "small.ts", SourceType::ts())
                .unwrap();
        });
    });

    group.finish();
}

fn bench_extract_large(c: &mut Criterion) {
    let extractor = DocExtractor::new();
    let source = generate_large_ts(100);
    let mut group = c.benchmark_group("extract_large");
    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_function("large_ts_100_items", |b| {
        b.iter(|| {
            let _ =
                extractor.extract_source(black_box(&source), "large.ts", SourceType::ts()).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_extract_small, bench_extract_large);
criterion_main!(benches);
