use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ox_content_allocator::Allocator;
use ox_content_napi::{
    parse, parse_mdast_raw, transform, transform_mdast_raw, JsParserOptions, JsTransformOptions,
};
use ox_content_parser::{Parser, ParserOptions};

const SAMPLE_BLOCK: &str = r"# Heading

This is a paragraph with **bold**, *italic*, ~~strike~~, `inline code`, and a [link](https://example.com).

## List

- [x] Task item
- [ ] Another task item
- Plain item with emoji 😀 and Japanese text こんにちは

## Table

| Name | Value | Notes |
| ---- | ----- | ----- |
| alpha | 1 | first |
| beta | 2 | second |

> Blockquote with a second line.

```ts
export function sum(a: number, b: number): number {
  return a + b;
}
```
";

fn bench_napi_mdast_transfer(c: &mut Criterion) {
    let documents = [
        ("small", SAMPLE_BLOCK.to_string()),
        ("medium", repeated_document(10)),
        ("large", repeated_document(100)),
    ];

    let parser_options = Some(gfm_parser_options());
    let transform_options = Some(gfm_transform_options());

    let mut group = c.benchmark_group("napi_mdast_transfer");

    for (name, document) in &documents {
        report_payload_sizes(name, document, parser_options.clone());
        group.throughput(Throughput::Bytes(document.len() as u64));

        group.bench_with_input(BenchmarkId::new("parse_native", name), document, |b, document| {
            let native_options = ParserOptions::from(gfm_parser_options());
            b.iter(|| {
                let allocator = Allocator::new();
                let parser = Parser::with_options(
                    &allocator,
                    black_box(document.as_str()),
                    native_options.clone(),
                );
                let result = parser.parse().expect("native parser should succeed");
                black_box(result.children.len());
            });
        });

        group.bench_with_input(BenchmarkId::new("parse_json", name), document, |b, document| {
            let parser_options = parser_options.clone();
            b.iter(|| {
                let result = parse(black_box(document.clone()), parser_options.clone());
                black_box(result.ast.len());
            });
        });

        group.bench_with_input(BenchmarkId::new("parse_raw", name), document, |b, document| {
            let parser_options = parser_options.clone();
            b.iter(|| {
                let result = parse_mdast_raw(black_box(document.clone()), parser_options.clone())
                    .expect("raw mdast parsing should succeed");
                black_box(result.len());
            });
        });

        group.bench_with_input(
            BenchmarkId::new("transform_mdast_raw", name),
            document,
            |b, document| {
                let transform_options = transform_options.clone();
                b.iter(|| {
                    let result =
                        transform_mdast_raw(black_box(document.clone()), transform_options.clone())
                            .expect("raw mdast transform should succeed");
                    black_box(result.len());
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("transform_html", name),
            document,
            |b, document| {
                let transform_options = transform_options.clone();
                b.iter(|| {
                    let result = transform(black_box(document.clone()), transform_options.clone());
                    black_box(result.html.len());
                });
            },
        );
    }

    group.finish();
}

fn repeated_document(times: usize) -> String {
    std::iter::repeat_n(SAMPLE_BLOCK, times).collect::<Vec<_>>().join("\n\n")
}

fn gfm_parser_options() -> JsParserOptions {
    JsParserOptions {
        gfm: Some(true),
        footnotes: Some(true),
        task_lists: Some(true),
        tables: Some(true),
        strikethrough: Some(true),
        autolinks: Some(true),
    }
}

fn gfm_transform_options() -> JsTransformOptions {
    JsTransformOptions {
        gfm: Some(true),
        footnotes: Some(true),
        task_lists: Some(true),
        tables: Some(true),
        strikethrough: Some(true),
        autolinks: Some(true),
        frontmatter: Some(true),
        toc_max_depth: Some(3),
        convert_md_links: Some(false),
        ..Default::default()
    }
}

#[allow(clippy::cast_precision_loss, clippy::print_stdout)]
fn report_payload_sizes(name: &str, document: &str, parser_options: Option<JsParserOptions>) {
    let json = parse(document.to_string(), parser_options.clone());
    let raw = parse_mdast_raw(document.to_string(), parser_options)
        .expect("raw mdast parsing should succeed for payload size reporting");
    let transformed = transform_mdast_raw(document.to_string(), Some(gfm_transform_options()))
        .expect("raw mdast transform should succeed for payload size reporting");
    let json_len = json.ast.len();
    let raw_len = raw.len();
    let transformed_len = transformed.len();
    let ratio = if json_len == 0 { 0.0 } else { (raw_len as f64 / json_len as f64) * 100.0 };
    let transformed_ratio =
        if json_len == 0 { 0.0 } else { (transformed_len as f64 / json_len as f64) * 100.0 };

    println!(
        "napi_mdast_transfer/{name}: input={}B json={}B raw={}B transform_raw={}B raw/json={ratio:.2}% transform_raw/json={transformed_ratio:.2}%",
        document.len(),
        json_len,
        raw_len,
        transformed_len,
    );
}

criterion_group!(benches, bench_napi_mdast_transfer);
criterion_main!(benches);
