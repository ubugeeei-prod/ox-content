---
title: カスタム 404 ページ
description: 生成 HTML の隣に書く、オプトインのテーマ付き 404 ページです。
---

# カスタム 404 ページ

`ssg.notFound` が有効なとき、SSG ビルドは既定レイアウトでテーマ付き 404 ページを書きます。
ナビ、検索、残りのサイトクロム付きです。ページは検索インデックスと、
それらが有効なときの `sitemap.xml` / `llms.txt` から省かれます。

機能は、オンにするまでオフです。既存サイトは変わりません。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        notFound: true,
      },
    }),
  ],
};
```

`false` または省略するとファイルはオフのままです。`true` は既定を有効にします（`srcDir` の
`404.md`、出力 `404.html`）。オブジェクトは機能を有効にし、設定した欄だけ上書きします。

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

| オプション     | 型                            | 既定         |
| -------------- | ----------------------------- | ------------ |
| `ssg.notFound` | `boolean` / `NotFoundOptions` | `false`      |
| `source`       | `string`                      | `"404.md"`   |
| `output`       | `string`                      | `"404.html"` |

ソースファイルがなくても、ビルドは「Page not found」というタイトルのテーマ付きページを書きます。
オプションを有効にすると、常に出力ファイルができます。

`404.md` からのタイトルとその他のメタデータは HTML 文書内でエスケープされるので、
`<title>` や属性から抜けられません。

## 関連

- [サイト生成](./site-generation.md)
- [検索](./search.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [組み込み機能の概要](../built-in-features.md)
