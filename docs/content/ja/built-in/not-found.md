---
title: カスタム 404
description: 生成 HTML の横に書き出す、オプトインのテーマ付き 404 ページ。
---

# カスタム 404

`ssg.notFound` を有効にすると、SSG ビルドは既定レイアウト（ナビ、検索、その他のサイト chrome）でテーマ付き 404 ページを書き出します。このページは検索インデックスに入りません。`sitemap.xml` / `llms.txt` が有効でもそこからも外れます。

機能は自分でオンにするまでオフです。既存サイトはそのままです。

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

`false` または省略はファイルを出しません。`true` は既定でオンです（`srcDir` の `404.md`、出力 `404.html`）。オブジェクトを渡すと機能はオンになり、設定したフィールドだけ上書きします。

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

ソースファイルがなくても、ビルドは「Page not found」というタイトルのテーマ付きページを書き出します。オプションをオンにすると、必ず出力ファイルができます。

`404.md` のタイトルやその他のメタデータは HTML 文書内でエスケープされるので、`<title>` や属性の外へは出られません。

## 関連

- [サイト生成](./site-generation.md)
- [検索](./search.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [組み込み機能の一覧](../built-in-features.md)
