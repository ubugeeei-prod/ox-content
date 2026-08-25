---
title: "@ox-content/napi"
description: Ox Content の Rust コア向け Node.js バインディング。
---

# @ox-content/napi

Ox Content の Rust コア向け Node.js バインディングです。

## インストール

```bash
vp install @ox-content/napi
```

## プラットフォーム対応

リリースパッケージは macOS arm64/x64、Linux arm64/x64 GNU、Windows x64 MSVC のネイティブバインディングを配ります。CI はすべての PR で、macOS、Linux、Windows に対して軽い読み込みとパース / 描画のスモークテストを走らせます。

他の Node.js プラットフォームは、Rust ツールチェーンと NAPI ビルド道具があればソースからビルドできることがありますが、事前ビルドの npm バインディングパッケージとしては公開しません。

## 使い方

### Markdown を AST へパース

```ts
import { parseMarkdown } from "@ox-content/napi";

const markdown = "# Hello World\n\nThis is **bold** text.";
const ast = parseMarkdown(markdown, { gfm: true });

console.log(JSON.stringify(ast, null, 2));
```

### パースと描画

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

Markdown をパースし、AST を返します。

#### 引数

- `content`: `string` — パースする Markdown
- `options`: `ParseOptions`（任意）

#### 戻り値

`MarkdownAst` — パース済み AST

### parseAndRender(content, options?)

1 回の呼び出しで Markdown をパースし、HTML に描画します。

#### 引数

- `content`: `string` — パースする Markdown
- `options`: `ParseOptions`（任意）

#### 戻り値

```ts
interface RenderResult {
  html: string;
  frontmatter?: Record<string, unknown>;
  toc?: TocEntry[];
}
```

## オプション

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

## AST の型

AST は [mdast](https://github.com/syntax-tree/mdast) 仕様に従います。

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

## 検索 API

NAPI バインディングには全文検索エンジンが含まれます。

### buildSearchIndex(documents)

文書配列から検索インデックスを作ります。

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

直列化したインデックスを検索します。

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

Markdown ソースから検索可能な内容を取り出します。

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

## 性能

いまのパーサとレンダラのベンチマークスナップショットは [性能](../performance.md) にあります。このパッケージページは N-API 固有のマイクロベンチマーク注記だけを残します。

### mdast 転送マイクロベンチマーク

unified ブリッジが使う mdast エクスポート経路をベンチマークするには次です。

```bash
cargo bench -p ox_content_napi --bench mdast_transfer -- --sample-size 20 --warm-up-time 1 --measurement-time 2
```

この Criterion ベンチマークは、小さい / 中 / 大きい GFM 文書に対して `parse_native`、`parse_json`、`parse_raw`、`transform_html` を比べます。各フィクスチャのエクスポート JSON と raw ペイロードサイズも出すので、パーサコストと転送形式コストを分けられます。

このベンチマークは Rust 側パイプラインだけを測ります。端から端までの unified ブリッジ評価では、N-API 境界と JS 側の mdast 実体化を含む JavaScript ベンチマークと組み合わせてください。

2026-05-17 の、転送に寄せたローカル実行は Node `v24.15.0`、Apple M5 Pro で、`--sample-size 10`、`--warm-up-time 1`、`--measurement-time 1` でした。大きいフィクスチャは 45,298 バイトの GFM 寄りの Markdown です。

| Path                  | Large fixture median |   Throughput |
| --------------------- | -------------------: | -----------: |
| `parse_native`        |            314.07 us | 137.55 MiB/s |
| `parse_json`          |            373.60 us | 115.63 MiB/s |
| `parse_raw`           |            560.24 us | 77.109 MiB/s |
| `transform_mdast_raw` |            594.15 us | 72.708 MiB/s |
| `transform_html`      |            686.09 us | 62.965 MiB/s |

同じ実行のペイロードサイズ:

| Fixture | JSON bytes | Raw bytes | Transform raw bytes |
| ------- | ---------: | --------: | ------------------: |
| small   |      2,292 |     4,177 |               4,682 |
| medium  |     22,668 |    40,582 |              45,164 |
| large   |    226,428 |   404,632 |             449,984 |

raw 転送経路はまだ役に立ちます。パース、frontmatter 除去、ソース起源メタデータを Rust が担い続けるからです。ただしこの実行では、最初の raw 符号化は JSON 単体より小さくも速くもありません。だから端から端までの mdast ブリッジ性能は、いまは互換機能として読むべきです。次の性能目標は raw 形式と JS デシリアライザの調整です。

### 転送エンベロープ

raw 転送はいま、`parseTransferRaw(source, kind, options)` 経由の、ペイロード種類を意識したエンベロープを使います。`mdast` がベースラインペイロードで、いちばん優先度の高い経路のままです。ただしエンベロープは、将来のペイロード（markdown-it トークンストリームなど）が、第二の場当たりバイナリ形式を導入せず、同じゼロコピーメモリブロック形を再利用できるように設計しています。

ネイティブ unified ブリッジはいま `transformMdastRaw(source, options)` も使うので、Rust が frontmatter をパースし、内容を除き、mdast を 1 つの外部 `Uint8Array` に直列化したあと、JavaScript がデシリアライズできます。markdown-it と独自パーサ連携では、`prepareSourceRaw(source, {frontmatter})` が、除いた内容と frontmatter JSON だけを運ぶより軽い `prepared-source` エンベロープを提供するので、ソース準備は JavaScript 前処理に落ちず Rust に留まります。

両方のエンベロープは、frontmatter を除いたとき、コンパクトな `source origin` 区画も運びます。JavaScript はそのメタデータで mdast の `position` フィールドを再ベースし、`file.data`、`file.data.oxContent`、Ox Content mdast プラグインコンテキストに `sourceOffset` を出すので、unified 診断と下流プラグインのメッセージは、frontmatter 後の内容スライスではなく、元の完全ソースファイルに揃います。

`parseMdastRaw(source, options)` は mdast 固有の互換ラッパとして残しています。
