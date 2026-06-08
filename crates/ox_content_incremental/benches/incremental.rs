use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};
use ox_content_incremental::{
    stable_prefix_len, IncrementalHtmlRenderer, IncrementalRenderOptions,
};
use ox_content_parser::ParserOptions;

const SAMPLE: &str = r"# Incremental renderer

This benchmark keeps the document shape close to chat and documentation previews.
It includes headings, paragraphs, lists, tables, and code fences so the stable
prefix scanner and fragment renderer both do meaningful work.

## Streaming behavior

Markdown often arrives in awkward boundaries. A heading can arrive as `# Incre`,
emphasis can arrive as `**strong`, and code fences may stay open for several
chunks. The incremental renderer keeps those cases readable by rendering the
pending tail with temporary inline completion.

- [x] Partial headings render as headings.
- [x] Partial emphasis can render as emphasized text.
- [x] Stable HTML does not need to be touched again.
- [ ] The UI can decide whether to use deltas or the full snapshot.

| Field | Meaning |
| --- | --- |
| `committedBytes` | Bytes that will not be re-rendered |
| `pendingBytes` | Bytes still held as the unstable tail |
| `didCommit` | Whether this append produced stable HTML |

```ts
const renderer = new IncrementalMarkdownRenderer({ gfm: true });
const result = renderer.append(chunk, {
  renderPending: true,
  completeInline: true,
});
```

Final paragraph with enough prose to keep the benchmark from being dominated by
tiny input overhead. The stream model should stay cheap even when the UI asks for
provisional pending HTML on every append.
";

fn repeated_sample() -> String {
    let mut source = String::with_capacity(SAMPLE.len() * 16);
    for _ in 0..16 {
        source.push_str(SAMPLE);
        source.push('\n');
    }
    source
}

fn chunks(source: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let sizes = [12usize, 18, 24, 32, 45, 64, 28, 52, 80, 36, 72];
    let mut cursor = 0usize;
    let mut size_index = 0usize;
    while cursor < source.len() {
        let mut end = cursor.saturating_add(sizes[size_index % sizes.len()]).min(source.len());
        while !source.is_char_boundary(end) {
            end -= 1;
        }
        out.push(&source[cursor..end]);
        cursor = end;
        size_index += 1;
    }
    out
}

fn bench_stable_prefix_scan(c: &mut Criterion) {
    let source = repeated_sample();
    c.bench_function("stable_prefix_len/large_pending_buffer", |b| {
        b.iter(|| {
            black_box(stable_prefix_len(black_box(&source)));
        });
    });
}

fn bench_incremental_render(c: &mut Criterion) {
    let source = repeated_sample();
    let chunks = chunks(&source);
    c.bench_function("incremental_render/characteristic_chunks", |b| {
        b.iter(|| {
            let mut renderer = IncrementalHtmlRenderer::new(ParserOptions::gfm());
            for chunk in &chunks {
                let result = renderer
                    .append(
                        black_box(chunk),
                        IncrementalRenderOptions {
                            is_final: false,
                            render_pending: true,
                            complete_inline: true,
                        },
                    )
                    .expect("incremental render should succeed");
                black_box(result);
            }
            black_box(renderer.finish().expect("finish should succeed"));
        });
    });
}

criterion_group!(benches, bench_stable_prefix_scan, bench_incremental_render);
criterion_main!(benches);
