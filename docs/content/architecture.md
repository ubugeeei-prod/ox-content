# Architecture

This document provides a deep dive into the architecture and design decisions of Ox Content.

## Overview

Ox Content is designed as a modular, high-performance Markdown processing toolkit. The architecture follows the Oxc philosophy of prioritizing speed, memory efficiency, and correctness.

```mermaid
graph TB
    subgraph UserApps["User Applications"]
        App[Your App]
        DocSite[Documentation Site]
        Blog[Blog]
    end

    subgraph JSPackages["JavaScript Packages"]
        VitePlugin["@ox-content/vite-plugin"]
        ViteVue["@ox-content/vite-plugin-vue"]
        ViteReact["@ox-content/vite-plugin-react"]
    end

    subgraph NAPI["Node.js Bindings"]
        NAPIPackage["@ox-content/napi"]
    end

    subgraph RustCore["Rust Core"]
        Renderer[ox_content_renderer]
        Parser[ox_content_parser]
        AST[ox_content_ast]
        Allocator[ox_content_allocator]
    end

    UserApps --> JSPackages
    JSPackages --> NAPI
    NAPI --> RustCore
    Renderer --> AST
    Parser --> AST
    AST --> Allocator
```

## Crate Structure

```
ox-content/
├── crates/
│   ├── ox_content_allocator/   # Foundation: Arena allocator
│   ├── ox_content_ast/         # Core: AST definitions
│   ├── ox_content_parser/      # Core: Markdown parser
│   ├── ox_content_renderer/    # Core: HTML renderer
│   ├── ox_content_search/      # Core: Full-text search engine
│   ├── ox_content_ssg/         # Core: Static site generation
│   ├── ox_content_napi/        # Bindings: Node.js via napi-rs
│   ├── ox_content_wasm/        # Bindings: WebAssembly
│   ├── ox_content_vite/        # Integration: Vite plugin
│   ├── ox_content_og_image/    # Feature: OG image generation
│   └── ox_content_docs/        # Feature: Source code documentation
├── playground/                 # Interactive playground
└── docs/                       # Documentation (self-hosted)
```

## Memory Management

### Arena Allocation with bumpalo

Ox Content uses [bumpalo](https://docs.rs/bumpalo) for arena-based allocation. This is the key to our performance advantage.

#### How Arena Allocation Works

```mermaid
graph LR
    subgraph Traditional["Traditional Allocation"]
        A1[A] --> H1[Heap]
        B1[B] --> H2[Heap]
        C1[C] --> H3[Heap]
    end

    subgraph Arena["Arena Allocation"]
        A2[A] --> CM
        B2[B] --> CM
        C2[C] --> CM[Contiguous Memory]
    end
```

**Traditional**: 4 separate heap allocations, 4 separate deallocations

**Arena**: 1 contiguous region, 1 deallocation (drop arena)

#### Benefits

1. **Fast Allocation** - Just bump a pointer, no free list traversal
2. **Zero-Copy Parsing** - AST nodes can reference source slices directly
3. **Efficient Deallocation** - Drop the entire arena at once
4. **Cache-Friendly** - Related data stored contiguously in memory
5. **No Fragmentation** - Memory is allocated linearly

#### Implementation

```rust
// ox_content_allocator/src/lib.rs

use bumpalo::Bump;

/// Arena allocator for AST nodes.
pub struct Allocator {
    bump: Bump,
}

impl Allocator {
    /// Creates a new allocator with default capacity.
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Allocates a value in the arena.
    pub fn alloc<T>(&self, value: T) -> &mut T {
        self.bump.alloc(value)
    }

    /// Allocates a string slice in the arena.
    pub fn alloc_str(&self, s: &str) -> &str {
        self.bump.alloc_str(s)
    }

    /// Creates a new Vec in the arena.
    pub fn new_vec<T>(&self) -> Vec<'_, T> {
        Vec::new_in(&self.bump)
    }
}

// Re-export arena-aware types with standard names
pub type Box<'a, T> = bumpalo::boxed::Box<'a, T>;
pub type Vec<'a, T> = bumpalo::collections::Vec<'a, T>;
pub type String<'a> = bumpalo::collections::String<'a>;
```

#### Usage Pattern

```rust
fn process_markdown(source: &str) -> String {
    // Create arena - all allocations happen here
    let allocator = Allocator::new();

    // Parse document - AST allocated in arena
    let parser = Parser::new(&allocator, source);
    let document = parser.parse().unwrap();

    // Render to HTML - output is owned String
    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&document);

    html
    // allocator dropped here - all AST memory freed at once
}
```

## AST Design

### mdast Specification

The AST follows the [mdast](https://github.com/syntax-tree/mdast) specification, which is part of the [unified](https://unifiedjs.com/) ecosystem. This ensures compatibility with existing tools and plugins.

### Node Hierarchy

```
Document (root)
├── Block Nodes
│   ├── Paragraph
│   │   └── Inline Nodes...
│   ├── Heading (depth: 1-6)
│   │   └── Inline Nodes...
│   ├── CodeBlock (lang, meta, value)
│   ├── BlockQuote
│   │   └── Block Nodes...
│   ├── List (ordered, start, spread)
│   │   └── ListItem (checked)
│   │       └── Block Nodes...
│   ├── Table
│   │   └── TableRow
│   │       └── TableCell
│   │           └── Inline Nodes...
│   ├── ThematicBreak
│   └── Html (raw)
│
└── Inline Nodes
    ├── Text (value)
    ├── Emphasis
    │   └── Inline Nodes...
    ├── Strong
    │   └── Inline Nodes...
    ├── InlineCode (value)
    ├── Link (url, title)
    │   └── Inline Nodes...
    ├── Image (url, alt, title)
    ├── Break
    ├── Delete (GFM)
    │   └── Inline Nodes...
    └── FootnoteReference (identifier)
```

### Span Information

Every node includes source location information:

```rust
/// Source span (byte offsets).
#[derive(Debug, Clone, Copy)]
pub struct Span {
    /// Start byte offset (inclusive).
    pub start: u32,
    /// End byte offset (exclusive).
    pub end: u32,
}

impl Span {
    pub fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}
```

This enables:

- Error messages with precise source locations
- Source maps for debugging
- Syntax highlighting in editors
- Incremental re-parsing

### Visitor Pattern

The AST can be traversed using the visitor pattern:

```rust
/// Trait for visiting AST nodes.
pub trait Visit<'a> {
    fn visit_document(&mut self, document: &Document<'a>) {
        for node in &document.children {
            self.visit_node(node);
        }
    }

    fn visit_node(&mut self, node: &Node<'a>) {
        match node {
            Node::Paragraph(n) => self.visit_paragraph(n),
            Node::Heading(n) => self.visit_heading(n),
            Node::CodeBlock(n) => self.visit_code_block(n),
            // ... other variants
        }
    }

    fn visit_paragraph(&mut self, paragraph: &Paragraph<'a>) {
        for child in &paragraph.children {
            self.visit_node(child);
        }
    }

    fn visit_heading(&mut self, heading: &Heading<'a>) {
        for child in &heading.children {
            self.visit_node(child);
        }
    }

    // ... other visit methods with default implementations
}
```

#### Example: Table of Contents Generator

```rust
use ox_content_ast::{Visit, Document, Heading, Node, Text};

struct TocGenerator {
    entries: Vec<TocEntry>,
}

struct TocEntry {
    depth: u8,
    text: String,
    id: String,
}

impl<'a> Visit<'a> for TocGenerator {
    fn visit_heading(&mut self, heading: &Heading<'a>) {
        let mut text = String::new();
        for child in &heading.children {
            if let Node::Text(t) = child {
                text.push_str(t.value);
            }
        }

        let id = slugify(&text);
        self.entries.push(TocEntry {
            depth: heading.depth,
            text,
            id,
        });
    }
}

fn generate_toc(document: &Document<'_>) -> Vec<TocEntry> {
    let mut generator = TocGenerator { entries: vec![] };
    generator.visit_document(document);
    generator.entries
}
```

## Parser Design

### Architecture

```mermaid
graph TB
    Source["Source Text<br/>(Markdown)"]
    Lexer["Lexer<br/>Tokenizes input (logos crate)"]
    Parser["Parser<br/>Builds AST from tokens"]
    AST["AST<br/>Arena-allocated nodes"]

    Source --> Lexer
    Lexer --> Parser
    Parser --> AST
```

### Parser Options

```rust
/// Parser configuration options.
#[derive(Debug, Clone, Default)]
pub struct ParserOptions {
    /// Enable GFM (GitHub Flavored Markdown) extensions.
    pub gfm: bool,
    /// Enable footnotes.
    pub footnotes: bool,
    /// Enable task lists.
    pub task_lists: bool,
    /// Enable tables.
    pub tables: bool,
    /// Enable strikethrough.
    pub strikethrough: bool,
    /// Enable autolinks.
    pub autolinks: bool,
    /// Maximum nesting depth for block elements.
    pub max_nesting_depth: usize,
}

impl ParserOptions {
    /// Creates options with all GFM extensions enabled.
    pub fn gfm() -> Self {
        Self {
            gfm: true,
            footnotes: true,
            task_lists: true,
            tables: true,
            strikethrough: true,
            autolinks: true,
            max_nesting_depth: 100,
        }
    }
}
```

### Parsing Strategy

1. **Block-First** - Parse block structure first (paragraphs, headings, etc.)
2. **Inline Later** - Parse inline content within blocks
3. **Lazy Evaluation** - Only parse what's needed
4. **Error Recovery** - Continue parsing after errors when possible

### CommonMark Compliance

The parser follows the [CommonMark spec](https://spec.commonmark.org/):

- ATX headings (`# Heading`)
- Setext headings (underlined)
- Fenced code blocks (``` or ~~~)
- Indented code blocks
- Block quotes (`>`)
- Lists (ordered and unordered)
- Thematic breaks (`---`, `***`, `___`)
- Emphasis and strong emphasis
- Links and images
- Hard and soft line breaks

## Renderer Design

### HTML Renderer

```rust
/// HTML renderer with customizable options.
pub struct HtmlRenderer {
    options: HtmlRendererOptions,
    output: String,
}

/// Renderer configuration.
#[derive(Debug, Clone)]
pub struct HtmlRendererOptions {
    /// Use XHTML-style self-closing tags.
    pub xhtml: bool,
    /// Soft break string.
    pub soft_break: String,
    /// Hard break string.
    pub hard_break: String,
    /// Enable syntax highlighting.
    pub highlight: bool,
    /// Sanitize HTML output.
    pub sanitize: bool,
}
```

### Renderer Trait

Custom renderers can be implemented:

```rust
/// Trait for rendering AST to output format.
pub trait Renderer {
    type Output;

    fn render(&mut self, document: &Document<'_>) -> RenderResult<Self::Output>;
}
```

### HTML Escaping

The renderer properly escapes HTML entities:

| Character | Entity   |
| --------- | -------- |
| `&`       | `&amp;`  |
| `<`       | `&lt;`   |
| `>`       | `&gt;`   |
| `"`       | `&quot;` |
| `'`       | `&#39;`  |

URL encoding is also handled for link/image URLs.

## NAPI Bindings

### Architecture

```mermaid
graph TB
    JS["JavaScript / TypeScript"]
    NPM["@ox-content/napi<br/>TypeScript types + JS wrapper"]
    NAPI["ox_content_napi<br/>Rust NAPI binding layer"]
    Core["ox_content_*<br/>Core Rust crates"]

    JS --> NPM
    NPM --> NAPI
    NAPI --> Core
```

### Data Transfer

- AST is serialized to JSON for JavaScript interop
- HTML rendering happens in Rust for maximum performance
- Async support for large documents

### Thread Safety

The NAPI bindings are designed to be thread-safe:

- Each parse operation creates its own allocator
- No shared mutable state between calls

## Vite Integration

### Environment API

Ox Content integrates with Vite's Environment API for SSG:

```typescript
// Creates a server-side environment for Markdown processing
const mdEnv = new Environment("markdown", {
  // Custom module resolution for .md files
  resolve: {
    extensions: [".md"],
  },
  // Transform .md to JS modules
  transform: async (code, id) => {
    if (id.endsWith(".md")) {
      const result = await parseAndRender(code);
      return `export default ${JSON.stringify(result)}`;
    }
  },
});
```

### Hot Module Replacement

The Vite plugin supports HMR for Markdown files:

1. File change detected
2. Re-parse changed file
3. Send update to client
4. Update rendered content without full reload

## Performance Characteristics

### Memory Usage

| Content Size | Traditional Parser | Ox Content    |
| ------------ | ------------------ | ------------- |
| 1 KB         | ~50 KB heap        | ~8 KB arena   |
| 10 KB        | ~500 KB heap       | ~80 KB arena  |
| 100 KB       | ~5 MB heap         | ~800 KB arena |

### Parse and Render Speed

Latest local benchmark sweep on 2026-04-24 with Node `v24.15.0` on Apple M5 Pro. The tables below show median results from 7 local runs of the benchmark harness for the large 48.7 KB case.

| Library             | Parse Only (48.7 KB) | Parse + Render (48.7 KB) |
| ------------------- | -------------------: | -----------------------: |
| `Bun.markdown.html` |                    - |             3376 ops/sec |
| `md4x (napi)`       |          958 ops/sec |             3167 ops/sec |
| `@ox-content/napi`  |         2337 ops/sec |             2599 ops/sec |
| `md4w (md4c)`       |          884 ops/sec |             2253 ops/sec |
| `markdown-it`       |          631 ops/sec |              628 ops/sec |
| `marked`            |          385 ops/sec |              381 ops/sec |
| `micromark`         |                    - |               36 ops/sec |
| `remark`            |           33 ops/sec |               29 ops/sec |

This benchmark uses `node benchmarks/bundle-size/parse-benchmark.mjs`. The comparison set now includes `md4w (md4c)` and `md4x (napi)` by default, and `Bun.markdown.html` is measured automatically when `bun` is available on the host.

## Security Considerations

### HTML Sanitization

When rendering untrusted Markdown, enable sanitization:

```rust
let options = HtmlRendererOptions {
    sanitize: true,
    ..Default::default()
};
let mut renderer = HtmlRenderer::with_options(options);
```

### Nesting Limits

The parser enforces maximum nesting depth to prevent stack overflow:

```rust
let options = ParserOptions {
    max_nesting_depth: 50,  // Limit nesting
    ..Default::default()
};
```

### Input Validation

- Maximum input size limits
- Invalid UTF-8 handling
- Malformed Markdown graceful handling

## Future Directions

- **Incremental Parsing** - Re-parse only changed portions
- **Streaming Parser** - Parse large documents in chunks
- **WASM Build** - Run in browsers without NAPI
- **Custom Syntax Extensions** - Plugin system for custom blocks
- **Source Maps** - Full source map generation
