---
title: Sitemap、robots.txt、llms.txt
description: 生成 HTML の隣に書く、オプトインのクロールマニフェストです。
---

# Sitemap、robots.txt、llms.txt

`siteMaps` が有効で `ssg.siteUrl` が設定されているとき、SSG ビルドは
生成 HTML の隣にクロールマニフェストを書きます。

- `sitemap.xml` — 公開されたすべてのページ URL（ソート済み）
- `robots.txt` — すべて許可と Sitemap 行
- `llms.txt` — サイトタイトル、説明、ページ一覧

機能は、オンにするまでオフです。既存サイトは変わりません。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      siteMaps: true,
      ssg: {
        siteUrl: "https://example.com",
      },
    }),
  ],
};
```

`false` または省略するとファイルはオフのままです。`true` は既定を有効にします。オブジェクトは
機能を有効にし、設定した欄だけ上書きします。

```ts
oxContent({
  siteMaps: {
    robots: false,
    llms: false,
  },
  ssg: {
    siteUrl: "https://example.com",
  },
});
```

| オプション | 型                            | 既定    |
| ---------- | ----------------------------- | ------- |
| `siteMaps` | `boolean` / `SiteMapsOptions` | `false` |
| `robots`   | `boolean`                     | `true`  |
| `llms`     | `boolean`                     | `true`  |

機能がオンのとき、`sitemap.xml` は常に書かれます。frontmatter に
`draft: true` があるページは省かれます。[`publishState`](./drafts.md) も
有効なときは、非公開とまだ予約公開前のページも省かれます。

`ssg.siteUrl` なしで `siteMaps` を有効にすると、ファイルは書かれません。ビルドは
続き、警告を出します。

タイトルと説明はエスケープされるので、XML や
`llms.txt` から抜けられません。

## 関連

- [サイト生成](./site-generation.md)
- [組み込み機能の概要](../built-in-features.md)
