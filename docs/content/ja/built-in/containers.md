---
title: カスタムコンテナ
description: オプトインの ::: tip / ::: warning / ::: details。
---

# カスタムコンテナ

`::: type` コンテナはオプトインです。GitHub 風の `> [!NOTE]` は既定のレンダラ経路にあり、このオプションは不要です。

| オプション   | 型                             | 既定    |
| ------------ | ------------------------------ | ------- |
| `containers` | `boolean` / `ContainerOptions` | `false` |

```ts
oxContent({
  containers: true,
});
```

組み込み型は `tip`, `note`, `info`, `important`, `warning`, `danger`, `caution`, `details` です。

```md
::: tip
先にプラグインを入れてください。
:::

::: warning 注意
描画されるマークアップが変わります。
:::

::: details{open}
追加の文脈。
:::
```

省略または `false` では `:::` はリテラルのままです。`true` またはオブジェクトでオンです。

## 関連

- [英語版ガイド](/built-in/containers.md)
- [カード](./cards.md)
