---
title: 定義リスト
description: オプトインの Term / : definition 記法を意味的な用語リストとして描画します。
---

# 定義リスト

用語集、オプション参照、プロトコルのフィールドでは、用語の横に 1 つ以上の
定義を置きたいことがあります。表では重すぎます。PHP Markdown Extra /
mdBook の短い記法はオプトインで、既定はオフです。既存の pandoc 風ソースは、
サイトがオンにするまで普通の段落やリストのままです。

| オプション        | 型                                  | 既定    |
| ----------------- | ----------------------------------- | ------- |
| `definitionLists` | `boolean` / `DefinitionListOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      definitionLists: true,
    }),
  ],
};
```

`false` または省略はソースを変えません。`true` またはオブジェクトは変換を
オンにします。クライアント JavaScript はありません。

## 書き方

用語を 1 行で書き、そのあと `: `（コロンと空白）で始まる定義を 1 つ以上
続けます。

HTTP
: Hypertext Transfer Protocol
: Web で使うリクエスト / レスポンスプロトコルの名前でもあります。

TLS
: Transport Layer Security

```md
HTTP
: Hypertext Transfer Protocol
: Web で使うリクエスト / レスポンスプロトコルの名前でもあります。

TLS
: Transport Layer Security
```

レンダラはテーマ向けの安定した class 付きの意味的なリストを出します。

```html
<dl class="ox-definition-list">
  <dt>…</dt>
  <dd>…</dd>
</dl>
```

用語と定義の中のインライン Markdown はパースされます。

**Status**
: `2xx` レスポンスは **成功** です。

```md
**Status**
: `2xx` レスポンスは **成功** です。
```

用語と最初の定義のあいだの空行も受け付けます。複数の用語が続く定義を
共有できます。不正または曖昧な形 — 単独の `: definition`、リスト項目の
あとの `: `、折り返した段落 — は普通の段落やリストのままです。

フェンス、インデントコード、インラインコード、HTML コメント、生の
`code` / `pre` / `script` / `style` は書き換えません。

## オプション

```ts
oxContent({
  definitionLists: {
    enabled: true,
  },
});
```

| フィールド | 型        | 既定   |
| ---------- | --------- | ------ |
| `enabled`  | `boolean` | `true` |

## 関連

- [構文拡張](./syntax-extensions.md)
- [キーボードキー](./keyboard-keys.md)
- [組み込み機能の一覧](../built-in-features.md)
