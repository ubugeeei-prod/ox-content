---
title: インラインバッジ
description: 見出しの横や本文に置く、オプトインのステータスラベルです。
---

# インラインバッジ

ガイドでは、見出しの横や文中に小さなステータスラベルが必要になることがあります —
ベータ、必須、非推奨。`{badge:variant}` マークアップはオプトインで、既定はオフです。

| オプション | 型                         | 既定    |
| ---------- | -------------------------- | ------- |
| `badges`   | `boolean` / `BadgeOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      badges: true,
    }),
  ],
};
```

`false` または省略するとソースは変わりません。`true` またはオブジェクトで
組み込みバリアントが有効になります。

## 執筆

形式は `{badge:VARIANT}TEXT{/badge}` です。`VARIANT` は小文字で、
大文字小文字を区別します。

{badge:tip}ベータ{/badge} {badge:note}メモ{/badge} {badge:info}情報{/badge}
{badge:warning}警告{/badge} {badge:danger}危険{/badge}
{badge:success}安定{/badge} {badge:deprecated}非推奨{/badge}
{badge:required}必須{/badge}

```md
API {badge:tip}Beta{/badge} — the `token` field is {badge:required}required{/badge}.
```

API {badge:tip}ベータ{/badge} — `token` 欄は {badge:required}必須{/badge} です。

使えるバリアント: `tip`、`note`、`info`、`warning`、`danger`、`success`、
`deprecated`、`required`。

未知、大文字、空、または閉じられていないタグはリテラルのままです。バッジテキストは
HTML エスケープされます。フェンス、インデント、インラインコードは書き換えません。

```md
`{badge:tip}ignored{/badge}`
```

`{badge:tip}ignored{/badge}`

## 関連

- [カスタムコンテナ](./containers.md)
- [組み込み機能の概要](../built-in-features.md)
