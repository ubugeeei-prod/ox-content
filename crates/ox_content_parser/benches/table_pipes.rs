//! Long table cells with and without escaped pipes.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

fn bench_table_pipes(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_table_pipes");
    for size in [8, 32, 128, 512] {
        for escaped in [false, true] {
            let mut cell = "x".repeat(size);
            if escaped {
                cell.push_str(" \\| 日本語");
            }
            let mut source = String::from("| A | B | C | D |\n| --- | --- | --- | --- |\n");
            for _ in 0..128 {
                for _ in 0..4 {
                    source.push_str("| ");
                    source.push_str(&cell);
                    source.push(' ');
                }
                source.push_str("|\n");
            }
            let kind = if escaped { "escaped" } else { "plain" };
            group.throughput(Throughput::Bytes(source.len() as u64));
            group.bench_with_input(BenchmarkId::new(kind, size), &source, |b, source| {
                b.iter(|| {
                    let allocator = Allocator::for_source_len(source.len());
                    black_box(
                        Parser::with_options(&allocator, black_box(source), ParserOptions::gfm())
                            .parse()
                            .unwrap(),
                    );
                });
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_table_pipes);
criterion_main!(benches);
