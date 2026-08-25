---
title: 手順リスト
description: チュートリアル向けに番号付きリストを整える、オプトインの ::: steps。
---

# 手順リスト

チュートリアルの手順は、普通の番号付きリストと見た目を分けておきます。機能はオプトインです。省略または `false` なら `::: steps` はリテラルのままです。

| オプション | 型                         | 既定    |
| ---------- | -------------------------- | ------- |
| `steps`    | `boolean` / `StepsOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      steps: true,
    }),
  ],
};
```

`{}` を渡しても既定でオンになります。

## 書き方

番号付きリストを `::: steps` で囲みます。ネストした Markdown（フェンス、強調、入れ子リスト）は各項目の中でそのまま描画されます。

::: steps

1. Install the CLI

   ```sh
   npm i -g ox-content
   ```

2. Run **build**

:::

````md
::: steps

1. Install the CLI

   ```sh
   npm i -g ox-content
   ```

2. Run **build**

:::
````

有効にすると、ラッパーは `<div class="ox-steps">` になり、各項目は `<ol class="ox-steps__list">` と `<li class="ox-steps__item">` になります。

閉じていない `::: steps` はリテラルのままで、ファイルの残りを飲み込みません。ラッパーの外にある普通の `1. foo` リストはそのままです。

カスタムコンテナもオンのとき、`steps` はこの機能が処理します。未知のコンテナ種類としては扱いません。

## 関連

- [カスタムコンテナ](./containers.md)
- [組み込み機能の一覧](../built-in-features.md)
