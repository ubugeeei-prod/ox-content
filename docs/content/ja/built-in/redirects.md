---
title: リダイレクトとエイリアス
description: frontmatter のエイリアスと書き換えマップから作る、オプトインの静的 HTML リダイレクト。
aliases:
  - /ja/built-in/aliases
---

# リダイレクトとエイリアス

`redirects` を有効にすると、SSG ビルドは既定で古いパスごとに小さな静的 HTML ページを書き出します。ページは meta refresh と canonical リンクを使うので、リネーム後も inbound URL が動きます。どの静的ホストでも動きます。

このページ自身も `aliases: [/ja/built-in/aliases]` を宣言しているので、ドキュメントサイトは古いパス向けのライブなリダイレクトを載せます。

機能は自分でオンにするまでオフです。既存サイトはそのままです。

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

`false` または省略は何も書き出しません。`true` は既定でオンです。オブジェクトを渡すと機能はオンになり、設定したフィールドだけ上書きします。

```ts
oxContent({
  redirects: {
    map: {
      "/old-guide": "/guide",
    },
  },
});
```

`{ "/old-guide": "/guide" }` のようなパスマップをオプションオブジェクトの代わりに渡すと、そのマップ付きで機能がオンになります。

| オプション      | 型                                          | 既定    |
| --------------- | ------------------------------------------- | ------- |
| `redirects`     | `boolean` / パスマップ / `RedirectsOptions` | `false` |
| `map`           | `Record<string, string>`                    | `{}`    |
| `provider`      | `"netlify"` / `"cloudflare"`                | 検出    |
| `headers`       | `boolean`                                   | `false` |
| `json`          | `boolean`                                   | `false` |
| `html`          | `boolean`                                   | `true`  |
| `allowExternal` | `boolean`                                   | `false` |

## Frontmatter

ページ上の `aliases` と `redirect` は **古い** パスを指します。それぞれ、現在のページパスへ向かうリダイレクトページを出します。

```md
---
title: Guide
aliases:
  - /old
  - /legacy
redirect: /retired
---
```

`/old`、`/legacy`、`/retired` はそれぞれ `old/index.html`、`legacy/index.html`、`retired/index.html` になり、`/guide` へ refresh します。

リダイレクトは Markdown 構文ではありません。フェンスやコードスパンの中のテキストは無視します。読むのは frontmatter と設定マップだけです。

## 安全性

行き先は同一オリジンのパスである必要があります。`/` で始まり、`//` で始まってはいけません。`javascript:`、`data:`、`https://evil` のような絶対 URL は、`allowExternal` を設定しない限り無視します。設定しても、受け付けるのは `http://` と `https://` の行き先だけです。

許可された行き先にマークアップ文字があっても、refresh URL、canonical href、見えるリンクでは HTML エスケープされます。

本物の公開ページと一致するソースは飛ばします。リダイレクトがコンテンツを上書きできないようにするためです。

## 末尾スラッシュと重なり

`/old` と `/old/` は、末尾スラッシュを除いたあと同一のソースです（`/` 自身は除く）。行き先も同じように正規化します。

正規化したソースを 2 つの規則が共有するとき、**最後の規則が勝ちます**。frontmatter のエイリアスと `redirect` を先に適用し、設定の `map` を最後に適用するので、同じ古いパスでは明示的なマップ項目がページエイリアスを上書きします。

## ホスト用ファイル

`provider: "netlify"` または `provider: "cloudflare"` で `_redirects` ファイル（`/old /guide 301`）も書き出します。どちらのホストも、いまは同じ本文です。`headers: true` でソースごとの `Location` 行を持つ `_headers` を書き出します。`json: true` で `redirects.json` を書き出します。HTML のフォールバックページは provider 選択とは独立しており、既定ではオンです。普通のパスのリダイレクト出力をホスト用 manifest だけにしたいときは `html: false` を設定します。

`provider` を省略すると、CI の環境変数からホストを検出します。

- `CF_PAGES=1` または `WORKERS_CI=1` → Cloudflare
- `NETLIFY=true` → Netlify

明示した `provider` は常に勝ちます。ローカルビルドと GitHub Actions でも同じです。Cloudflare と Netlify の変数が同時に付いているときは警告し、ホストを黙って選ばず `_redirects` を出しません。一致がなければ `_redirects` は出しません（これまでの既定と同じです）。

Cloudflare Workers の `_redirects` は静的アセットの応答にだけ効きます。Worker コードが処理するリクエストには適用されません。

ソースに `*` が含まれる場合、それは Netlify や Cloudflare Pages 向けのホスト規則の構文であり、リテラルな URL セグメントではありません。該当出力がオンなら `_redirects`、`_headers`、`redirects.json` には残しますが、`talks*/index.html` のような静的 HTML ファイルは書き出しません。

```ts
oxContent({
  redirects: {
    map: {
      "/talks*": "/works/talks",
      "/old-guide": "/guide",
    },
    provider: "netlify",
    html: false,
  },
});
```

このマップは両方の規則を `_redirects` に書き、HTML リダイレクトページは出しません。`html: false` を外すと、`/old-guide` 向けの HTML ページも書きます。

## 2.x からの移行

3.0 では `redirects.netlify` を削除しました。`netlify: true` は `provider: "netlify"` に置き換えるか、CI 環境にホストを選ばせるなら `provider` を省略してください。

## 下書き

下書き、非公開、予約公開のページはこの機能の対象外です。将来の下書きオプションで、未公開ページのエイリアスを外すことがあります。

## 関連

- [サイト生成](./site-generation.md)
- [組み込み機能の一覧](../built-in-features.md)
