---
title: インラインバッジ
description: 見出しの横や本文中に置く、オプトインの状態ラベル。
---

# インラインバッジ

ガイドでは、見出しの横や文中に小さな状態ラベル（beta、required、deprecated）が欲しくなることがあります。`{badge:variant}` 記法はオプトインで、既定はオフです。

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

`false` または省略はソースを変えません。`true` またはオブジェクトは組み込みの種類をオンにします。

## 書き方

形は `{badge:VARIANT}TEXT{/badge}` です。`VARIANT` は小文字で、大文字小文字を区別します。

{badge:tip}Beta{/badge} {badge:note}Note{/badge} {badge:info}Info{/badge}
{badge:warning}Warning{/badge} {badge:danger}Danger{/badge}
{badge:success}Stable{/badge} {badge:deprecated}Deprecated{/badge}
{badge:required}Required{/badge}

```md
API {badge:tip}Beta{/badge} — the `token` field is {badge:required}required{/badge}.
```

API {badge:tip}Beta{/badge} — the `token` field is {badge:required}required{/badge}.

使える種類は `tip`、`note`、`info`、`warning`、`danger`、`success`、`deprecated`、`required` です。

未知、大文字、空、または閉じていないタグはリテラルのままです。バッジ本文は HTML エスケープされます。フェンス、インデントコード、インラインコードは書き換えません。

```md
`{badge:tip}ignored{/badge}`
```

`{badge:tip}ignored{/badge}`

## 関連

- [カスタムコンテナ](./containers.md)
- [キーボードキー](./keyboard-keys.md)
- [組み込み機能の一覧](../built-in-features.md)
