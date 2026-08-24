---
title: インラインバッジ
description: 見出しや本文の横に置くオプトインの状態ラベル。
---

# インラインバッジ

見出しや文中に小さな状態ラベル（beta、必須、非推奨）を置きたいことがあります。`{badge:variant}` はオプトインで、既定はオフです。

| オプション | 型                         | 既定    |
| ---------- | -------------------------- | ------- |
| `badges`   | `boolean` / `BadgeOptions` | `false` |

```ts
oxContent({
  badges: true,
});
```

省略または `false` ではソースはそのままです。`true` またはオブジェクトで組み込みバリアントが使えます。

形式は `{badge:VARIANT}TEXT{/badge}` です。`VARIANT` は小文字で、大文字小文字を区別します。

```md
{badge:tip}Beta{/badge} {badge:warning}注意{/badge} {badge:deprecated}非推奨{/badge}
```

使える型は `tip`, `note`, `info`, `warning`, `danger`, `success`, `deprecated`, `required` です。

## 関連

- [英語版ガイド](/built-in/badges.md)
