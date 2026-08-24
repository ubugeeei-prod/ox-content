---
title: リダイレクトとエイリアス
description: frontmatter のエイリアスと書き換えマップから静的 HTML リダイレクトを書く。
---

# リダイレクトとエイリアス

`redirects` を有効にすると、古いパスに小さな静的 HTML を書きます。meta refresh と canonical で、リネーム後も inbound URL が動きます。任意の静的ホストで動きます。

省略または `false` では何も書きません。

```ts
oxContent({
  redirects: true,
});
```

マップで追加の書き換えもできます。

```ts
oxContent({
  redirects: {
    map: {
      "/old-guide": "/guide",
    },
  },
});
```

```md
---
title: 新しい案内
aliases:
  - /old-guide
---
```

外部 URL は `allowExternal` を付けない限り拒否します。`javascript:` / `data:` / `vbscript:` / `//` は落ちます。

## 関連

- [英語版ガイド](/built-in/redirects.md)
- [パーマリンクと Cascade](./permalinks.md)
