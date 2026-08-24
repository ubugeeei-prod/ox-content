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

The current parser and renderer benchmark snapshot lives on
[Performance](../performance.md). This package page keeps only N-API-specific
micro-benchmarking notes.

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
