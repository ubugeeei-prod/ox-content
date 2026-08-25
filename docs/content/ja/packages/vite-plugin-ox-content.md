---
title: "@ox-content/vite-plugin"
description: Environment API 対応の Ox Content Vite プラグイン。
---

# @ox-content/vite-plugin

Environment API 対応の、Ox Content 向けベース Vite プラグインです。

## インストール

```bash
vp install @ox-content/vite-plugin
```

`@ox-content/vite-plugin` はすでに `@ox-content/napi` に依存するので、Vite プラグインを使うときは別途 `vp install @ox-content/napi` は不要です。

## 基本的な使い方

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
    }),
  ],
});
```

## VitePress からの移行

すでに VitePress サイトがあるときは、編集可能な ox-content オプションオブジェクトを生成します。

```bash
ox-content-migrate-vitepress .vitepress/config.ts \
  --src-dir docs \
  --out-dir dist \
  --out ox-content.config.ts
```

CLI は Node.js、Deno、Bun で走れます。

```bash
# Node.js。@ox-content/vite-plugin を入れたあと
ox-content-migrate-vitepress .vitepress/config.ts --out ox-content.config.ts

# Deno
deno run -A npm:@ox-content/vite-plugin/vitepress-migrate .vitepress/config.ts \
  --out ox-content.config.ts

# Bun
bunx --bun @ox-content/vite-plugin .vitepress/config.ts --out ox-content.config.ts
```

生成される `ox-content.config.ts` は、これらの設定を ox-content へ写します。

- `title` / `themeConfig.siteTitle` → `ssg.siteName`
- `base` → `base`
- `themeConfig.sidebar` → `ssg.navigation`
- `themeConfig.socialLinks` / `themeConfig.footer` / `themeConfig.logo` → `ssg.theme`
- `themeConfig.search.placeholder` → `search.placeholder`

ランディングページでは、VitePress 風の `layout: home` frontmatter は ox-content の `layout: entry` と同じ扱いになります。

## オプション

非標準機能のどれがオプトインかを含む、まとめた既定表は [組み込み機能](../built-in-features.md) を見てください。

### srcDir

- 型: `string`
- 既定: `'docs'`

Markdown ファイルのソースディレクトリです。

### extensions

- 型: `string[]`
- 既定: `['.md', '.markdown', '.mdx']`

Vite プラグイン、SSG、開発サーバ、検索インデックス、OG ビューアが処理する Markdown 風ファイル拡張子です。

### outDir

- 型: `string`
- 既定: `'dist'`

ビルド成果物の出力ディレクトリです。

### ssg

- 型: `SsgOptions | boolean`
- 既定: `{ enabled: true }`

SSG（静的サイト生成）オプションです。既定では、ox-content はビルド中に各 Markdown ファイルの静的 HTML を生成します。

```ts
oxContent({
  ssg: {
    enabled: true,
    extension: ".html",
    clean: false,
  },
});
```

#### SsgOptions

| オプション  | 型        | 既定      | 説明                                     |
| ----------- | --------- | --------- | ---------------------------------------- |
| `enabled`   | `boolean` | `true`    | SSG モードのオン / オフ                  |
| `extension` | `string`  | `'.html'` | 出力ファイル拡張子                       |
| `clean`     | `boolean` | `false`   | ビルド前に出力ディレクトリを消す         |
| `bare`      | `boolean` | `false`   | 素の HTML 出力（ナビなし、スタイルなし） |

### Bare モード（ベンチマーク向け）

```ts
oxContent({
  ssg: {
    bare: true, // ナビ / スタイルなしの最小 HTML
  },
});
```

### SSG を切る

```ts
oxContent({
  ssg: false, // SSG を切り、モジュール変換器としてだけ使う
});
```

### gfm

- 型: `boolean`
- 既定: `true`

GitHub Flavored Markdown 拡張を有効にします。

### codeAnnotations

- 型: `boolean | CodeAnnotationsOptions`
- 既定: `false`

フェンス付きコードブロック向けの、オプトインのコード注釈を有効にします。

既定では Ox Content は設定可能な属性構文を使います。VitePress 互換のフェンスメタデータとインライン記法にオプトインすることも、両方同時にオンにすることもできます。

```ts
oxContent({
  highlight: true,
  codeAnnotations: {
    notation: "both",
  },
});
```

既定 `metaKey` の属性構文:

````md
```ts annotate="highlight:1,6;warning:2;error:3"
export function loadUser(input: string) {
  if (!input) console.warn("missing payload");
  throw new Error("missing id");
}

const user = loadUser(payload);
console.log(user);
```
````

VitePress 互換構文:

````md
```ts:line-numbers=10 {1,4} [config.ts]
const user = loadUser(payload);
console.warn("Deprecated") // [!code warning]
throw new Error("boom") // [!code error]
```
````

描画例:

```ts annotate="highlight:1,6;warning:2;error:3"
export function loadUser(input: string) {
  if (!input) console.warn("missing payload");
  throw new Error("missing id");
}

const user = loadUser(payload);
console.log(user);
```

属性名も変えられます。

```ts
oxContent({
  codeAnnotations: {
    metaKey: "markers",
  },
});
```

描画例は [Code Annotations の例](/examples/code-annotations.md) を見てください。

### toc

- 型: `boolean`
- 既定: `true`

目次を生成します。

### embeds

- 型: `BuiltinEmbedOptions | false`
- 既定: `{ github: true, openGraph: true, pm: false, spotify: false, stackBlitz: false, twitter: false, bluesky: false, webContainer: false }`

組み込みの静的埋め込みは変換時に描画され、クライアント側 JavaScript は使いません。非標準の埋め込みはオプトインです。既定表と描画例の全体は [埋め込み](../built-in/embeds.md) を見てください。

```md
<GitHub repo="ubugeeei-prod/ox-content" />

<GitHub permalink="https://github.com/ubugeeei-prod/ox-content/blob/278098b/README.md#L1-L12" />

<GitHub repo="ubugeeei-prod/ox-content" path="README.md" ref="main" loc="1-12" />

<OgCard url="https://github.com/ubugeeei-prod/ox-content" />
```

`permalink`、`url`、`href` は GitHub の `blob` URL を受け付けます。`#L1-L12` フラグメントはソース行範囲として使います。完全なパーマリンクを貼りたくないときは `repo`、`path`、`ref`、`loc` も使えます。ソース埋め込みは GitHub contents API を取り、Open Graph プレビューではなくコードを直接描画します。

すべての埋め込みを切るか、各取得器を設定します。

```ts
oxContent({
  embeds: {
    github: {
      token: process.env.GITHUB_TOKEN,
      maxSourceBytes: 200000,
      maxSourceLines: 120,
    },
    openGraph: {
      timeout: 5000,
    },
    pm: true,
  },
});
```

```ts
oxContent({
  embeds: false,
});
```

#### 組み込み埋め込みのスタイル

組み込み埋め込みのマークアップは安定した CSS クラスを使うので、生成 HTML はクライアント側 JavaScript なしでテーマできます。

リポジトリカードのクラス:

- `.ox-github-card`
- `.ox-github-header`
- `.ox-github-icon`
- `.ox-github-repo`
- `.ox-github-description`
- `.ox-github-stats`
- `.ox-github-stat`
- `.ox-github-language`

ソースコードカードのクラス:

- `.ox-github-code`
- `.ox-github-code-header`
- `.ox-github-code-title`
- `.ox-github-code-loc`
- `.ox-github-code-block`
- `.ox-github-code-line`
- `.ox-github-code-line-number`
- `.ox-github-code-line-content`

Open Graph カードのクラス:

- `.ox-ogp-card`
- `.ox-ogp-simple`
- `.ox-ogp-content`
- `.ox-ogp-title`
- `.ox-ogp-description`
- `.ox-ogp-image`
- `.ox-ogp-meta`
- `.ox-ogp-domain`
- `.ox-ogp-favicon`

```css
.ox-github-card,
.ox-github-code,
.ox-ogp-card {
  border-color: var(--my-border-color);
}

.ox-github-code-line-number,
.ox-ogp-domain {
  color: var(--my-muted-color);
}
```

### docs

- 型: `DocsOptions | false`
- 既定: `{ enabled: true }`

ソースドキュメント生成オプションです。切るときは `false` です。

生成 API ページはいま、要約統計、シグネチャバッジ、1 行のシンボル概要、展開できる詳細、ラベル付き例を含みます。集計件数を持つ機械可読の `docs.json` ペイロードも Markdown の横に出るので、独自ビューアはソースを再パースせずより豊かな体験を作れます。

```ts
oxContent({
  docs: {
    enabled: true,
    src: ["./src"],
    out: "docs/api",
    include: ["**/*.ts"],
    exclude: ["**/*.test.*"],
    format: "markdown",
    toc: true,
    groupBy: "file",
  },
});
```

#### DocsOptions

| オプション | 型                               | 既定                             | 説明                             |
| ---------- | -------------------------------- | -------------------------------- | -------------------------------- |
| `enabled`  | `boolean`                        | `true`                           | docs 生成のオン / オフ           |
| `src`      | `string[]`                       | `['./src']`                      | 走査するソースディレクトリ       |
| `out`      | `string`                         | `'docs/api'`                     | 出力ディレクトリ                 |
| `include`  | `string[]`                       | JS/TS ソース glob                | 含めるファイル                   |
| `exclude`  | `string[]`                       | `['**/*.test.*', '**/*.spec.*']` | 除くファイル                     |
| `format`   | `'markdown' \| 'json' \| 'html'` | `'markdown'`                     | 出力形式                         |
| `private`  | `boolean`                        | `false`                          | @private メンバーを含める        |
| `toc`      | `boolean`                        | `true`                           | 目次を生成する                   |
| `groupBy`  | `'file' \| 'category'`           | `'file'`                         | ファイルまたはカテゴリでグループ |

## docs 生成を切る

```ts
oxContent({
  docs: false, // 組み込み docs 生成をオプトアウト
});
```

### search

- 型: `SearchOptions | boolean`
- 既定: `{ enabled: true }`

全文検索オプションです。Ox Content は BM25 スコア付きの、Rust 駆動の組み込み検索エンジンを載せます。

```ts
oxContent({
  search: {
    enabled: true,
    limit: 10,
    prefix: true,
    placeholder: "Search documentation...",
    hotkey: "/",
  },
});
```

#### SearchOptions

| オプション    | 型        | 既定                        | 説明                                     |
| ------------- | --------- | --------------------------- | ---------------------------------------- |
| `enabled`     | `boolean` | `true`                      | 検索機能のオン / オフ                    |
| `limit`       | `number`  | `10`                        | 検索結果の上限                           |
| `prefix`      | `boolean` | `true`                      | オートコンプリート向けプレフィックス一致 |
| `placeholder` | `string`  | `'Search documentation...'` | 検索入力のプレースホルダ                 |
| `hotkey`      | `string`  | `'/'`                       | 検索を開くキーボードショートカット       |

#### 動き方

1. **ビルド時**: プラグインはすべての Markdown を走査し、Rust ベースの検索エンジンでインデックスを作る
2. **インデックス保存**: インデックスは出力ディレクトリの `search-index.json` に書く
3. **クライアント側検索**: 検索インデックスは必要になったときに読み、検索はすべてクライアント側

#### 機能

- **BM25 スコア**: 業界標準の関連度順位アルゴリズム
- **複数フィールド検索**: タイトル、見出し、本文、コードを異なる重みで索引
- **日本語 / CJK 対応**: CJK 文字の適切なトークン化
- **プレフィックス一致**: オートコンプリート向けタイプアヘッド
- **スコープ付きクエリ**: `@api transform` のようにプレフィックスして区画で結果を制限
- **依存ゼロ**: 外部検索サービスは不要

### 検索を切る

```ts
oxContent({
  search: false, // 組み込み検索を切る
});
```

### 独自検索 UI と使う

仮想モジュール経由で検索インデックスにプログラムから触れます。

```ts
import { search, searchOptions } from "virtual:ox-content/search";

// Search the index
const results = await search("query text", { limit: 5 });

// Scope search to the API reference
const apiResults = await search("@api transform", { limit: 5 });

// Results include:
// - id: document ID
// - title: document title
// - url: document URL
// - score: relevance score
// - snippet: text snippet with context
```

### collections

- 型: `CollectionsOptions | boolean`
- 既定: `{ content: { source: "**/*" } }`

コレクションは Markdown frontmatter とルートメタデータを `virtual:ox-content/collections` 経由で出します。その仮想モジュールを import したときだけビルドします。既定ペイロードはメタデータのみです。Ox Content はディレクトリ歩行、ソースパターンフィルタ、frontmatter パース、ルートパス生成、タイトル取り出しにネイティブ Rust マニフェストビルダを使うので、大きな Markdown 木はファイルごとの JavaScript / NAPI 往復を避け、すべての Markdown を HTML に描画しません。

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { defineCollection, oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "content",
      collections: {
        blog: defineCollection({
          source: "blog/**/*.md",
        }),
        docs: defineCollection({
          source: "docs/**/*.md",
          include: ["body"],
        }),
      },
    }),
  ],
});
```

```ts
import { queryCollection } from "virtual:ox-content/collections";

const posts = await queryCollection("blog")
  .where("draft", "=", false)
  .order("date", "DESC")
  .select("title", "path", "description")
  .all();

const page = await queryCollection("docs").path("/docs/getting-started").first();
```

大きなサイトでは `include` は意図して明示です。

| フィールド | コスト                                              |
| ---------- | --------------------------------------------------- |
| `body`     | 除いた生 Markdown を仮想モジュールに埋め込む。      |
| `html`     | ネイティブ Markdown 変換を走らせ、HTML を埋め込む。 |
| `toc`      | ネイティブ Markdown 変換を走らせ、TOC を埋め込む。  |

シンタックスハイライトや Mermaid 描画のような、ページ単位の完全な JavaScript 後処理では、Markdown モジュールを直接 import してください。コレクションの `html` はクエリペイロード向けに最適化しています。

コレクション全体を切るときは `collections: false` です。

## Environment API

プラグインは、SSG に寄せた描画のため、Vite の Environment API で `markdown` 環境を作ります。

## HMR 対応

開発中、Markdown はホットリロードされます。プラグインは独自 HMR イベントを送ります。

```ts
// Client-side
if (import.meta.hot) {
  import.meta.hot.on("ox-content:update", (data) => {
    console.log("Markdown updated:", data.file);
  });
}
```

## 仮想モジュール

プラグインは次の仮想モジュールを提供します。

- `virtual:ox-content/config` — 解決済みプラグイン設定
- `virtual:ox-content/runtime` — ランタイムユーティリティ
- `virtual:ox-content/search` — 検索機能
- `virtual:ox-content/collections` — コレクションクエリヘルパー

```ts
import config from "virtual:ox-content/config";
import { useMarkdown, withBase, withoutBase } from "virtual:ox-content/runtime";
import { search, searchOptions } from "virtual:ox-content/search";
import { queryCollection } from "virtual:ox-content/collections";

const assetUrl = withBase("/og.png");
const routePath = withoutBase("/docs/guide");

// Use the search function
const results = await search("query", { limit: 10 });

const page = await queryCollection("content").path("/guide").first();
```
