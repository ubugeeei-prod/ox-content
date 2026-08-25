---
title: JSON-LD 構造化データ
description: ページ head に出すオプトインの TechArticle / WebSite / BreadcrumbList JSON-LD。
---

# JSON-LD 構造化データ

`ssg.jsonLd` を有効にすると、テーマ付きページの `<head>` に、既存の Open Graph
タグの直後へ `<script type="application/ld+json">` が出力されます。ペイロードは
ページを `WebSite` の一部である `TechArticle` として記述します。表示用の
パンくずがあり、JSON-LD 側の breadcrumbs がオフでなければ、
`BreadcrumbList` も含まれます。

この機能は明示的にオンにするまでオフです。既存サイトの出力は変わりません。
パンくずだけを有効にしても JSON-LD は**出ません**。[パンくず](./breadcrumbs.md)
と [#696](https://github.com/ubugeeei-prod/ox-content/issues/696) を見てください。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        jsonLd: true,
        siteUrl: "https://example.com",
      },
    }),
  ],
};
```

`false` または省略では構造化データを出しません。`true` はデフォルトでオンです。
オブジェクトもオンにしたうえで、`BreadcrumbList` を隠したり publisher を渡したりできます。

```ts
oxContent({
  ssg: {
    jsonLd: {
      breadcrumbs: true,
      publisher: {
        name: "Ox Content",
        url: "https://oxc.rs",
      },
    },
    breadcrumbs: true,
    siteUrl: "https://example.com",
  },
});
```

| フィールド    | 既定          | 効果                                                                 |
| ------------- | ------------- | -------------------------------------------------------------------- |
| `breadcrumbs` | `true`        | 表示用の道筋があるときだけ `BreadcrumbList` を出す。`false` で隠す。 |
| `publisher`   | 省略          | 任意の `{ name?, url? }`。未設定のフィールドは捏造しない。           |
| `type`        | `TechArticle` | ページの `@type`。`TechArticle` / `BlogPosting` / `WebPage`。        |
| `graph`       | 省略          | 追加の `@graph` オブジェクト。不正な JSON は落とす。                 |

## 何が出力されるか

スクリプトは単一の `@graph` ドキュメントです。

| `@type`          | いつ                                                                  | 主なフィールド                                                                                                |
| ---------------- | --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `WebSite`        | `jsonLd` がオン                                                       | `name` は `siteName`。`url` / `@id` は `siteUrl` があるとき                                                   |
| `TechArticle`    | `jsonLd` がオン                                                       | `headline`、`description`。`url` / `@id` / `isPartOf` は `siteUrl` があるとき。`publisher` は設定したときだけ |
| `BreadcrumbList` | 表示用パンくずがあり、**かつ** `jsonLd.breadcrumbs` が `false` でない | `itemListElement` に `position`、`name`、絶対 URL を組み立てられるときの `item`                               |

`@id` と `url` には `siteUrl` が必要です。無いときはそれらの絶対 URL
フィールドを省略します。ホスト、ロゴ、publisher を捏造しません。

## パンくず

`ssg.breadcrumbs` / `theme.breadcrumbs` は表示用の道筋を制御します。
`ssg.jsonLd.breadcrumbs` は、その道筋を `BreadcrumbList` としても出すかどうかだけを制御します。

| 表示用の道筋 | `jsonLd.breadcrumbs` | `BreadcrumbList` |
| ------------ | -------------------- | ---------------- |
| オフ         | `true`（既定）       | 省略             |
| オン         | `true`（既定）       | 出力             |
| オン         | `false`              | 省略             |

エントリページは表示用の道筋を出さないので、`BreadcrumbList` も出しません。

## 安全性

文字列はすべて JSON エンコードされます。`<`、`>`、`&` は JSON の `\u`
エスケープで書くので、敵対的な title で `<script>` タグを抜けられません。

publisher とパンくずの `item` URL では `javascript:`、`data:`、`vbscript:`、
プロトコル相対の `//` を拒否します。出力するのは `http:` / `https:` の絶対
URL、または `siteUrl` で解決できるサイト相対パスだけです。

bare モードでは JSON-LD を出しません。

## 関連

- [ページ head](./page-head.md)
- [SEO](./seo.md)
- [パンくず](./breadcrumbs.md)
- [サイト生成](./site-generation.md)
- [組み込み機能の概要](../built-in-features.md)
- トラッキング issue: [#696](https://github.com/ubugeeei-prod/ox-content/issues/696)
