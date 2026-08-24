---
title: タクソノミーと関連ページ
description: 関連ページ一覧付きの、オプトイン タグ / カテゴリ用語ページです。
---

# タクソノミーと関連ページ

`taxonomies` が有効なとき、SSG ビルドはページ frontmatter から用語を読み、
次を書きます。

- `/tags/index.html` や `/categories/index.html` のような用語一覧ページ
- `/tags/rust/index.html` のような用語ごとのページ
- 用語を少なくともひとつ共有するソースページ上の関連ページブロック

機能は、オンにするまでオフです。既存サイトは変わりません。

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

`false` または省略すると用語ページと関連一覧はオフのままです。`true` は既定を有効にします。
タクソノミーは `tags` と `categories`、関連ページ上限は 5 です。
オブジェクトは機能を有効にし、設定した欄だけ上書きします。

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

オブジェクトは、既定キー
`tags` と `categories` を置き換えるために `taxonomies` も設定できます。

用語は frontmatter だけから来ます。文字列または文字列配列を受け付けます。
フェンスやインラインコード内の `tags` や `categories` の言及は用語を作りません。

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

用語 slug は安定しており、`[a-z0-9-]` に制限されます。`javascript:`、
`../`、`//evil.com` のような敵対的な値は href から捨てられます。すべての用語、
タイトル、href は HTML エスケープされます。

`publishState` がオンのとき、下書き、非公開、予約公開ページは
用語ページや関連一覧に現れません。

## 関連

- [下書き / 非公開 / 予約公開](./drafts.md)
- [コレクション](./collections.md)
- [サイト生成](./site-generation.md)
- [組み込み機能の概要](../built-in-features.md)
