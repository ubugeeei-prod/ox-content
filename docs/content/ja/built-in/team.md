---
title: チーム / メンバーページ
description: layout: team ページ向けの、オプトイン静的メンバーカードです。
---

# チーム / メンバーページ

`ssg.team` が有効なとき、`layout: team` の Markdown ページは、
ページ本文の代わり（またはまわり）に人の静的カードグリッドを描画します。名前、役割、
リンクラベルはエスケープされます。アバターとリンク URL は `https:`、または
`/` で始まり `//` でないサイト相対パスである必要があります。

機能は、オンにするまでオフです。既存サイトは変わりません。オフのあいだ
`layout: team` は無視され、ファイルは普通のページのままです。

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

`false` または省略するとレイアウトは惰性のままです。`true` は空のメンバー
一覧を有効にします。オブジェクトは機能を有効にし、`members` を供給します。

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

拒否されたアバターとリンク URL（`javascript:`、`data:`、`http:`、プロトコル
相対の `//`）はマークアップから省かれます。`/avatars/ada.png` のようなサイト相対パスは残ります。

カードは `.ox-team` クラスを使います。bare モードでも、`generateHtmlPage` がオプションを受け取るときは
同じ URL とエスケープ規則が走ります。

## 関連

- [サイト生成](./site-generation.md)
- [組み込み機能の概要](../built-in-features.md)
