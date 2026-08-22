use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

const MARKDOWN: &str = r"# Renderer benchmark

> [!NOTE]
> Visit https://example.com and use **care**.

This paragraph contains *emphasis*, `code`, and a [link](/guide.md).
";

fn bench_fresh_renderer(c: &mut Criterion) {
    let allocator = Allocator::for_source_len(MARKDOWN.len());
    let document = Parser::new(&allocator, MARKDOWN).parse().unwrap();
    let mut group = c.benchmark_group("fresh_renderer");
    group.throughput(Throughput::Bytes(MARKDOWN.len() as u64));

    group.bench_function("static_defaults", |b| {
        b.iter(|| {
            let html = HtmlRenderer::new().render(black_box(&document));
            black_box(html);
        });
    });
    group.bench_function("owned_defaults", |b| {
        b.iter(|| {
            let html = HtmlRenderer::with_options(HtmlRendererOptions::default())
                .render(black_box(&document));
            black_box(html);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_fresh_renderer);
criterion_main!(benches);
