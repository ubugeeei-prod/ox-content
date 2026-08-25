---
title: カード
description: 概要ページ向けの、オプトインの card / link-card / card-grid。
---

# カード

card、link-card、card-grid ブロックはオプトインです。オフのとき、`:::` 形式はリテラルのままです。

| オプション | 型                        | 既定    |
| ---------- | ------------------------- | ------- |
| `cards`    | `boolean` / `CardOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      cards: true,
    }),
  ],
};
```

## カード

`::: card` は `<article class="ox-card">` になります。内側の Markdown はパースされます。タイトルは先頭の見出し、または `::: card[Title]` からです。

```md
::: card

### Install

Copy the package and run the CLI.
:::
```

このサイトは `cards` をオンにしているので、次のブロックはライブのカードです。

::: card

### Install

Copy the package and run the CLI.
:::

## リンクカード

`::: link-card[Title]{HREF}` は `<a class="ox-link-card" href="...">` になります。タイトルと説明はエスケープされます。`javascript:`、`data:`、`vbscript:`、プロトコル相対の `//` href は拒否します。ブロック自体は描画されますが、その href を持つアンカーは出しません。

```md
::: link-card[Guide]{/getting-started}
Short description
:::
```

::: link-card[Guide]{/getting-started}
Short description
:::

## カードグリッド

`::: card-grid` は内側のカードを `<div class="ox-card-grid">` で包みます。単独のカードとリンクカードはグリッドの外でも動きます。

```md
::: card-grid
::: card

### Install

Copy the package and run the CLI.
:::
::: link-card[Guide]{/getting-started}
Short description
:::
:::
```

::: card-grid
::: card

### Install

Copy the package and run the CLI.
:::
::: link-card[Guide]{/getting-started}
Short description
:::
:::

閉じていないブロックはリテラルのままで、ファイルの残りを飲み込みません。フェンス、インライン、インデントコードの中のマーカーはそのままです。

## 関連

- [カスタムコンテナ](./containers.md)
- [組み込み機能の一覧](../built-in-features.md)
