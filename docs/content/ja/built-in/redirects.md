---
title: リダイレクトとエイリアス
description: frontmatter エイリアスと書き換えマップからの、オプトイン静的 HTML リダイレクトです。
aliases:
  - /built-in/aliases
---

# リダイレクトとエイリアス

`redirects` が有効なとき、SSG ビルドは各古いパスに小さな静的 HTML ページを書きます。
ページは meta refresh と canonical リンクを使うので、リネーム後も inbound
URL は動き続けます。どの静的ホストでも動きます。

このページも `aliases: [/built-in/aliases]` を宣言しているので、ドキュメントサイト
自身がその古いパス向けのライブリダイレクトを出荷します。

機能は、オンにするまでオフです。既存サイトは変わりません。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      redirects: true,
    }),
  ],
};
```

`false` または省略すると何も書きません。`true` は既定を有効にします。オブジェクトは
機能を有効にし、設定した欄だけ上書きします。

```ts
oxContent({
  redirects: {
    map: {
      "/old-guide": "/guide",
    },
  },
});
```

`{ "/old-guide": "/guide" }` のようなパスマップは、オプションオブジェクトの代わりに渡せ、
そのマップ付きで機能を有効にします。

| オプション      | 型                                        | 既定    |
| --------------- | ----------------------------------------- | ------- |
| `redirects`     | `boolean` / パスマップ / `RedirectsOptions` | `false` |
| `map`           | `Record<string, string>`                  | `{}`    |
| `netlify`       | `boolean`                                 | `false` |
| `headers`       | `boolean`                                 | `false` |
| `json`          | `boolean`                                 | `false` |
| `allowExternal` | `boolean`                                 | `false` |

## Frontmatter

ページ上では、`aliases` と `redirect` は **古い** パスを指します。それぞれが
現在のページパスを指すリダイレクトページを出します。

```md
---
title: Guide
aliases:
  - /old
  - /legacy
redirect: /retired
---
```

`/old`、`/legacy`、`/retired` はそれぞれ `old/index.html`、
`legacy/index.html`、`retired/index.html` になり、`/guide` へ refresh します。

リダイレクトは Markdown 構文ではありません。フェンスやコードスパン内のテキストは
無視されます。読むのは frontmatter と設定マップだけです。

## 安全性

宛先は同一オリジンパスである必要があります。`/` で始まり、`//` で始まってはいけません。
`javascript:`、`data:`、`https://evil` のような絶対 URL は、
`allowExternal` を設定しない限り無視されます。その場合でも、受け付けるのは `http://` と
`https://` の宛先だけです。

許可された宛先でもマークアップ文字を含むものは、refresh URL、canonical href、
見えるリンクで HTML エスケープされます。

実際に公開されたページと一致するソースはスキップされるので、リダイレクトが
コンテンツを上書きできません。

## 末尾スラッシュと重複

`/old` と `/old/` は、末尾スラッシュを取り除いたあと（`/` 自体を除く）同じソースです。
宛先も同じように正規化されます。

2 つの規則が正規化後のソースを共有するとき、**最後の規則が勝ちます**。frontmatter
エイリアスと `redirect` が先に適用されます。設定の `map` は最後に適用されるので、
明示的なマップエントリは同じ古いパスのページエイリアスを上書きします。

## ホストファイル

`netlify: true` を設定すると `_redirects` ファイル
（`/old /guide 301`）も書きます。`headers: true` を設定すると、ソースごとに
`Location` 行付きの `_headers` を書きます。`json: true` を設定すると `redirects.json` を書きます。
HTML ページはいまも書かれます。

## 下書き

下書き、非公開、予約公開ページはこの機能の対象外です。
後続の下書きオプションが、未公開ページのエイリアスを省くかもしれません。

## 関連

- [サイト生成](./site-generation.md)
- [組み込み機能の概要](../built-in-features.md)
