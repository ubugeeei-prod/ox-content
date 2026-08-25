---
title: Sitemap / robots / llms.txt
description: 生成 HTML の横に書き出す、オプトインのクロール用マニフェスト。
---

# Sitemap / robots / llms.txt

`siteMaps` を有効にし、`ssg.siteUrl` を設定すると、SSG ビルドは生成 HTML の横にクロール用マニフェストを書き出します。

- `sitemap.xml` — 公開ページの URL をソートして列挙
- `robots.txt` — すべて許可し、Sitemap 行を付ける
- `llms.txt` — サイトタイトル、説明、ページ一覧

機能は自分でオンにするまでオフです。既存サイトはそのままです。

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

`false` または省略はファイルを出しません。`true` は既定でオンです。オブジェクトを渡すと機能はオンになり、設定したフィールドだけ上書きします。

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

機能がオンなら `sitemap.xml` は必ず書き出します。frontmatter で `draft: true` のページは外れます。[`publishState`](./drafts.md) もオンなら、非公開（unlisted）とまだ公開時刻になっていないページも外れます。

`siteMaps` をオンにしても `ssg.siteUrl` がなければ、ファイルは書き出しません。ビルドは続き、警告を出します。

タイトルと説明はエスケープされるので、XML や `llms.txt` の外へは出られません。

## 関連

- [サイト生成](./site-generation.md)
- [組み込み機能の一覧](../built-in-features.md)
