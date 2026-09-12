---
title: コレクション
description: SQL 風ビルダで、型付きコレクションとして Markdown を問い合わせます。マニフェストは Rust が生成します。
---

# コレクション

コレクションは、サイトの Markdown ファイルを遅延読み込み可能な問い合わせデータとして出します。ブログ索引、変更履歴、「関連ページ」一覧、他ページを列挙する任意のページ向けです。マニフェストはビルド時に Rust がネイティブ生成します。クエリはプレーンデータに対してクライアント側で走るので、頼んでいないページ本文は読みません。

`srcDir` 以下のすべての Markdown を覆う既定の `content` コレクションは最初からあります。`collections: false` で機能を切れます。

## コレクションの定義

`collections` レコードの値は、完全なオプションオブジェクト、glob 文字列、または glob の配列です。

```ts
import { oxContent, defineCollections } from "@ox-content/vite-plugin";

oxContent({
  collections: defineCollections({
    blog: {
      source: "blog/**/*.md",
      include: ["html", "toc"],
    },
    changelog: "changelog/*.md",
    guides: ["guide/**/*.md", "tutorials/**/*.md"],
  }),
});
```

| オプション | 既定              | 目的                                                  |
| ---------- | ----------------- | ----------------------------------------------------- |
| `source`   | すべての Markdown | `srcDir` から解決する glob パターン。                 |
| `include`  | `[]`              | エントリごとの追加フィールド: `body`、`html`、`toc`。 |
| `validate` | `undefined`       | パース済み document ごとのビルド時 validation hook。  |

既定では各エントリはメタデータだけです。`include` でコレクションごとに重いフィールドを足します。`body` は生 Markdown、`html` はネイティブ描画した HTML、`toc` はパース済み目次です。`1.guide/2.install.md` のような数値ルートプレフィックスは、生成される `path` から除きます。

## Frontmatter の validation

ソースパスに依存する frontmatter ルールがある場合は `validate` を使います。この hook は Ox Content が frontmatter をパースしたあと、permalink / cascade の書き換えを適用する前に、コレクションマニフェスト生成の中で走ります。document を無効にするには文字列または文字列配列を返します。何も返さない、`null`、`false` は通過です。無効な document はまとめて報告され、ビルドは失敗します。

```ts
import { oxContent, defineCollections } from "@ox-content/vite-plugin";

oxContent({
  permalinks: true,
  collections: defineCollections({
    blog: {
      source: "blog/*/index.md",
      validate({ frontmatter, path }) {
        const permalink =
          typeof frontmatter.permalink === "string" ? frontmatter.permalink : "(missing)";
        if (permalink !== path) {
          return `expected permalink ${path}, found ${permalink}`;
        }
      },
    },
  }),
});
```

`path` と `stem` は permalink / cascade 書き換え前のファイルツリー由来 route です。`source` は `srcDir` からの相対パスで、`documentPath` は絶対ソースファイルパスです。

## Frontmatter の読み書き

`parseFrontmatter` と `stringifyFrontmatter` は、コレクションのパイプラインが使う
frontmatter 処理をそのまま公開します。ビルドの外で単一の document を扱うツール
（scaffold スクリプトや、ファイルを直読みするローダー）のための API です。

```ts
import { parseFrontmatter, stringifyFrontmatter } from "@ox-content/vite-plugin";

const { frontmatter, content } = parseFrontmatter(source);

const document = stringifyFrontmatter(
  { permalink: "/blog/post", title: "Post", draft: false },
  "Body\n",
);
```

両者は round-trip します。`stringifyFrontmatter` で書き出した document は渡した値
のままパースし直せるので、生成したファイルはビルドが読むのと同じ結果になります。
キーの順序は保たれ、キーが空の場合は空ブロックを出さずに body をそのまま返します。

## エントリの形

各エントリは `CollectionEntry` です。

```ts
interface CollectionEntry {
  id: string; // "content/built-in/collections.md"
  collection: string; // "content"
  path: string; // "/built-in/collections"
  stem: string; // "built-in/collections"
  source: string; // srcDir からの相対ソースパス
  extension: string; // ".md"
  title: string; // frontmatter の title または最初の見出し
  description?: string;
  frontmatter: Record<string, unknown>;
  body?: string; // include: ["body"]
  html?: string; // include: ["html"]
  toc?: TocEntry[]; // include: ["toc"]
}
```

## 問い合わせ

マニフェストは、SQL 風のクエリビルダ付き仮想モジュールとして出ます。

```ts
import { queryCollection } from "virtual:ox-content/collections";

const recent = await queryCollection("content")
  .where("path", "LIKE", "/built-in/%")
  .order("title", "ASC")
  .limit(5)
  .all();

const page = await queryCollection("content").path("/getting-started").first();

const total = await queryCollection("content").count();
```

モジュールは `getCollection(name)`（全エントリのプレーン配列）と `collectionNames` も export します。

TypeScript プログラムが仮想モジュールだけを import する場合は、package の ambient declarations を一度 `tsconfig.json` で読ませます。

```json
{
  "compilerOptions": {
    "types": ["@ox-content/vite-plugin"]
  }
}
```

ローカルの `declare module "virtual:ox-content/collections"` shim は不要です。

### ビルダ API

| メソッド                                   | 挙動                                              |
| ------------------------------------------ | ------------------------------------------------- |
| `path(path)`                               | 正規化付きの `where("path", "=", path)` の短縮。  |
| `select(...fields)`                        | 各結果の指定フィールドだけ残す。                  |
| `where(field, operator, value?)`           | AND 条件を足す。                                  |
| `where(field, value)`                      | 2 引数形は等価。                                  |
| `andWhere(q => ...)` / `orWhere(q => ...)` | AND / OR でつなぐグループ条件。                   |
| `order(field, "ASC" \| "DESC")`            | ソート。繰り返し呼ぶと複数キー。                  |
| `limit(n)` / `skip(n)`                     | ページング。                                      |
| `all()` / `first()` / `count()`            | 実行: 配列、先頭エントリまたは `null`、一致件数。 |

`field` は入れ子データへのドットパスを受け付けるので、frontmatter キーを直接問い合わせられます。

```ts
const drafts = await queryCollection("blog")
  .where("frontmatter.draft", "=", true)
  .orWhere((q) => q.where("frontmatter.date", "IS NULL"))
  .all();
```

### 演算子

`=` `==` `!=` `<>` `>` `>=` `<` `<=` `IN` `NOT IN` `BETWEEN` `NOT BETWEEN`
`IS NULL` `IS NOT NULL` `LIKE` `NOT LIKE`

`LIKE` は SQL のワイルドカードを、大文字小文字を区別せずに使います。`%` は任意長、`_` はちょうど 1 文字です。比較は数値を意識します。数値は数値として、日付っぽい値は日付として、文字列は `localeCompare(..., { numeric: true })` です。

## 描画例

このサイトの既定 `content` コレクションは、すべてのドキュメントページを索引します。この区画を問い合わせると、

```ts
await queryCollection("content")
  .where("path", "LIKE", "/built-in/%")
  .order("path", "ASC")
  .select("path", "title")
  .all();
```

このサイドバーグループのガイドのエントリが返ります。独自の索引ページを動かすのと同じデータです。

```json
[
  { "path": "/built-in/code-blocks", "title": "Code Blocks" },
  { "path": "/built-in/collections", "title": "Collections" },
  { "path": "/built-in/embeds", "title": "Embeds" },
  { "path": "/built-in/markdown", "title": "Markdown Baseline" },
  { "path": "/built-in/mermaid", "title": "Mermaid Diagrams" },
  { "path": "/built-in/quality-checks", "title": "Quality Checks" },
  { "path": "/built-in/search", "title": "Search" },
  { "path": "/built-in/site-generation", "title": "Site Generation" },
  { "path": "/built-in/syntax-extensions", "title": "Syntax Extensions" }
]
```

## 関連

- [サイト生成](./site-generation.md) — マニフェストを生成するビルド。
- [検索](./search.md) — 構造化ではなく全文の問い合わせ。
