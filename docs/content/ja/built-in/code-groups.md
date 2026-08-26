---
title: コードグループ
description: 既存の no-JS タブウィジェットを再利用する、オプトインの VitePress 風 ::: code-group。
---

# コードグループ

隣り合うコード例は、手書きの `<tabs>` なしで、既存のアクセシブルなタブ UI を共有できます。機能はオプトインです。省略または `false` なら `::: code-group` は通常の Markdown / カスタムコンテナ経路のままです。

| オプション   | 型                             | 既定    |
| ------------ | ------------------------------ | ------- |
| `codeGroups` | `boolean` / `CodeGroupOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      codeGroups: true,
    }),
  ],
};
```

`{}` を渡しても既定でオンになります。書き換えは静的です。no-JS の `ox-tabs` ウィジェットと `<noscript>` の details フォールバックを再利用します。追加のクライアント JavaScript は出荷しません。

## 書き方

### 以前（`<tabs>`）

```html
<tabs>
  <tab label="config.js">
    <pre><code>export default {}</code></pre>
  </tab>
  <tab label="config.ts">
    <pre><code>export default {}</code></pre>
  </tab>
</tabs>
```

### 以後（`::: code-group`）

::: code-group

```js [config.js]
export default {};
```

```ts [config.ts]
export default {};
```

:::

````md
::: code-group

```js [config.js]
export default {};
```

```ts [config.ts]
export default {};
```

:::
````

タブタイトルは ` ```ts [label] ` か、`title="config.ts"` のようなフェンス meta から取ります。どちらも無いときは言語識別子（`js`、`ts`）です。言語の無いフェンスは `Tab 1`、`Tab 2` のように落ちます。

閉じていない `::: code-group` はリテラルのままで、ファイルの残りを飲み込みません。未知または不正なグループメタデータは普通のフェンスに落ち、transform の警告を出します。フェンスやインデントコードの中に書いたグループはそのままです。

カスタムコンテナもオンのとき、`code-group` はこの機能が処理します。未知のコンテナ種類としては扱いません。

既存の `<tabs>` と `<pm>` は変わりません。

## 関連

- [コードブロック](./code-blocks.md)
- [埋め込み](./embeds.md#タブ)
- [カスタムコンテナ](./containers.md)
- [組み込み機能の一覧](../built-in-features.md)
