---
title: サイト生成
description: 静的 HTML、OG 画像、編集リンク、コレクション、API ドキュメント、transformer。
---

# サイト生成

ページ単位の Markdown 変換に加え、ドキュメントサイト向けのビルド機能が付きます。

| オプション     | 既定                | 役割                                     |
| -------------- | ------------------- | ---------------------------------------- |
| `ssg`          | `{ enabled: true }` | ビルド時に静的 HTML を生成               |
| `ogImage`      | `false`             | ページごとの OG 画像                     |
| `editThisPage` | `false`             | 「このページを編集」リンク               |
| `collections`  | `content`           | クライアントから Markdown を問い合わせ   |
| `permalinks`   | `false`             | frontmatter の `permalink` / `slug`      |
| `cascade`      | `false`             | ディレクトリ `_index` の既定 frontmatter |
| `docs`         | `{ enabled: true }` | JSDoc / TSDoc から API ドキュメント      |
| `transformers` | `[]`                | 独自の Markdown AST 変換                 |

## 静的サイト生成

SSG は既定でオンです。`srcDir` の各 Markdown が、デフォルトテーマ・ナビ・検索付きの静的 HTML になります。いま読んでいるサイトも同じ経路です。

```ts
import { defineConfig } from "vite-plus";
import { oxContent, defineTheme, defaultTheme } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "content",
      outDir: "dist/docs",
      ssg: {
        siteName: "Ox Content",
        siteUrl: "https://example.com",
        lastUpdated: true,
        theme: defineTheme({
          extends: defaultTheme,
          sidebar: [
            {
              text: "ガイド",
              items: [{ text: "はじめに", link: "/getting-started.md" }],
            },
          ],
        }),
      },
    }),
  ],
});
```

`ssg: false` で静的 HTML を止め、変換だけにできます。

## 関連

- [英語版ガイド](/built-in/site-generation.md)
- [テーマ](../theming.md)
- [パーマリンクと Cascade](./permalinks.md)
