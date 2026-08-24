---
title: 数式
description: エスケープされた静的マークアップによる、オプトインのインライン `$…$` とディスプレイ `$$…$$` 数式です。
---

# 数式

数式の執筆はオプトインです。サイトが transform を有効にするまで、普通の `$` 文字はリテラルのままです。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      math: true,
    }),
  ],
};
```

`true` と `{}` はどちらも既定を有効にします。オプションを省略するか `false` を渡すと
`$` と `$$` はそのままです。

## 区切り

| 形式       | ソース               | 結果                                     |
| ---------- | -------------------- | ---------------------------------------- |
| インライン | `$E=mc^2$`           | `<span class="ox-math ox-math-inline">…` |
| ディスプレイ | `$$E = mc^2$$`     | `<div class="ox-math ox-math-block">…`   |
| インライン | `Before $$x$$ after` | `<span class="ox-math ox-math-inline">…` |

ディスプレイ区切りは、段落全体を占めるときだけブロックになります。
囲まれた `$$…$$` はインラインのままなので、Markdown が `<p>` の中に `<div>` を出しません。

transform はアクセスしやすい静的 MathML を出します。TeX は、パイプラインの他の箇所と同じ HTML
エスケープのあと `<mtext>` に置かれるので、`<script>`、引用符、
属性風の断片が raw HTML になることはありません。

## 描画例

インライン: 等式は $E=mc^2$ です。

ディスプレイ:

$$E = mc^2$$

フェンス付きコード、インデントコード、インラインコードは書き換えません。

````md
```
$E=mc^2$
```

Use `$E=mc^2$` in prose. Currency stays literal: `$5`, `$5.00`, `$5-$10`, `US$`.
````

閉じられていない `$` または `$$` はリテラルのままで、ファイルの残りを消費しません。
数式がオンでリテラルのドル記号が必要なときは `\$` と書いてください。

GitHub 風の `> [!NOTE]` コールアウトや他の既存構文は変わりません。

## 関連

- [構文拡張](./syntax-extensions.md) — 他のオプトイン Markdown 構文。
- [組み込み機能の概要](../built-in-features.md)
