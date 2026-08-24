---
title: コレクション
description: Rust が生成するマニフェストに裏打ちされた、SQL 風ビルダーで Markdown ファイルを型付きコレクションとして問い合わせます。
---

# コレクション

コレクションは、サイトの Markdown ファイルを遅延読み込み可能な問い合わせデータとして公開します。
ブログ索引、変更履歴、「関連ページ」一覧、他のページを列挙する任意のページ向けです。
マニフェストはビルド時に Rust でネイティブ生成されます。
問い合わせはプレーンデータに対してクライアント側で走るので、頼んでいないページ本文は
フィルタや並べ替えで読み込まれません。

`srcDir` 配下のすべての Markdown ファイルを覆う既定の `content` コレクションは
最初からあります。`collections: false` で機能を切ります。

## コレクションを定義する

`collections` レコードの値は、完全なオプションオブジェクト、glob
文字列、または glob の配列にできます。

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

| オプション | 既定               | 目的                                           |
| ---------- | ------------------ | ---------------------------------------------- |
| `source`   | すべての Markdown  | `srcDir` から解決する glob パターン。          |
| `include`  | `[]`               | エントリごとの追加欄: `body`、`html`、`toc`。  |

既定では各エントリはメタデータだけを持ちます。`include` はコレクションごとに重い
欄をオプトインします。`body` は生 Markdown、`html` はネイティブ描画 HTML、
`toc` はパース済み目次です。`1.guide/2.install.md` のような数値ルート接頭辞は
生成された `path` から取り除かれます。

## エントリの形

各エントリは `CollectionEntry` です。

```ts
interface CollectionEntry {
  id: string; // "content/built-in/collections.md"
  collection: string; // "content"
  path: string; // "/built-in/collections"
  stem: string; // "built-in/collections"
  source: string; // source file path relative to srcDir
  extension: string; // ".md"
  title: string; // frontmatter title or first heading
  description?: string;
  frontmatter: Record<string, unknown>;
  body?: string; // include: ["body"]
  html?: string; // include: ["html"]
  toc?: TocEntry[]; // include: ["toc"]
}
```

## 問い合わせ

マニフェストは、SQL 風クエリビルダー付きの仮想モジュール経由で公開されます。

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

モジュールは `getCollection(name)`（全エントリのプレーン配列）と
`collectionNames` もエクスポートします。

### ビルダー API

| メソッド                                   | 振る舞い                                                     |
| ------------------------------------------ | ------------------------------------------------------------ |
| `path(path)`                               | 正規化付きの `where("path", "=", path)` 短縮形。             |
| `select(...fields)`                        | 各結果に指名した欄だけを残す。                               |
| `where(field, operator, value?)`           | AND 条件を足す。                                             |
| `where(field, value)`                      | 2 引数形式は等価。                                           |
| `andWhere(q => ...)` / `orWhere(q => ...)` | AND / OR でつなぐグループ条件。                              |
| `order(field, "ASC" \| "DESC")`            | 並べ替え。複数キーなら繰り返し呼ぶ。                         |
| `limit(n)` / `skip(n)`                     | ページネーション。                                           |
| `all()` / `first()` / `count()`            | 実行: 配列、先頭エントリまたは `null`、または一致数。        |

`field` はネストしたデータへのドットパスを受け付けるので、frontmatter キーは直接問い合わせできます。

```ts
const drafts = await queryCollection("blog")
  .where("frontmatter.draft", "=", true)
  .orWhere((q) => q.where("frontmatter.date", "IS NULL"))
  .all();
```

### 演算子

`=` `==` `!=` `<>` `>` `>=` `<` `<=` `IN` `NOT IN` `BETWEEN` `NOT BETWEEN`
`IS NULL` `IS NOT NULL` `LIKE` `NOT LIKE`

`LIKE` は SQL ワイルドカードを大文字小文字無視で使います。`%` は任意長の
文字に一致し、`_` はちょうど 1 文字です。比較は数値を意識します。
数値は数値として、日付風の値は日付として、文字列は
`localeCompare(..., { numeric: true })` で比べます。

## 描画例

このサイトの既定 `content` コレクションは、すべてのドキュメントページを索引します。
このセクションを問い合わせると:

```ts
await queryCollection("content")
  .where("path", "LIKE", "/built-in/%")
  .order("path", "ASC")
  .select("path", "title")
  .all();
```

このサイドバーグループのガイド向けエントリが返ります。同じデータで
カスタム索引ページを駆動できます。

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
