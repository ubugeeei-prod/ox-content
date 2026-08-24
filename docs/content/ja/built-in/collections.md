---
title: コレクション
description: Markdown を型付きコレクションとして、SQL 風ビルダで問い合わせる。
---

# コレクション

コレクションは、サイトの Markdown を遅延読み込みできる問い合わせデータにします。ブログ一覧、changelog、関連ページなど、他ページを列挙する用途向けです。マニフェストはビルド時に Rust で生成し、問い合わせはプレーンデータに対してクライアントで走ります。

`srcDir` 以下の全 Markdown を覆う `content` コレクションが既定であります。`collections: false` で機能ごとオフです。

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

| オプション | 既定        | 役割                                  |
| ---------- | ----------- | ------------------------------------- |
| `source`   | 全 Markdown | `srcDir` からの glob                  |
| `include`  | `[]`        | 追加フィールド: `body`, `html`, `toc` |

既定のエントリはメタデータだけです。`include` で重いフィールドを足します。`1.guide/2.install.md` のような数値プレフィックスは生成 `path` から落ちます。

## 関連

- [英語版ガイド](/built-in/collections.md)
- [RSS / Atom / JSON フィード](./feeds.md)
