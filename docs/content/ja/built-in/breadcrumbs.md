---
title: パンくず
description: サイトルートからサイドバー祖先までのオプトインの道筋。
---

# パンくず

`ssg.breadcrumbs` または `theme.breadcrumbs` を有効にすると、記事の上にルートから現在ページまでの道筋が付きます。現在ページはリンクではありません。

省略または `false` ではオフです。

```ts
oxContent({
  ssg: {
    breadcrumbs: true,
  },
});
```

テーマ側にも置けます。

```ts
oxContent({
  ssg: {
    theme: {
      breadcrumbs: true,
    },
  },
});
```

`true` はデフォルトでオン、オブジェクトもオンです。エントリページと bare モードは出しません。

## 関連

- [英語版ガイド](/built-in/breadcrumbs.md)
- [前へ / 次へ](./pagination.md)
