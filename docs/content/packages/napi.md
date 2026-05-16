# @ox-content/napi

Node.js bindings for Ox Content's Rust core.

## Installation

```bash
vp install @ox-content/napi
```

## Usage

### Parse Markdown to AST

```ts
import { parseMarkdown } from "@ox-content/napi";

const markdown = "# Hello World\n\nThis is **bold** text.";
const ast = parseMarkdown(markdown, { gfm: true });

console.log(JSON.stringify(ast, null, 2));
```

### Parse and Render

```ts
import { parseAndRender } from "@ox-content/napi";

const markdown = `
# Welcome

- Item 1
- Item 2
- Item 3

| Column A | Column B |
|----------|----------|
| Value 1  | Value 2  |
`;

const result = parseAndRender(markdown, {
  gfm: true,
  footnotes: true,
  tables: true,
});

console.log(result.html);
```

## API

### parseMarkdown(content, options?)

Parses Markdown content and returns the AST.

#### Parameters

- `content`: `string` - Markdown content to parse
- `options`: `ParseOptions` (optional)

#### Returns

`MarkdownAst` - The parsed AST

### parseAndRender(content, options?)

Parses and renders Markdown to HTML in a single call.

#### Parameters

- `content`: `string` - Markdown content to parse
- `options`: `ParseOptions` (optional)

#### Returns

```ts
interface RenderResult {
  html: string;
  frontmatter?: Record<string, unknown>;
  toc?: TocEntry[];
}
```

## Options

```ts
interface ParseOptions {
  /** Enable GitHub Flavored Markdown */
  gfm?: boolean;

  /** Enable footnotes */
  footnotes?: boolean;

  /** Enable tables */
  tables?: boolean;

  /** Enable task lists */
  taskLists?: boolean;

  /** Enable strikethrough */
  strikethrough?: boolean;
}
```

## AST Types

The AST follows the [mdast](https://github.com/syntax-tree/mdast) specification:

```ts
interface MarkdownNode {
  type: string;
  children?: MarkdownNode[];
  value?: string;
  // Additional properties based on node type
}

// Block nodes
type BlockNode =
  | "root"
  | "paragraph"
  | "heading"
  | "codeBlock"
  | "blockquote"
  | "list"
  | "listItem"
  | "table"
  | "tableRow"
  | "tableCell"
  | "thematicBreak"
  | "html";

// Inline nodes
type InlineNode =
  | "text"
  | "emphasis"
  | "strong"
  | "inlineCode"
  | "link"
  | "image"
  | "break"
  | "delete"
  | "footnoteReference";
```

## Search API

The NAPI bindings include a full-text search engine.

### buildSearchIndex(documents)

Builds a search index from an array of documents.

```ts
import { buildSearchIndex } from "@ox-content/napi";

const documents = [
  {
    id: "getting-started",
    title: "Getting Started",
    url: "/getting-started",
    body: "Welcome to the documentation...",
    headings: ["Installation", "Quick Start"],
    code: ["npm install package"],
  },
];

const indexJson = buildSearchIndex(documents);
```

### searchIndex(indexJson, query, options?)

Searches a serialized index.

```ts
import { searchIndex } from "@ox-content/napi";

const results = searchIndex(indexJson, "getting started", {
  limit: 10,
  prefix: true,
});

// results: Array<{
//   id: string;
//   title: string;
//   url: string;
//   score: number;
//   matches: string[];
//   snippet: string;
// }>
```

### extractSearchContent(source, id, url, options?)

Extracts searchable content from Markdown source.

```ts
import { extractSearchContent } from "@ox-content/napi";

const markdown = "# Hello World\n\nThis is content.";
const doc = extractSearchContent(markdown, "hello", "/hello", { gfm: true });

// doc: {
//   id: 'hello',
//   title: 'Hello World',
//   url: '/hello',
//   body: 'This is content.',
//   headings: ['Hello World'],
//   code: [],
// }
```

## Performance

Ox Content is positioned both as a document generator and as a high-performance Markdown toolkit. The numbers below focus on the Markdown engine side.

Latest local benchmark sweep on 2026-04-24 with Node `v24.15.0` on Apple M5 Pro. The tables below show median results from 7 local runs of the benchmark harness for the large 48.7 KB case.

### Parse Only (48.7 KB)

| Library            | ops/sec | avg time |  throughput |
| ------------------ | ------: | -------: | ----------: |
| `@ox-content/napi` |    2337 |  0.43 ms | 111.20 MB/s |
| `md4x (napi)`      |     958 |  1.04 ms |  45.56 MB/s |
| `md4w (md4c)`      |     884 |  1.13 ms |  42.06 MB/s |
| `markdown-it`      |     631 |  1.58 ms |  30.04 MB/s |
| `marked`           |     385 |  2.60 ms |  18.33 MB/s |
| `remark`           |      33 | 29.97 ms |   1.59 MB/s |

### Parse + Render (48.7 KB)

| Library             | ops/sec | avg time |  throughput |
| ------------------- | ------: | -------: | ----------: |
| `Bun.markdown.html` |    3376 |  0.30 ms | 160.67 MB/s |
| `md4x (napi)`       |    3167 |  0.32 ms | 150.73 MB/s |
| `@ox-content/napi`  |    2599 |  0.38 ms | 123.66 MB/s |
| `md4w (md4c)`       |    2253 |  0.44 ms | 107.21 MB/s |
| `markdown-it`       |     628 |  1.59 ms |  29.91 MB/s |
| `marked`            |     381 |  2.62 ms |  18.15 MB/s |
| `micromark`         |      36 | 28.16 ms |   1.69 MB/s |
| `remark`            |      29 | 34.97 ms |   1.36 MB/s |

Reproduce with:

```bash
node benchmarks/bundle-size/parse-benchmark.mjs
```

In this latest local release-build sweep, Ox Content stays ahead for parse-only throughput. The parse+render comparison now includes `md4x (napi)`, where Bun and md4x lead the table and Ox Content remains close behind while still serving as the native core for the full documentation pipeline.

The benchmark includes `md4w (md4c)` and `md4x (napi)` by default and adds `Bun.markdown.html` automatically when `bun` is available.

### mdast Transfer Micro-benchmark

To benchmark the mdast export paths used by the unified bridge, run:

```bash
cargo bench -p ox_content_napi --bench mdast_transfer -- --sample-size 20 --warm-up-time 1 --measurement-time 2
```

This Criterion benchmark compares `parse_native`, `parse_json`, `parse_raw`, and `transform_html`
across small, medium, and large GFM documents. It also prints the exported JSON and raw payload sizes
for each fixture so you can distinguish parser cost from transfer-format cost.

This benchmark measures the Rust-side pipeline only. For end-to-end unified bridge evaluation, pair it
with a JavaScript benchmark that includes the N-API boundary and JS-side mdast materialization.

### Transfer Envelope

Raw transfers now use a payload-kind-aware envelope via `parseTransferRaw(source, kind, options)`.
`mdast` is the baseline payload and remains the highest-priority path, but the envelope is designed so
future payloads such as markdown-it token streams can reuse the same zero-copy memory block shape
instead of introducing a second ad-hoc binary format.

The native unified bridge now also uses `transformMdastRaw(source, options)` so Rust can parse
frontmatter, strip content, and serialize mdast into one external `Uint8Array` before JavaScript
deserializes it. For markdown-it and custom parser interop paths, `prepareSourceRaw(source, {frontmatter})`
provides a lighter `prepared-source` envelope that carries only stripped content plus frontmatter JSON,
so source preparation stays in Rust instead of falling back to JavaScript preprocessing.

Both envelopes now also carry a compact `source origin` section when frontmatter is stripped. JavaScript
uses that metadata to rebase mdast `position` fields and expose `sourceOffset` on `file.data`,
`file.data.oxContent`, and the Ox Content mdast plugin context, so unified diagnostics and downstream
plugin messages stay aligned with the original full source file instead of the post-frontmatter content
slice.

`parseMdastRaw(source, options)` is kept as the mdast-specific compatibility wrapper.
