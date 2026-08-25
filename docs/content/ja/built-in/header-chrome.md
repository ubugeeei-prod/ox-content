---
title: ヘッダー chrome
description: オプトインのヘッダーナビ、告知バー、ページ単位の chrome フラグ。
---

# ヘッダー chrome

既定テーマはヘッダーのタイトル、検索、テーマ切替を載せます。ヘッダーナビ、告知バー、ページ単位の chrome フラグは、オプトインするまで **オフ** です。新しいオプションを設定しない限り、既存サイトは変わりません。

## ヘッダーナビ

`theme.nav` に `{ text, link }` 項目、または `{ text, items }` ドロップダウンの配列を置きます。`text` は文字列でも、ロケールマップ（`{ en: "Guide", ja: "ガイド" }`）でも構いません。現在ページのロケールがあればそれを使います。

```ts
import { oxContent, defineTheme } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        theme: defineTheme({
          nav: [
            { text: "Guide", link: "/guide/" },
            {
              text: "API",
              items: [
                { text: "SSG", link: "/api/ssg/" },
                { text: "Search", link: "/api/search/" },
              ],
            },
          ],
        }),
      },
    }),
  ],
};
```

ラベルはエスケープされます。`link` が `javascript:`、`data:`、`vbscript:`、またはプロトコル相対の `//` href の項目は外れます。

ドロップダウンは `aria-expanded` と `aria-haspopup` 付きの `button` です。Escape で開いているメニューを閉じます。狭いビューポートではリストが横スクロールするので、ページは溢れません。

## 告知バー

`theme.announcement` を設定すると、ヘッダーの上にバーが出ます。

```ts
oxContent({
  ssg: {
    theme: defineTheme({
      announcement: {
        text: "Ox Content 3 is in progress.",
        link: "/v3-roadmap/",
        dismissKey: "v3-wip",
      },
    }),
  },
});
```

| フィールド   | 必須   | 効果                                                                                                                                            |
| ------------ | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| `text`       | はい   | エスケープされます。生 HTML 用のスロットはありません。                                                                                          |
| `link`       | いいえ | `https:` または同一オリジンだけ。他のスキームは捨てます。                                                                                       |
| `dismissKey` | いいえ | ベストエフォートの `localStorage` キー。不正なキーでも静的バーは出ます。閉じるとバーに `hidden` が付き、テーマ CSS は `[hidden]` を尊重します。 |

## ページ単位の chrome

`ssg.pageChrome` は既定オフです。`true` または `{}` で、次の frontmatter フラグを読むようになります。省略したフラグは今のレイアウトのままです。`false` はその領域を隠します。

```md
---
title: Landing
sidebar: false
outline: false
footer: false
navbar: false
lastUpdated: false
editLink: false
---
```

`aside: false` は `outline: false` の別名です。`pageChrome` がオフのとき、これらのフラグは無視するので、既存の frontmatter がシェルを変えられません。

bare モードはヘッダーナビ、告知バー、page-chrome クラスを一切出しません。
