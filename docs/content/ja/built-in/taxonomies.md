---
title: タクソノミー
description: 関連ページ一覧付きの、オプトインのタグ / カテゴリ用語ページ。
---

# タクソノミー

`taxonomies` を有効にすると、SSG ビルドはページ frontmatter から用語を読み、次を書き出します。

- `/tags/index.html` や `/categories/index.html` のような用語一覧ページ
- `/tags/rust/index.html` のような用語ごとのページ
- 用語を 1 つ以上共有するソースページ上の関連ページブロック

機能は自分でオンにするまでオフです。既存サイトはそのままです。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      taxonomies: true,
    }),
  ],
};
```

`false` または省略は用語ページと関連一覧をオフのままにします。`true` は既定でオンです。タクソノミーは `tags` と `categories`、関連ページ上限は 5 です。オブジェクトを渡すと機能はオンになり、設定したフィールドだけ上書きします。

```ts
oxContent({
  taxonomies: {
    taxonomies: ["topics"],
    relatedLimit: 3,
  },
});
```

| オプション     | 型                              | 既定    |
| -------------- | ------------------------------- | ------- |
| `taxonomies`   | `boolean` / `TaxonomiesOptions` | `false` |
| `relatedLimit` | `number`                        | `5`     |

オブジェクトでは `taxonomies` を設定して、既定キー `tags` と `categories` を置き換えられます。

用語は frontmatter だけから取ります。文字列または文字列配列を受け付けます。フェンスやインラインコードの中の `tags` や `categories` は用語を作りません。

```md
---
title: Install
tags:
  - rust
  - napi
categories: guide
---
```

`categories: guide` は単一の文字列用語です。上の `tags` は文字列配列です。

用語スラッグは安定で、`[a-z0-9-]` に制限されます。`javascript:`、`../`、`//evil.com` のような敵意のある値は href から捨てます。用語、タイトル、href はすべて HTML エスケープされます。

`publishState` がオンのとき、下書き、非公開、予約公開のページは用語ページにも関連一覧にも出ません。

## 関連

- [下書き / 非公開 / 予約公開](./drafts.md)
- [コレクション](./collections.md)
- [サイト生成](./site-generation.md)
- [組み込み機能の一覧](../built-in-features.md)
