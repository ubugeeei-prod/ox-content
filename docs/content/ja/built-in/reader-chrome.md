---
title: リーダー chrome
description: オプトインのコピーボタン、外部リンクアイコン、先頭へ戻る操作。
---

# リーダー chrome

`ssg.readerChrome` を有効にすると、テーマ付きページに小さな読書用コントロールが 3 つ付きます。

- フェンス付きコードブロックの **Copy** ボタン
- 外部 `http(s)` リンクへのアイコンと `rel="noopener noreferrer"`
- スクロール後に現れる **先頭へ戻る** 操作

機能は自分でオンにするまでオフです。オフのページには余分なマークアップも JavaScript も出ません。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        readerChrome: true,
      },
    }),
  ],
};
```

`false` または省略は chrome をオフのままにします。`true` は既定でオンです。オブジェクトを渡すと機能はオンになり、個別の操作だけ切れます。

```ts
oxContent({
  ssg: {
    readerChrome: { copy: false },
  },
});
```

| フィールド      | 既定   | 効果                                |
| --------------- | ------ | ----------------------------------- |
| `copy`          | `true` | フェンス付き `<pre>` にコピーボタン |
| `externalLinks` | `true` | 外部リンクにアイコンと `rel`        |
| `backToTop`     | `true` | スクロール後に先頭へ戻るボタン      |

コピーは読者がボタンを押したときにブラウザのクリップボードを使います。フェンス本文はビルド時にはコピーしません。ページ全体の Copy as Markdown は別のオプトイン [`ssg.markdownSource.copy`](./markdown-source.md) です。

外部リンクアイコンは相対、ハッシュ、`mailto:`、`tel:` を飛ばします。フェンス内やインラインコード内のリンクはそのままです。`javascript:`、`data:`、`vbscript:` の href には生きた操作を付けません。

先頭へ戻る操作は `prefers-reduced-motion` を尊重します。エントリページでは出しません。bare モードはリーダー chrome を一切出しません。
