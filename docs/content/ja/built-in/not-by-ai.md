---
title: NotByAI バッジ
description: 公式 Not By AI バッジを使う、オプトインの静的な執筆開示。
---

# NotByAI バッジ

ドキュメントサイトでは、ページが人の手で書かれたことを示す必要があることがあります。
`<NotByAI />` はオプトインの執筆開示バッジです。状態ラベルではありません。

beta や deprecated などの状態ラベルは
[インラインバッジ](./badges.md)（`{badge:tip}`）を使います。この機能は公式の
[Not By AI](https://notbyai.fyi) アートワークだけを静的 HTML として出します。

| オプション | 型                           | 既定    |
| ---------- | ---------------------------- | ------- |
| `notByAi`  | `boolean` / `NotByAiOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      notByAi: true,
    }),
  ],
};
```

`false` または省略はソースを変えません。`true` またはオブジェクトでバッジを
オンにします。クライアント JavaScript も hydration もありません。

## 書き方

正式な形は `<NotByAI />` です。空白なしの自己閉じも受け付けます。

```md
このページは人が書いています。

<NotByAI />
```

このページは人が書いています。

<NotByAI />

既定のアクセシブルラベルは `Written by human, not by AI` です。既定のリンクは
`https://notbyai.fyi` です。どちらも上書きできます。

```ts
oxContent({
  notByAi: {
    label: "ドキュメントチームが執筆",
    href: "https://example.com/authorship",
  },
});
```

安全でない `href`（`javascript:`、`data:`、プロトコル相対 URL）は公式 URL に
戻します。label と href は HTML エスケープされます。

フェンス、インデントコード、インラインコード、HTML コメントは書き換えません。
不正または閉じていないタグはリテラルのままです。`.md` と `.mdx` は同じ静的
マークアップを出します。`NotByAI` は予約済みなので、MDX は island にしません。

## ライト / ダークのアートワーク

バッジは公式のライト / ダーク SVG を含みます。組み込み SSG の CSS は
`prefers-color-scheme` とホストのカラースキームクラス（`[data-theme]`、
`.dark`、`.light`）で切り替えます。カスタムホストは同じシートを import します。

```css
@import "@ox-content/vite-plugin/styles/not-by-ai.css";
```

[コンポーネント CSS](./component-styles.md) を見てください。

## サイト側プリプロセッサからの移行

サイトがすでに `<NotByAI />` をプレースホルダに置き換え、描画後にベンダー
済み SVG を差し込んでいる場合（ryoppippi.com の形）、`notByAi` をオンにして
そのプリプロセッサ、SVG import、描画後の置換、保護テストを削除できます。

## 関連

- [インラインバッジ](./badges.md) — beta や required などの状態ラベル
- [Markdown の土台](./markdown.md)
- [組み込み機能の一覧](../built-in-features.md)
