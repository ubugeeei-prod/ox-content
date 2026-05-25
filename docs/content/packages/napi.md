# @ox-content/napi

Node.js bindings for Ox Content's Rust core.

## Installation

```bash
vp install @ox-content/napi
```

## Platform Support

Release packages ship native bindings for macOS arm64/x64, Linux arm64/x64 GNU, and Windows x64 MSVC. CI runs a lightweight load and parse/render smoke test on macOS, Linux, and Windows for every PR.

Other Node.js platforms may build from source if the Rust toolchain and NAPI build tooling are available, but they are not published as prebuilt npm binding packages.

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

Latest local benchmark sweep on 2026-05-25 with Node `v24.15.0` on Apple M5 Pro. The tables below show median results from 7 local runs of the benchmark harness for the large 48.7 KB case.

### Parse Only (48.7 KB)

| Library            | ops/sec | avg time |  throughput |
| ------------------ | ------: | -------: | ----------: |
| `@ox-content/napi` |    4207 |  0.24 ms | 200.20 MB/s |
| `md4x (napi)`      |    1231 |  0.81 ms |  58.56 MB/s |
| `md4w (md4c)`      |    1143 |  0.87 ms |  54.41 MB/s |
| `markdown-it`      |    1035 |  0.97 ms |  49.24 MB/s |
| `marked`           |     530 |  1.89 ms |  25.23 MB/s |
| `remark`           |      44 | 22.74 ms |   2.09 MB/s |

### Parse + Render (48.7 KB)

| Library             | ops/sec | avg time |  throughput |
| ------------------- | ------: | -------: | ----------: |
| `@ox-content/napi`  |    4503 |  0.22 ms | 214.26 MB/s |
| `Bun.markdown.html` |    4225 |  0.24 ms | 201.06 MB/s |
| `md4x (napi)`       |    4014 |  0.25 ms | 191.02 MB/s |
| `md4w (md4c)`       |    2653 |  0.38 ms | 126.23 MB/s |
| `markdown-it`       |     840 |  1.19 ms |  39.96 MB/s |
| `marked`            |     470 |  2.13 ms |  22.36 MB/s |
| `micromark`         |      45 | 22.35 ms |   2.13 MB/s |
| `remark`            |      36 | 28.16 ms |   1.69 MB/s |

Reproduce with:

```bash
node benchmarks/bundle-size/parse-benchmark.mjs
```

In this latest local release-build sweep, Ox Content leads every comparison: 3.4× ahead of the next-fastest native parser (`md4x (napi)`) on parse-only and 1.07× ahead of `Bun.markdown.html` on parse+render, while remaining the native core that drives the full documentation pipeline. Margins widen further on small documents — see `node benchmarks/bundle-size/parse-benchmark.mjs` for the full sweep across small, medium, and large inputs.

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

A local transfer-focused run on 2026-05-17 with Node `v24.15.0` on Apple M5 Pro used `--sample-size 10`,
`--warm-up-time 1`, and `--measurement-time 1`. The large fixture was 45,298 bytes of GFM-heavy Markdown.

| Path                  | Large fixture median |   Throughput |
| --------------------- | -------------------: | -----------: |
| `parse_native`        |            314.07 us | 137.55 MiB/s |
| `parse_json`          |            373.60 us | 115.63 MiB/s |
| `parse_raw`           |            560.24 us | 77.109 MiB/s |
| `transform_mdast_raw` |            594.15 us | 72.708 MiB/s |
| `transform_html`      |            686.09 us | 62.965 MiB/s |

Payload sizes from the same run:

| Fixture | JSON bytes | Raw bytes | Transform raw bytes |
| ------- | ---------: | --------: | ------------------: |
| small   |      2,292 |     4,177 |               4,682 |
| medium  |     22,668 |    40,582 |              45,164 |
| large   |    226,428 |   404,632 |             449,984 |

The raw transfer path is still useful because it keeps Rust in charge of parsing, frontmatter stripping,
and source-origin metadata, but this run shows that the first raw encoding is not yet smaller or faster
than JSON by itself. End-to-end mdast bridge performance should therefore be read as a compatibility
feature today, with raw format and JS deserializer tuning as the next performance target.

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
