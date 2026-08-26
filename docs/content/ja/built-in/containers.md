---
title: カスタムコンテナ
description: ドキュメントのコールアウト向け、オプトインの ::: tip / ::: warning / ::: details。
---

# カスタムコンテナ

`::: type` コンテナはオプトインです。GitHub 風の `> [!NOTE]` コールアウトは既定のレンダラ経路のままなので、このオプションは不要です。

| オプション   | 型                             | 既定    |
| ------------ | ------------------------------ | ------- |
| `containers` | `boolean` / `ContainerOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      containers: true,
    }),
  ],
};
```

## 組み込みの種類

`tip`、`note`、`info`、`important`、`warning`、`danger`、`caution`、`details` です。

```md
::: tip
Install the plugin first.
:::

::: warning Watch out
This changes rendered markup.
:::

::: details{open}
Optional extra context.
:::
```

タイトルは `::: tip Title` または `::: tip[Title]` と書けます。属性は `#id`、`.class`、そして `details` の真偽フラグ `open` を受け付けます。

## 独自の種類

```ts
oxContent({
  containers: {
    types: {
      cli: { title: "CLI" },
      spoiler: { title: "Spoiler", tag: "details" },
    },
  },
});
```

種類名は ASCII 識別子である必要があります。敵意のある名前と属性は捨てられます。

## 関連

- [コードグループ](./code-groups.md)
- [構文拡張](./syntax-extensions.md)
- [組み込み機能の一覧](../built-in-features.md)
