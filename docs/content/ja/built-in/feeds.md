---
title: RSS / Atom / JSON フィード
description: 生成 HTML の横に書き出す、オプトインのコレクションフィード。
---

# RSS / Atom / JSON フィード

`feeds` を有効にし、`ssg.siteUrl` を設定すると、SSG ビルドは名前付きコレクションから機械可読のフィードを書き出します。

- `feed.xml` — RSS 2.0
- `atom.xml` — Atom 1.0
- `feed.json` — JSON Feed 1.1

機能は自分でオンにするまでオフです。既存サイトはそのままです。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      feeds: true,
      ssg: {
        siteUrl: "https://example.com",
      },
    }),
  ],
};
```

`false` または省略はファイルを出しません。`true` は既定でオンです。3 形式すべて、`content` コレクション（なければ設定された最初のコレクション）、20 件制限です。オブジェクトを渡すと機能はオンになり、設定したフィールドだけ上書きします。

```ts
oxContent({
  feeds: {
    formats: ["rss", "json"],
    collection: "blog",
    limit: 10,
    path: "/feeds",
  },
  ssg: {
    siteUrl: "https://example.com",
  },
});
```

| オプション   | 型                              | 既定                                  |
| ------------ | ------------------------------- | ------------------------------------- |
| `feeds`      | `boolean` / `FeedsOptions`      | `false`                               |
| `formats`    | `("rss" \| "atom" \| "json")[]` | `["rss", "atom", "json"]`             |
| `collection` | `string`                        | `content`、なければ最初のコレクション |
| `limit`      | `number`                        | `20`                                  |
| `path`       | `string`                        | `/`（サイトルート）                   |

`path` は生成ファイルのサイト相対ディレクトリです。`/feeds` なら `feeds/feed.xml`、`feeds/atom.xml`、`feeds/feed.json` を書き出します。

項目は新しい順です。ソートキーは frontmatter の `date`、なければ `lastUpdated` です。`draft: true` のエントリは外れます。

`feeds` をオンにしても `ssg.siteUrl` がなければ、ファイルは書き出しません。ビルドは続き、警告を出します。

タイトルと説明はエスケープされるので、XML や JSON の外へは出られません。

## ブログ索引の項目

[ブログ](./blog.md) の `blog.feeds` で集めた外部投稿は索引にだけ載ります。
生成ファイルには入りません。このリリースに取り込みスイッチはありません。

## 関連

- [コレクション](./collections.md)
- [ブログ](./blog.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [サイト生成](./site-generation.md)
- [組み込み機能の一覧](../built-in-features.md)
