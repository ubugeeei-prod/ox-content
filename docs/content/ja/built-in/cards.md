---
title: カード
description: オプトインの card / link-card / card-grid。
---

# カード

カード系ブロックはオプトインです。オフのとき `:::` はそのまま残ります。

| オプション | 型                        | 既定    |
| ---------- | ------------------------- | ------- |
| `cards`    | `boolean` / `CardOptions` | `false` |

```ts
oxContent({
  cards: true,
});
```

`::: card` は `<article class="ox-card">` になります。内側は Markdown です。タイトルは先頭の見出し、または `::: card[Title]` です。

```md
::: card

### インストール

パッケージを入れて CLI を実行します。
:::
```

`::: link-card` はリンク付きカード、`::: card-grid` はグリッドです。

## 関連

- [英語版ガイド](/built-in/cards.md)
- [カスタムコンテナ](./containers.md)
