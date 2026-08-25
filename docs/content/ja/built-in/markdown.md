---
title: Markdown の土台
description: 最初からオンの GitHub Flavored Markdown、frontmatter、目次の既定。
---

# Markdown の土台

よく使う GitHub Flavored Markdown の挙動は既定でオンです。このページの機能に設定は不要です。下の描画例はすべて、このドキュメントサイト自身が既定設定で出しています。

| オプション      | 型        | 既定         | 目的                                 |
| --------------- | --------- | ------------ | ------------------------------------ |
| `gfm`           | `boolean` | `true`       | GitHub Flavored Markdown 拡張。      |
| `tables`        | `boolean` | `true`       | GFM の表。                           |
| `taskLists`     | `boolean` | `true`       | `- [ ]` / `- [x]` チェックボックス。 |
| `strikethrough` | `boolean` | `true`       | `~~text~~`。                         |
| `autolinks`     | `boolean` | `gfm` を継承 | 裸の URL をリンクにする。            |
| `footnotes`     | `boolean` | `true`       | `[^1]` 参照と定義。                  |
| `frontmatter`   | `boolean` | `true`       | 描画前に YAML frontmatter をパース。 |
| `toc`           | `boolean` | `true`       | 見出しから目次を作る。               |
| `tocMaxDepth`   | `number`  | `3`          | TOC に含める最も深い見出しレベル。   |

上のオプションはどれも CommonMark の上の拡張で、それぞれオプトアウトです。下のパーサは完全適合を狙います。コアプロファイルでは [CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/) の仕様例 652 件を正しく描画し、毎回の CI で確認します。拡張を使わない文書は、適合スイートの HTML 正規化規則の下で仕様に適合します。マークアップはバイト一致ではありません。ox-content が見出しに slug の `id` 属性を付けるからです。プロファイルごとの数値は [CommonMark 適合](../performance.md#commonmark-適合) を見てください。

より厳しい CommonMark が必要なサイトでは、明示的にオフにします。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      strikethrough: false,
      taskLists: false,
    }),
  ],
};
```

## 表

```md
| Feature    | Status  |
| ---------- | ------- |
| Tables     | Default |
| Task lists | Default |
```

描画:

| Feature    | Status  |
| ---------- | ------- |
| Tables     | Default |
| Task lists | Default |

## タスクリスト

```md
- [x] Parse Markdown in Rust
- [x] Render HTML
- [ ] Take over the world
```

描画:

- [x] Parse Markdown in Rust
- [x] Render HTML
- [ ] Take over the world

## 取り消し線

```md
Ox Content is ~~slow~~ fast.
```

描画:

Ox Content is ~~slow~~ fast.

## Autolink

裸の URL はリンクになります。既定は `gfm` に従うので、`autolinks: false` で GFM の残りを捨てずにオプトアウトできます。

```md
Docs live at https://ubugeeei-prod.github.io/ox-content/
```

描画:

Docs live at https://ubugeeei-prod.github.io/ox-content/

自動リンクされた URL は新しいタブで開き、`rel="noopener noreferrer"` が付きます。

## 脚注

```md
Ox Content renders footnotes natively.[^1]

[^1]: This is the footnote body.
```

描画:

Ox Content renders footnotes natively.[^1]

[^1]: This is the footnote body.

参照は上付きリンクになり、定義はソースに書いた場所で戻るリンク付きで描画されます。ページ末に定義を置くと、そこに集まります。

## Frontmatter

YAML frontmatter は描画前にパースされ、出力 HTML には出ません。このページは次で始まります。

```yaml
---
title: Markdown の土台
description: 最初からオンの GitHub Flavored Markdown、frontmatter、目次の既定。
---
```

SSG テーマは `title` を文書タイトルとナビに使い、`description` を `<meta name="description">` と Open Graph タグに使います。他のキーはそのまま通ります。`.md` モジュールは `frontmatter` export として出し、[コレクション](./site-generation.md#コレクション) はクエリに渡し、[独自トランスフォーマ](./site-generation.md#独自トランスフォーマ) は `context.frontmatter` として受け取ります。

```ts
import { frontmatter, html } from "./guide.md";

console.log(frontmatter.title); // "Markdown Baseline"
```

## 目次

TOC は変換中に見出しから作ります。まさにこのページのサイドバーナビもそれで動いています。`tocMaxDepth: 3` は既定で `#` から `###` までを含めます。より深い見出しは描画されますが、索引には入りません。

```ts
oxContent({
  toc: true,
  tocMaxDepth: 3,
});
```

TOC は `.md` モジュールに `{ depth, text, slug, children }` の木として出ます。

```json
[
  {
    "depth": 1,
    "text": "Install Guide",
    "slug": "install-guide",
    "children": [
      { "depth": 2, "text": "Prerequisites", "slug": "prerequisites", "children": [] },
      { "depth": 2, "text": "Run Vite", "slug": "run-vite", "children": [] }
    ]
  }
]
```

見出しには安定した `id` 属性（上の `slug`）も付くので、[#タスクリスト](#タスクリスト) のような深いリンクがどのページでも動きます。

## 関連

- [構文拡張](./syntax-extensions.md) — この土台の上の、オプトインの執筆構文。
- [組み込み機能の一覧](../built-in-features.md)
