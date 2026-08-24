---
title: タクソノミーと関連ページ
description: タグ / カテゴリの用語ページと、関連ページ一覧。
---

# タクソノミーと関連ページ

`taxonomies` を有効にすると、frontmatter の用語を読んで次を書きます。

- `/tags/index.html` や `/categories/index.html` のような用語一覧
- `/tags/rust/index.html` のような用語ページ
- 用語を共有するソースページへの関連ページブロック

省略または `false` ではオフです。既存サイトは変わりません。

```ts
oxContent({
  taxonomies: true,
});
```

`true` の既定は `tags` と `categories`、関連上限 5 件です。

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

用語は frontmatter だけです。文字列または文字列配列を受け付けます。このドキュメントサイトでは `taxonomies` はオフです。

## 関連

- [英語版ガイド](/built-in/taxonomies.md)
- [コレクション](./collections.md)
