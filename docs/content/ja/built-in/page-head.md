---
title: ページ head
description: テーマ付き・bare・独自ホスト・ssg: false で共有する、ビルド時の Unhead 互換 page-head API。
---

# ページ head

Ox Content は `<title>`、`meta`、`link`、JSON-LD を **ビルド時** に解決します。
Unhead ランタイムも、クライアントの head マネージャも、追加のブラウザ JS もありません。

テーマ付きページ、bare、`ssg.render`、`ssg: false` は同じ型付きリゾルバを使います。
文書を自分で持つときは、こう呼びます。

```ts
import { renderHead } from "@ox-content/vite-plugin";

const { html, diagnostics } = renderHead({
  site: { name: "Docs", url: "https://example.com" },
  title: "Guide",
  description: "How it works",
  canonical: "https://example.com/guide/",
});
```

`html` はすでにエスケープ済みです。`<head>` に差し込みます。独自の
`ssg.render` レイアウトなら `raw()` が使えます。
`@ox-content/vite-plugin-svelte` も同じ `renderHead` を再エクスポートします。

| ホスト                      | head タグの出し方                                                                    |
| --------------------------- | ------------------------------------------------------------------------------------ |
| 既定テーマ                  | 組み込みリゾルバ。従来どおりの OG / Twitter。                                        |
| `bare: true`                | 同じリゾルバ。説明、`siteUrl`、サイト名、OG 画像があるときだけソーシャルタグを出す。 |
| `ssg.render` / `ssg: false` | 何も注入しない。`renderHead` を呼んで自分で書く。                                    |

既存サイトのタグは、[SEO](./seo.md)（`ssg.siteUrl`、`robots`、ロケール alternate）
や [JSON-LD](./json-ld.md) をオプトインしない限り変わりません。

## デスクリプタ

`renderHead` は Unhead 形の入力を受けます。同じ identity の後勝ちで、位置は最初のままです。

| フィールド | Identity                                               |
| ---------- | ------------------------------------------------------ |
| `title`    | title は一つ                                           |
| `metas`    | `key`、なければ `name` / `property` / `httpEquiv`      |
| `links`    | `key`、なければ `rel`、または `alternate` + `hreflang` |
| `jsonLd`   | `key`                                                  |

```ts
renderHead({
  title: "Guide",
  titleTemplate: "%s · %siteName",
  site: { name: "Docs" },
  metas: [{ name: "theme-color", content: "#111" }],
  links: [{ rel: "icon", href: "https://example.com/favicon.ico" }],
  jsonLd: [{ key: "blog", json: JSON.stringify({ "@type": "BlogPosting" }) }],
});
```

`twitter.imggg` のような未知キーは TypeScript エラーです。追加タグは `metas` /
`links` に入れてください。

## 安全性

属性値は HTML エスケープします。JSON-LD の script 本文は `<`、`>`、`&`、
U+2028 / U+2029 を `\uXXXX` にします。

`javascript:`、`data:`、`vbscript:`、プロトコル相対の `//` は落とします。
独自ホストは既定で検証します。組み込みの themed / bare は、これまで計算していた
OG 画像と canonical の文字列をそのまま出します。

`ssg.headValidation` は [SEO](./seo.md) を見てください。

## 関連

- [SEO](./seo.md)
- [JSON-LD](./json-ld.md)
- [サイト生成](./site-generation.md)
- 追跡: [#819](https://github.com/ubugeeei-prod/ox-content/issues/819)
