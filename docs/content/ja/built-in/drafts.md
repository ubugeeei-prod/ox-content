---
title: 下書き / 非公開 / 予約公開
description: 本番 HTML、ナビ、検索、sitemap 向けの、オプトインの frontmatter 公開状態。
---

# 下書き / 非公開 / 予約公開

`publishState` を有効にすると、本番ビルドは frontmatter の公開フィールドを守ります。省略または `false` は今までの挙動のままです。すべての Markdown ページが公開されます。

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

`false` または省略はフィルタをオフのままにします。`true` は既定でオンです。オブジェクトを渡すと機能はオンになり、ビルド時の時計を注入できます。

```ts
oxContent({
  publishState: {
    now: "2026-08-24T00:00:00Z",
  },
});
```

| オプション     | 型                                | 既定           |
| -------------- | --------------------------------- | -------------- |
| `publishState` | `boolean` / `PublishStateOptions` | `false`        |
| `now`          | `string`（ISO-8601）              | システムの時計 |

開発サーバは下書きと、まだ公開時刻になっていないページを見せ続けます。プレビューできるようにするためです。本番 HTML、検索、sitemap からはそれらのページが外れます。

## Frontmatter

```md
---
title: Work in progress
draft: true
---
```

| フィールド       | 本番での結果                                    |
| ---------------- | ----------------------------------------------- |
| `draft: true`    | HTML、ナビ、検索、sitemap のいずれも出さない    |
| `unlisted: true` | HTML は書き出す。ナビ、検索、sitemap からは外す |
| `scheduled`      | その瞬間まで未公開                              |
| `date`           | 値が有効なタイムスタンプなら `scheduled` と同じ |
| `expiry`         | その瞬間以降は未公開                            |

両方あるときは `scheduled` が `date` より優先です。`draft` と `unlisted` は JSON の `true` だけが真です。

## タイムゾーンと不正な日付

ナイーブなタイムスタンプ（`2026-08-24` と `2026-08-24T12:00:00`）は UTC です。`+09:00` や `Z` のようなオフセットは尊重します。

不正な `scheduled` や `expiry` はページを未公開にします。不正な `date` は無視します。`date` は表示用メタデータとしても使うからです。不正な `now` オプションはシステムの時計に戻します。

## 関連

- [サイト生成](./site-generation.md)
- [検索](./search.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [Markdown ソースの併記](./markdown-source.md)
- [組み込み機能の一覧](../built-in-features.md)
