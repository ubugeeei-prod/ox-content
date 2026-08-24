---
title: ヘッダーナビ、告知、ページクロム
description: オプトインのヘッダーナビ、告知バー、ページ単位のクロムフラグです。
---

# ヘッダーナビ、告知、ページクロム

既定テーマはヘッダータイトル、検索、テーマトグルを出荷します。ヘッダー
ナビ、告知バー、ページ単位のクロムフラグは、オプトインするまで **オフ** です。
新しいオプションを設定しない限り、既存サイトは変わりません。

## ヘッダーナビ

`theme.nav` に `{ text, link }` 項目、または `{ text, items }`
ドロップダウンの配列を設定します。`text` は文字列またはロケールマップ
（`{ en: "Guide", ja: "ガイド" }`）にできます。現在のページロケールがあるときに使います。

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

ラベルはエスケープされます。`link` が `javascript:`、`data:`、
`vbscript:`、またはプロトコル相対の `//` href を使う項目は省かれます。

ドロップダウンは `aria-expanded` と `aria-haspopup` 付きの `button` を使います。Escape は
開いたメニューを閉じます。小さなビューポートではリストが横スクロールするので、
ページは溢れません。

## 告知バー

ヘッダーの上にバーを出すには `theme.announcement` を設定します。

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

| 欄           | 必須 | 効果                                                                    |
| ------------ | ---- | ----------------------------------------------------------------------- |
| `text`       | はい | エスケープされます。raw HTML スロットはありません。                     |
| `link`       | いいえ | `https:` または同一オリジンのみ。他のスキームは捨てられます。         |
| `dismissKey` | いいえ | 最善努力の `localStorage` キー。不正なキーでも静的バーは描画されます。 |

## ページ単位のクロム

`ssg.pageChrome` は既定オフです。`true` または `{}` でこれらの
frontmatter フラグを読むようになります。省略したフラグはいまのレイアウトを保ちます。`false` はその
領域を隠します。

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

`aside: false` は `outline: false` のエイリアスです。`pageChrome` がオフのとき、
これらのフラグは無視されるので、既存の frontmatter がシェルを変えられません。

bare モードはヘッダーナビ、告知バー、ページクロム
クラスを決して出しません。
