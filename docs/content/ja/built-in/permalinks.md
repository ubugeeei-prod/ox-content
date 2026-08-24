---
title: パーマリンクと Cascade
description: frontmatter の独自 URL と、ディレクトリ単位の既定 frontmatter。
---

# パーマリンクと Cascade

ページ URL は通常、Markdown のファイルツリーに従います。公開パスを変えたいときは `permalinks`、ディレクトリで frontmatter を共有したいときは `cascade` です。

どちらも明示するまでオフです。既存サイトは変わりません。

```ts
oxContent({
  permalinks: true,
  cascade: true,
});
```

| オプション   | 型                              | 既定    |
| ------------ | ------------------------------- | ------- |
| `permalinks` | `boolean` / `PermalinksOptions` | `false` |
| `cascade`    | `boolean` / `CascadeOptions`    | `false` |

## パーマリンク

`permalinks` がオンだと、frontmatter がファイルツリー URL を置き換えます。

```md
---
title: はじめに
permalink: /getting-started
---
```

`slug` はファイル名だけを置き、親ディレクトリは残します。危険なスキームや `..` は拒否されます。

## Cascade

ディレクトリの `_index.md`（または設定したインデックス名）の frontmatter が、子ページの省略フィールドの既定になります。子が書いたフィールドが勝ちます。

## 関連

- [英語版ガイド](/built-in/permalinks.md)
- [リダイレクトとエイリアス](./redirects.md)
