//! Benchmarks for the Markdown parser.

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use ox_content_allocator::Allocator;
use ox_content_parser::Parser;
use std::hint::black_box;

const SIMPLE_MD: &str = r#"# Hello World

This is a paragraph with some **bold** and *italic* text.

## Second heading

- Item 1
- Item 2
- Item 3

```rust
fn main() {
    println!("Hello!");
}
```
"#;

const LARGE_MD: &str = r#"# Large Document Benchmark

This is a comprehensive markdown document used for benchmarking the parser performance.

## Introduction

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.

### Features

- **Bold text** for emphasis
- *Italic text* for subtle emphasis
- ~~Strikethrough~~ for removed content
- `inline code` for technical terms

## Code Examples

Here's a Rust example:

```rust
fn main() {
    let message = "Hello, World!";
    println!("{}", message);

    for i in 0..10 {
        println!("Count: {}", i);
    }
}
```

And a TypeScript example:

```typescript
interface User {
  id: number;
  name: string;
  email: string;
}

async function fetchUser(id: number): Promise<User> {
  const response = await fetch(`/api/users/${id}`);
  return response.json();
}
```

## Tables

| Feature | Status | Notes |
|---------|--------|-------|
| Parsing | Done | Fast and efficient |
| Rendering | Done | HTML output |
| GFM | Done | GitHub Flavored Markdown |
| Frontmatter | Done | YAML support |

## Task Lists

- [x] Implement parser
- [x] Add GFM support
- [x] Syntax highlighting
- [ ] Mermaid diagrams
- [ ] Math equations

## Blockquotes

> This is a blockquote.
> It can span multiple lines.
>
> And have multiple paragraphs.

## Links and Images

Check out [our documentation](https://example.com/docs) for more information.

![Example Image](https://example.com/image.png)

## Nested Lists

1. First item
   - Nested bullet
   - Another nested bullet
2. Second item
   1. Nested number
   2. Another nested number
3. Third item

## Horizontal Rules

---

## More Content

This section contains additional content to make the document larger for benchmarking purposes.

### Subsection A

Pellentesque habitant morbi tristique senectus et netus et malesuada fames ac turpis egestas. Vestibulum tortor quam, feugiat vitae, ultricies eget, tempor sit amet, ante.

### Subsection B

Donec eu libero sit amet quam egestas semper. Aenean ultricies mi vitae est. Mauris placerat eleifend leo. Quisque sit amet est et sapien ullamcorper pharetra.

### Subsection C

Vestibulum erat wisi, condimentum sed, commodo vitae, ornare sit amet, wisi. Aenean fermentum, elit eget tincidunt condimentum, eros ipsum rutrum orci, sagittis tempus lacus enim ac dui.

## Conclusion

This document serves as a comprehensive test case for the markdown parser, covering various markdown features and edge cases.
"#;

const LONG_LIST: &str = r"- item 01
- item 02
- item 03
- item 04
- item 05
- item 06
- item 07
- item 08
- item 09
- item 10
- item 11
- item 12
- item 13
- item 14
- item 15
- item 16
- item 17
- item 18
- item 19
- item 20
- item 21
- item 22
- item 23
- item 24
- item 25
- item 26
- item 27
- item 28
- item 29
- item 30
- item 31
- item 32
";

fn bench_parse_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_simple");
    group.throughput(Throughput::Bytes(SIMPLE_MD.len() as u64));

    group.bench_function("simple_md", |b| {
        b.iter(|| {
            let allocator = Allocator::new();
            let parser = Parser::new(&allocator, black_box(SIMPLE_MD));
            let _ = parser.parse();
        });
    });

    group.finish();
}

fn bench_parse_large(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_large");
    group.throughput(Throughput::Bytes(LARGE_MD.len() as u64));

    group.bench_function("large_md", |b| {
        b.iter(|| {
            let allocator = Allocator::new();
            let parser = Parser::new(&allocator, black_box(LARGE_MD));
            let _ = parser.parse();
        });
    });

    group.finish();
}

fn bench_parse_lists(c: &mut Criterion) {
    let single_item = "- one\n";
    let mut group = c.benchmark_group("parse_lists");

    for (name, source) in [("single_item", single_item), ("long_list", LONG_LIST)] {
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

criterion_group!(benches, bench_parse_simple, bench_parse_large, bench_parse_lists);
criterion_main!(benches);
