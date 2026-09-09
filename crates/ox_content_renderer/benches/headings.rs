//! Heading output workloads, with parsing outside the measured region.

use std::fmt::Write as _;
use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

fn bench_headings(c: &mut Criterion) {
    let mut group = c.benchmark_group("headings");
    for (name, title) in [
        ("ascii", "API reference"),
        ("unicode", "日本語の見出しと設定ガイド"),
        ("long", "An extended API reference heading describing configuration and options"),
    ] {
        let mut source = String::new();
        for index in 0..512 {
            writeln!(source, "## {title} {index}\n").unwrap();
        }
        let allocator = Allocator::for_source_len(source.len());
        let document = Parser::new(&allocator, &source).parse().unwrap();
        group.throughput(Throughput::Bytes(source.len() as u64));
        for permalinks in [false, true] {
            let case = format!("{name}/permalinks_{permalinks}");
            group.bench_function(case, |b| {
                b.iter(|| {
                    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
                        heading_permalinks: permalinks,
                        ..Default::default()
                    });
                    black_box(renderer.render(black_box(&document)));
                });
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_headings);
criterion_main!(benches);
