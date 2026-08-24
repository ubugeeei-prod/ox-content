---
title: 下書き / 非公開 / 予約公開
description: 本番 HTML・ナビ・検索・sitemap 向けのオプトイン公開状態。
---

# 下書き / 非公開 / 予約公開

`publishState` を有効にすると、本番ビルドが frontmatter の公開フィールドを尊重します。省略または `false` では今までどおり、すべての Markdown が公開されます。

```ts
oxContent({
  publishState: true,
});
```

オブジェクトでビルド時刻を注入できます。

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
| `now`          | ISO-8601 文字列                   | システム時計 |

dev サーバでは下書きと未到来の予約公開もプレビューできます。本番の HTML、検索、sitemap からは落ちます。

```md
---
title: 準備中
draft: true
---
```

`unlisted: true` は HTML を出しますが、ナビ・検索・sitemap から外します。`publishAt` は指定時刻まで下書き扱いです。

## 関連

- [英語版ガイド](/built-in/drafts.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
