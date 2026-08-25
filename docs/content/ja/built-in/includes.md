---
title: ファイル取り込み
description: 共有スニペット向けの、オプトインの Markdown ファイル取り込み。
---

# ファイル取り込み

Markdown ファイルの取り込みはオプトインです。有効にすると、HTML コメントのディレクティブが、ホスト文書のパース前に別の Markdown ファイルをインライン展開します。

| オプション | 型                           | 既定    |
| ---------- | ---------------------------- | ------- |
| `includes` | `boolean` / `IncludeOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      includes: true,
    }),
  ],
};
```

## ディレクティブ

```md
<!-- @include: ./shared/warning.md -->
```

パスは引用符で囲んでも構いません（`"./shared/warning.md"` または `'./shared/warning.md'`）。パス周りの空白は切り詰めます。

このサイトは `includes` をオンにしているので、次の段落は `_fragments/include-warning.md` のライブ取り込みです。

<!-- @include: ./_fragments/include-warning.md -->

## パス解決

- 相対パスは現在のファイルから解決します。
- `@/` と先頭の `/` は `rootDir` から解決します（`rootDir` を省略したときは Vite のプロジェクトルート）。
- canonicalize のあと、`rootDir` の外に出るパスは拒否します。ディレクティブはソースに残り、変換エラーを報告します。
- 見つからない、または読めない対象も変換エラーです。ディレクティブはそのまま残します。

```ts
oxContent({
  includes: {
    rootDir: process.cwd(),
  },
});
```

## 入れ子の取り込み

取り込まれたファイルは、さらに別のファイルを取り込めます。循環（`A` が `B` を取り込み、`B` が `A` を取り込む）と、16 段より深い入れ子は変換エラーです。それらのディレクティブはリテラルのままです。

## 展開されないもの

フェンスコード、インデントコード、インラインコードの中ではディレクティブは展開しません。`<!-- @include: PATH -->` ちょうどではない HTML コメントはそのままです。閉じていないコメントも同様です。

取り込まれた Markdown はそのあとホスト文書の一部としてパースされるので、断片の中の見出しやリストは本物の見出しやリストになります。

## 関連

- [コードブロック](./code-blocks.md)
- [組み込み機能の一覧](../built-in-features.md)
