---
title: リーダークロム
description: オプトインのコピーボタン、外部リンクアイコン、先頭へ戻る操作です。
---

# リーダークロム

`ssg.readerChrome` が有効なとき、テーマ付きページは 3 つの小さな読み取り
操作を得ます。

- フェンス付きコードブロックの **コピー** ボタン
- 外向き `http(s)` リンクへのアイコンと `rel="noopener noreferrer"`
- ページをスクロールしたあとに現れる **先頭へ戻る** 操作

機能は、オンにするまでオフです。無効なページは余分なマークアップも
JavaScript も出しません。

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

`false` または省略するとクロムはオフのままです。`true` は既定を有効にします。オブジェクトは
機能を有効にし、ひとつの操作を切れます。

```ts
oxContent({
  ssg: {
    readerChrome: { copy: false },
  },
});
```

| 欄              | 既定    | 効果                                 |
| --------------- | ------- | ------------------------------------ |
| `copy`          | `true`  | フェンス付き `<pre>` ブロックのコピーボタン |
| `externalLinks` | `true`  | 外向きリンクへのアイコンと `rel`     |
| `backToTop`     | `true`  | スクロール後の先頭へ戻るボタン       |

コピーは、読者がボタンをクリックしたときにブラウザのクリップボードを使います。フェンステキストは
ビルド時にはコピーされません。

外部アイコンは相対、ハッシュ、`mailto:`、`tel:` リンクをスキップします。フェンス付きブロックや
インラインコードスパン内のリンクはそのままです。`javascript:`、`data:`、
`vbscript:` href にはライブ操作を付けません。

先頭へ戻る操作は `prefers-reduced-motion` を尊重します。bare モードは
リーダークロムを決して出しません。
