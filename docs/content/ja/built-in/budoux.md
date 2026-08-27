---
title: BudouX
description: 日本語の改行のための、ビルド時フレーズ分割。
---

# BudouX

`budoux` は Markdown 変換時にフレーズの間へ zero-width space を挿入します。日本語本文の改行機会を増やしつつ、出力は静的 HTML のままです。

この機能は既定でオフです。有効にするプロジェクトだけ、任意依存の `budoux` package を追加してください。

```sh
pnpm add -D budoux
```

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      budoux: true,
    }),
  ],
};
```

## 出力

次の日本語本文があるとします。

```md
今日はとても良い天気です。
```

描画 HTML にはフレーズ区切りが入ります。

```html
<p>今日は\u200bとても\u200b良い\u200b天気です。</p>
```

変わるのは HTML の text content だけです。生成される page module は `budoux` を import せず、browser bundle に parser は入りません。

## 保護されるマークアップ

変換は HTML tag と attribute をそのままコピーします。HTML entity は entity のまま保ち、`code`、`pre`、`script`、`style`、`textarea`、`svg`、`math` は処理しません。island の JSON payload も変更しません。リンク URL は変えず、リンク label は visible text として分割します。

通常の Markdown block にある visible text は処理対象です。段落、見出し、リスト項目、blockquote、table cell、island slot HTML が含まれます。そのため local island SSR を走らせる framework adapter は、分割済みの slot HTML を受け取ります。一方で component props と island payload data は変わりません。

## スタイル

BudouX は改行機会を作りますが、どれくらい折り返すかは CSS が決めます。Ox Content は inline style を注入しません。BudouX らしい折り返しが必要な場合は、content container や theme に次を足してください。

```css
.ox-content {
  word-break: keep-all;
  overflow-wrap: anywhere;
}
```

## オプション

| オプション  | 型                                          | 既定           |
| ----------- | ------------------------------------------- | -------------- |
| `enabled`   | `boolean`                                   | `true`         |
| `language`  | `"ja"` / `"zh-hans"` / `"zh-hant"` / `"th"` | `"ja"`         |
| `separator` | `string`                                    | `"\u200b"`     |
| `parser`    | `{ parse(text: string): string[] }`         | default parser |

別の BudouX default parser を使う場合は `language` を指定します。

```ts
oxContent({
  budoux: {
    language: "ja",
  },
});
```

サイト側の custom model がある場合は `parser` を渡します。このとき Ox Content は任意依存の `budoux` package を import しません。

```ts
oxContent({
  budoux: {
    parser: customParser,
  },
});
```

## 関連

- [構文拡張](./syntax-extensions.md) - ほかのオプトイン Markdown 機能。
- [CJK Emphasis](/examples/cjk-emphasis.md) - CJK 句読点付近の強調パース。
