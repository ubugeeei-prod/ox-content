//! GFM table allocation workloads. Header width determines every row's size.

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};

fn bench_tables(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_tables");
    for columns in [1, 2, 4, 8, 16, 32] {
        for (name, cell) in [("plain", "value"), ("inline", "**value** with `code`")] {
            let mut source = "| heading ".repeat(columns);
            source.push_str("|\n");
            source.push_str(&"| --- ".repeat(columns));
            source.push_str("|\n");
            for _ in 0..128 {
                for _ in 0..columns {
                    source.push_str("| ");
                    source.push_str(cell);
                    source.push(' ');
                }
                source.push_str("|\n");
            }
            group.throughput(Throughput::Bytes(source.len() as u64));
            group.bench_with_input(BenchmarkId::new(name, columns), &source, |b, source| {
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

criterion_group!(benches, bench_tables);
criterion_main!(benches);
