---
title: ヘッダー chrome
description: オプトインのヘッダーナビ、告知バー、ページ単位の chrome。
---

# ヘッダー chrome

デフォルトテーマにはタイトル、検索、テーマ切替が付きます。ヘッダーナビ、告知バー、ページ単位の chrome フラグは、明示するまでオフです。既存サイトは新しいオプションを置かない限り変わりません。

## ヘッダーナビ

`theme.nav` に `{ text, link }` または `{ text, items }` ドロップダウンを置きます。`text` は文字列、またはロケールマップ (`{ en: "Guide", ja: "ガイド" }`) です。現在ページのロケールが使われます。

```ts
import { oxContent, defineTheme } from "@ox-content/vite-plugin";

oxContent({
  ssg: {
    theme: defineTheme({
      nav: [
        { text: { en: "Guide", ja: "ガイド" }, link: "/getting-started/" },
        { text: "API", link: "/api/" },
      ],
    }),
  },
});
```

ラベルはエスケープされます。`javascript:` / `data:` / `vbscript:` / `//` の `link` は落ちます。

## 告知バー

`theme.announcement` でヘッダー上にバーを出します。`text` はエスケープされ、生 HTML 枠はありません。`link` は `https:` または同一オリジンだけです。`dismissKey` は最善努力の `localStorage` キーです。

## ページ単位の chrome

`ssg.pageChrome` は既定オフです。`true` または `{}` で frontmatter フラグを読みます。省略したフラグは今のレイアウトのまま、`false` でその領域を隠します。

```md
---
sidebar: false
outline: false
---
```

対象は `sidebar`, `outline` / `aside`, `footer`, `navbar`, `lastUpdated`, `editLink` です。

## 関連

- [英語版ガイド](/built-in/header-chrome.md)
- [ロケールスイッチャー](./locale-switcher.md)
