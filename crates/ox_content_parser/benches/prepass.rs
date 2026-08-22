//! Benchmarks for the document-wide reference-definition pre-pass.

use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;

// The changelog contains ordinary prose mentioning the literal `]:` but no
// reference definition. That shape used to trigger the full document pre-pass.
const CHANGELOG_DECOY: &str = include_str!("../../../CHANGELOG.md");

const REFERENCE_DEFINITIONS: &str = r#"# Reference definitions

[docs]: https://example.com/docs "Documentation"
[api]: https://example.com/api

Read the [documentation][docs] and [API reference][api].
"#;

fn bench_prepass(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_prepass");

    for (name, source) in
        [("changelog_decoy", CHANGELOG_DECOY), ("reference_definitions", REFERENCE_DEFINITIONS)]
    {
        group.throughput(Throughput::Bytes(source.len() as u64));
        group.bench_with_input(name, source, |b, source| {
            b.iter(|| {
                let allocator = Allocator::new();
                let parser = Parser::new(&allocator, black_box(source));
                let _ = parser.parse();
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_prepass);
criterion_main!(benches);
