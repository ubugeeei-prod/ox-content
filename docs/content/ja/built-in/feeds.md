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

`false` または省略はファイルを出しません。`true` は既定でオンです。3 形式すべて、`content` コレクション（なければ設定された最初のコレクション）、20 件制限です。単一オブジェクトは 1 本の既定フィードで、設定したフィールドだけ上書きします。

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

名前付きレコードまたは配列を渡すと、複数フィードを書き出します。チャンネルごとにコレクション、パス、形式、メタデータを設定できます。

```ts
oxContent({
  feeds: {
    blog: {
      formats: ["rss"],
      collection: "blog",
      path: "/",
      title: "blog | example.com",
      description: "Technical articles",
      language: "en",
      image: "https://example.com/icon.png",
      favicon: "https://example.com/icon.png",
      copyright: "© 2026 example.com",
    },
    media: {
      formats: ["rss"],
      collection: "media",
      path: "/works/media",
      title: "Media | example.com",
      language: "ja",
    },
  },
  ssg: {
    siteUrl: "https://example.com",
  },
});
```

## Programmatic items

フィードの元データが JSON ファイル、database の結果、ビルド時に集めた curated データのときは、チャンネルに `collection` ではなく `items` を設定できます。resolver は SSG 中に実行され、promise を返せます。

```ts
import media from "./src/contents/external-rss/media.json";

oxContent({
  feeds: {
    media: {
      formats: ["rss"],
      path: "/works/media",
      title: "Media | ryoppippi.com",
      items: async () =>
        media
          .filter((item) => !item.playlist)
          .map((item) => ({
            title: item.title,
            url: item.link,
            id: `media:${item.link}`,
            date: item.pubDate,
            description: `${item.kind === "podcast" ? "Podcast" : "YouTube"} | ${item.title}`,
            author: { name: "ryoppippi", url: "https://ryoppippi.com" },
            language: item.lang,
          })),
    },
  },
  ssg: {
    siteUrl: "https://ryoppippi.com",
  },
});
```

1 つのチャンネルに `collection` と `items` を同時に書くことはできません。
同時に指定すると reject されます。programmatic item は `title`、`url` または
`loc`、任意の `id`、`date`、`description`、`content`、`author` /
`authors`、`image`、`attachments`、`language` を受け取ります。RSS、Atom、
JSON Feed の各 renderer は、その形式が対応する field を出力します。

| オプション    | 型                                         | 既定                                  |
| ------------- | ------------------------------------------ | ------------------------------------- |
| `feeds`       | `boolean` / 単一フィード / 名前付き / 配列 | `false`                               |
| `formats`     | `("rss" \| "atom" \| "json")[]`            | `["rss", "atom", "json"]`             |
| `collection`  | `string`                                   | `content`、なければ最初のコレクション |
| `items`       | `FeedItemInput[]` / async resolver         | 省略                                  |
| `limit`       | `number`                                   | `20`                                  |
| `path`        | `string`                                   | `/`（サイトルート）                   |
| `title`       | `string`                                   | SSG のサイト名                        |
| `description` | `string`                                   | SSG のサイト説明                      |
| `language`    | `string`                                   | 省略                                  |
| `image`       | `string`                                   | 省略                                  |
| `favicon`     | `string`                                   | 省略                                  |
| `copyright`   | `string`                                   | 省略                                  |

`path` は生成ファイルのサイト相対ディレクトリです。`/feeds` なら `feeds/feed.xml`、`feeds/atom.xml`、`feeds/feed.json` を書き出します。チャンネルの `title`、`description`、`language`、`image`、`favicon`、`copyright` は、各形式に対応するフィールドがあるときサイト既定を上書きします（JSON Feed に copyright はありません）。

項目は新しい順です。ソートキーは frontmatter の `date`、なければ `lastUpdated` です。`draft: true` のエントリは外れます。

`feeds` をオンにしても `ssg.siteUrl` がなければ、ファイルは書き出しません。ビルドは続き、警告を出します。

タイトルと説明はエスケープされるので、XML や JSON の外へは出られません。

## ブログ索引の項目

[ブログ](./blog.md) の `blog.feeds` で集めた外部投稿は索引にだけ載ります。
生成ファイルには入りません。このリリースに取り込みスイッチはありません。

## 関連

- [SSG 出力プリミティブ](./ssg-output.md)
- [コレクション](./collections.md)
- [ブログ](./blog.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [サイト生成](./site-generation.md)
- [組み込み機能の一覧](../built-in-features.md)
