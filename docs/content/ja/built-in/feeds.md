---
title: RSS、Atom、JSON フィード
description: 生成 HTML の隣に書く、オプトインのコレクションフィードです。
---

# RSS、Atom、JSON フィード

`feeds` が有効で `ssg.siteUrl` が設定されているとき、SSG ビルドは
名前付きコレクションから機械可読フィードを書きます。

- `feed.xml` — RSS 2.0
- `atom.xml` — Atom 1.0
- `feed.json` — JSON Feed 1.1

機能は、オンにするまでオフです。既存サイトは変わりません。

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

`false` または省略するとファイルはオフのままです。`true` は既定を有効にします。3 形式すべて、
`content` コレクション（または最初に設定したコレクション）、
20 件上限です。オブジェクトは機能を有効にし、設定した欄だけ上書きします。

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

| オプション   | 型                              | 既定                                 |
| ------------ | ------------------------------- | ------------------------------------ |
| `feeds`      | `boolean` / `FeedsOptions`      | `false`                              |
| `formats`    | `("rss" \| "atom" \| "json")[]` | `["rss", "atom", "json"]`            |
| `collection` | `string`                        | `content`、なければ最初のコレクション |
| `limit`      | `number`                        | `20`                                 |
| `path`       | `string`                        | `/`（サイトルート）                  |

`path` は生成ファイルのサイト相対ディレクトリです。`/feeds` は
`feeds/feed.xml`、`feeds/atom.xml`、`feeds/feed.json` を書きます。

項目は新しい順です。ソートキーは frontmatter の `date`、
`date` がないときは `lastUpdated` です。`draft: true` のエントリは省かれます。

`ssg.siteUrl` なしで `feeds` を有効にすると、ファイルは書かれません。ビルドは
続き、警告を出します。

タイトルと説明はエスケープされるので、XML や JSON から抜けられません。

## 関連

- [コレクション](./collections.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [サイト生成](./site-generation.md)
- [組み込み機能の概要](../built-in-features.md)
