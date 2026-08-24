---
title: 下書き、非公開、予約公開ページ
description: 本番 HTML、ナビ、検索、サイトマップ向けの、オプトイン frontmatter 公開状態です。
---

# 下書き、非公開、予約公開ページ

`publishState` が有効なとき、本番ビルドは frontmatter の公開
欄を尊重します。省略または `false` はいまの振る舞いのままです。すべての Markdown ページが
公開されます。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      publishState: true,
    }),
  ],
};
```

`false` または省略するとフィルタはオフのままです。`true` は既定を有効にします。オブジェクトは
機能を有効にし、ビルド時の時計を注入できます。

```ts
oxContent({
  publishState: {
    now: "2026-08-24T00:00:00Z",
  },
});
```

| オプション     | 型                                | 既定         |
| -------------- | --------------------------------- | ------------ |
| `publishState` | `boolean` / `PublishStateOptions` | `false`      |
| `now`          | `string`（ISO-8601）              | システム時計 |

dev サーバーは下書きとまだ予約公開前のページを見えるままにするので、
プレビューできます。本番 HTML、検索、サイトマップはそれらのページを省きます。

## Frontmatter

```md
---
title: Work in progress
draft: true
---
```

| 欄               | 本番の結果                                          |
| ---------------- | --------------------------------------------------- |
| `draft: true`    | HTML、ナビ、検索、サイトマップなし                  |
| `unlisted: true` | HTML は書く。ナビ、検索、サイトマップからは省く     |
| `scheduled`      | その瞬間まで非公開                                  |
| `date`           | 値が有効なタイムスタンプなら `scheduled` と同じ     |
| `expiry`         | その瞬間のあと非公開                                |

両方あるとき `scheduled` が `date` に勝ちます。`draft` または `unlisted` として数えるのは JSON の `true` だけです。

## タイムゾーンと不正な日付

素朴なタイムスタンプ（`2026-08-24` と `2026-08-24T12:00:00`）は UTC です。`+09:00` や
`Z` のようなオフセットは尊重されます。

不正な `scheduled` または `expiry` 値はページを非公開にします。不正な `date`
値は無視されます。`date` は表示メタデータとしても使われるからです。不正な
`now` オプションはシステム時計へフォールバックします。

## 関連

- [サイト生成](./site-generation.md)
- [検索](./search.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [組み込み機能の概要](../built-in-features.md)
