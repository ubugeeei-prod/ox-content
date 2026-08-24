---
title: Sitemap / robots / llms.txt
description: 生成 HTML の横に書くオプトインのクロール用マニフェスト。
---

# Sitemap / robots / llms.txt

`siteMaps` を有効にし、`ssg.siteUrl` があると、SSG ビルドが生成 HTML の横にマニフェストを書きます。

- `sitemap.xml` — 公開ページ URL（整列済み）
- `robots.txt` — allow-all と Sitemap 行
- `llms.txt` — サイト名、説明、ページ一覧

省略または `false` ではオフです。既存サイトは変わりません。

```ts
oxContent({
  siteMaps: true,
  ssg: {
    siteUrl: "https://example.com",
  },
});
```

オブジェクトで個別に切れます。

```ts
oxContent({
  siteMaps: {
    robots: false,
    llms: false,
  },
});
```

`siteUrl` が無いとファイルは書かず、警告が出ます。下書きや非公開ページは `publishState` がオンなら落ちます。

## 関連

- [英語版ガイド](/built-in/site-maps.md)
- [下書き / 非公開 / 予約公開](./drafts.md)
