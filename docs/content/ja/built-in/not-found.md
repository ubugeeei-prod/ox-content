---
title: カスタム 404
description: 生成 HTML の横に書く、テーマ付き 404。
---

# カスタム 404

`ssg.notFound` を有効にすると、デフォルトレイアウト（ナビ、検索、サイト chrome）の 404 を書きます。検索インデックスと、有効なら `sitemap.xml` / `llms.txt` からは外れます。

省略または `false` ではオフです。

```ts
oxContent({
  ssg: {
    notFound: true,
  },
});
```

`true` の既定は `srcDir` の `404.md` を `404.html` に出すことです。オブジェクトで上書きできます。

```ts
oxContent({
  ssg: {
    notFound: {
      source: "pages/missing.md",
      output: "not-found.html",
    },
  },
});
```

このサイトの日本語 404 本文は [`../404.md`](../404.md) です。ロケール付き URL でも、ファイル名はホストの慣習に合わせて `404.html` です。

## 関連

- [英語版ガイド](/built-in/not-found.md)
