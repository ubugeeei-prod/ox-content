---
title: 略語・用語集
description: オプトインの用語定義をアクセス可能な `<abbr>` として描画します。
---

# 略語・用語集

長く残るドキュメントには頭字語や製品名が増えます。用語を一度定義すると、
ox-content は一致する本文を `title` 付きの `<abbr class="ox-abbr">` に
展開します。オプトインで、既定はオフです。クライアント JavaScript はありません。

| オプション      | 型                                 | 既定    |
| --------------- | ---------------------------------- | ------- |
| `abbreviations` | `boolean` / `AbbreviationsOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      abbreviations: true,
    }),
  ],
};
```

`false` または省略はソースを変えません。`true` またはオブジェクトは変換を
オンにします。

## 書き方

Markdown Extra 形式の定義を行単位で書きます。

*[LSP]: Language Server Protocol

```md
*[LSP]: Language Server Protocol

Use LSP in the editor.
```

Use LSP in the editor.

定義行は出力から取り除かれます。一致は Unicode の単語境界なので、`XLSPY`
や `myLSP` はそのままです。設定の `terms` も同じルールで、共有の用語集に
置けます。同じキーのページ内定義は設定より優先します。

`*[LSP]` や `*[LSP]:` のような不正な行は見えるまま残します。フェンス、
インデントコード、インラインコード、HTML コメント、生の `code` / `pre` /
`script` / `style`、既存のリンクは書き換えません。

## オプション

```ts
oxContent({
  abbreviations: {
    terms: {
      LSP: "Language Server Protocol",
    },
    firstUseOnly: false,
  },
});
```

| フィールド     | 型                       | 既定    |
| -------------- | ------------------------ | ------- |
| `enabled`      | `boolean`                | `true`  |
| `terms`        | `Record<string, string>` | `{}`    |
| `firstUseOnly` | `boolean`                | `false` |

`firstUseOnly: true` は各用語の最初の出現だけを包みます。既定はすべての
一致を展開します。`title` と用語テキストは HTML エスケープされます。

## 関連

- [キーボードキー](./keyboard-keys.md)
- [構文拡張](./syntax-extensions.md)
- [組み込み機能の一覧](../built-in-features.md)
