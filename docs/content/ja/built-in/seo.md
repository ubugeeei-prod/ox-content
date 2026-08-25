---
title: SEO
description: page-head API 上の組み込み canonical、robots、hreflang、Open Graph、head 検証。
---

# SEO

[ページ head](./page-head.md) のリゾルバは、文書化した SEO タグを出せます。
設定していない author / publisher / URL / インデックス可否は**発明しません**。

## タグが乗る条件

| タグ                          | いつ                                                                                           |
| ----------------------------- | ---------------------------------------------------------------------------------------------- |
| `<title>`、OG / Twitter title | テーマ付きでは常時。bare も `<title>` は常時。                                                 |
| `description` と OG / Twitter | ページ frontmatter の `description`。                                                          |
| `og:image` / `twitter:image`  | `ssg.ogImage` または生成 OG 画像。                                                             |
| `canonical` / `og:url`        | テーマ付きは `ssg.siteUrl`。bare は計算済み `canonicalUrl`。frontmatter `canonical` が上書き。 |
| `og:site_name`                | bare で `siteName` があるとき。テーマ付きは、独自 meta を足さない限り出さない。                |
| `robots`                      | frontmatter `robots` があるときだけ。                                                          |
| `hreflang` alternate          | i18n の `locale_paths` から、絶対 URL を組み立てられるとき。                                   |

`ssg.siteUrl` が無いテーマ付きページは従来どおりです。canonical も `og:url` も
`hreflang` も出しません。サイトマップや JSON-LD 用に `siteUrl` を置いていると、
これらのタグもオンになります。相対のロケール href を絶対にするにも `siteUrl` が要ります。

```yaml
---
title: Guide
description: How it works
robots: noindex, nofollow
canonical: https://example.com/guide/
---
```

```ts
oxContent({
  ssg: {
    siteName: "Docs",
    siteUrl: "https://example.com",
    headValidation: "warn",
  },
  i18n: {
    locales: [
      { code: "en", name: "English" },
      { code: "ja", name: "日本語" },
    ],
  },
});
```

## 検証

`ssg.headValidation`:

| 値             | 効果                                              |
| -------------- | ------------------------------------------------- |
| 省略 / `false` | 不正値は黙って落とす。既定。                      |
| `warn`         | 正しいタグは残し、指摘をログする。                |
| `strict`       | 危険な URL や不正な `hreflang` でビルドを落とす。 |

不正な独自デスクリプタはどのモードでも落とします。`strict` は CI 向けです。

独自ホストは `renderHead({ validation: "strict", ... })` の `diagnostics` で
同じ指摘を受け取れます。

## JSON-LD の型

`ssg.jsonLd.type` は `TechArticle`（既定）、`BlogPosting`、`WebPage` です。
`ssg.jsonLd.graph` は追加の `@graph` ノードです。未知の型は `TechArticle` に戻します。
[JSON-LD](./json-ld.md) を見てください。

## 関連

- [ページ head](./page-head.md)
- [JSON-LD](./json-ld.md)
- [ロケールスイッチャー](./locale-switcher.md)
- [サイト生成](./site-generation.md)
- 追跡: [#820](https://github.com/ubugeeei-prod/ox-content/issues/820)
