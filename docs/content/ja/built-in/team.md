---
title: チーム / メンバー
description: layout: team ページ向けの、オプトインの静的メンバーカード。
---

# チーム / メンバー

`ssg.team` を有効にすると、`layout: team` の Markdown ページは、ページ本文の代わり（またはその周り）に、人の静的カードグリッドを描画します。名前、役割、リンクラベルはエスケープされます。アバターとリンクの URL は `https:`、または `/` で始まり `//` ではないサイト相対パスである必要があります。

機能は自分でオンにするまでオフです。既存サイトはそのままです。オフのあいだ、`layout: team` は無視され、ファイルは普通のページのままです。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        team: {
          members: [
            {
              name: "Ada Lovelace",
              role: "Mathematician",
              avatar: "https://example.com/ada.png",
              links: [{ label: "Website", href: "https://example.com/ada" }],
            },
          ],
        },
      },
    }),
  ],
};
```

`false` または省略はレイアウトを無効のままにします。`true` は空のメンバー一覧でオンになります。オブジェクトを渡すと機能はオンになり、`members` を渡せます。

```md
---
title: Team
layout: team
---

Optional introduction. Safe member cards are rendered above this body.
```

| オプション | 型                        | 既定    |
| ---------- | ------------------------- | ------- |
| `ssg.team` | `boolean` / `TeamOptions` | `false` |
| `members`  | `TeamMember[]`            | `[]`    |
| `name`     | `string`                  | —       |
| `role`     | `string`                  | —       |
| `avatar`   | `string`                  | —       |
| `links`    | `{ label, href }[]`       | —       |

拒否されるアバターとリンク URL（`javascript:`、`data:`、`http:`、プロトコル相対の `//`）はマークアップから外れます。`/avatars/ada.png` のようなサイト相対パスは残します。

カードは `.ox-team` クラスを使います。bare モードでも、`generateHtmlPage` がオプションを受け取れば、同じ URL とエスケープ規則が走ります。

## 関連

- [サイト生成](./site-generation.md)
- [組み込み機能の一覧](../built-in-features.md)
