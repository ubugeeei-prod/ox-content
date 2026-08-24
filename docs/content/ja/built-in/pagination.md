---
title: 前へ / 次へ
description: サイドバー順から作るオプトインの前後リンク。
---

# 前へ / 次へ

`ssg.pagination` を有効にすると、本文のあと・最終更新の前に前後リンクが付きます。順序はサイドバーです。グループは深さ優先で平坦化し、サイト内 href のない項目は飛ばします。

省略または `false` ではオフです。

```ts
oxContent({
  ssg: {
    pagination: true,
  },
});
```

`true` はデフォルトでオン、オブジェクトもオンです。先頭ページに前はなく、末尾に次はありません。1 ページだけのサイドバーは何も出しません。エントリページと bare モードは pager を出しません。

frontmatter で片側を上書き、または隠せます。

```md
---
prev: false
next: /guide/install
---
```

## 関連

- [英語版ガイド](/built-in/pagination.md)
- [パンくず](./breadcrumbs.md)
