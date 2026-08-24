---
title: RSS / Atom / JSON フィード
description: コレクションから生成 HTML の横に書くオプトインのフィード。
---

# RSS / Atom / JSON フィード

`feeds` を有効にし、`ssg.siteUrl` があると、名前付きコレクションから機械可読フィードを書きます。

- `feed.xml` — RSS 2.0
- `atom.xml` — Atom 1.0
- `feed.json` — JSON Feed 1.1

省略または `false` ではオフです。

```ts
oxContent({
  feeds: true,
  ssg: {
    siteUrl: "https://example.com",
  },
});
```

`true` の既定は 3 形式すべて、`content` コレクション（または最初の設定済みコレクション）、20 件上限です。オブジェクトで上書きできます。

```ts
oxContent({
  feeds: {
    formats: ["rss", "json"],
    collection: "blog",
    limit: 10,
    path: "/feeds",
  },
});
```

`siteUrl` が無いとファイルは書かず、警告が出ます。下書きは `publishState` がオンなら落ちます。

## 関連

- [英語版ガイド](/built-in/feeds.md)
- [コレクション](./collections.md)
