---
title: チーム / メンバー
description: layout: team 向けのオプトイン静的カード。
---

# チーム / メンバー

`ssg.team` を有効にすると、`layout: team` の Markdown が本文の代わり（または周囲）に人物カードのグリッドを描画します。名前、役割、リンクラベルはエスケープされます。アバターとリンク URL は `https:`、または `/` で始まり `//` ではないサイト相対パスだけです。

省略または `false` では `layout: team` は無視され、ふつうのページのままです。

```ts
oxContent({
  ssg: {
    team: {
      members: [
        {
          name: "Ada Lovelace",
          role: "数学者",
          avatar: "https://example.com/ada.png",
          links: [{ label: "Web", href: "https://example.com/ada" }],
        },
      ],
    },
  },
});
```

`true` は空のメンバー一覧でオンです。オブジェクトで `members` を渡します。

```md
---
title: チーム
layout: team
---
```

## 関連

- [英語版ガイド](/built-in/team.md)
