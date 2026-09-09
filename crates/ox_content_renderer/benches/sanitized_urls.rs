//! URL sanitization costs, with parsing outside the measured region.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;
use ox_content_renderer::{HtmlRenderer, HtmlRendererOptions};

fn bench_sanitized_urls(c: &mut Criterion) {
    let mut group = c.benchmark_group("sanitized_urls");
    for (name, url) in [
        ("https", "https://example.com/api/reference?lang=ja#settings"),
        ("mixed_case", "hTtPs://example.com/api"),
        ("relative", "./guide/api.md#settings"),
        ("unsafe", "javascript:alert(1)"),
    ] {
        let source = format!("[API reference]({url}) ![Preview]({url})\n\n").repeat(256);
        let allocator = Allocator::for_source_len(source.len());
        let document = Parser::new(&allocator, &source).parse().unwrap();
        for sanitize in [false, true] {
            group.bench_function(format!("{name}/enabled_{sanitize}"), |b| {
                b.iter(|| {
                    let mut renderer = HtmlRenderer::with_options(HtmlRendererOptions {
                        sanitize,
                        ..Default::default()
                    });
                    black_box(renderer.render(black_box(&document)));
                });
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_sanitized_urls);
criterion_main!(benches);
