---
title: パーマリンクと Cascade
description: オプトインの frontmatter permalink / slug ルーティングと、ディレクトリ単位の既定 frontmatter。
---

# パーマリンクと Cascade

ページ URL は通常、Markdown のファイルツリーに従います。公開パスを変えたいページには `permalinks`、ディレクトリ配下で既定 frontmatter を共有したいときは `cascade` をオンにします。

どちらも自分でオンにするまでオフです。既存サイトはそのままです。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      permalinks: true,
      cascade: true,
    }),
  ],
};
```

`false` または省略は機能をオフのままにします。`true` は既定でオンです。オブジェクトを渡すと機能はオンになり、設定したフィールドだけ上書きします。

| オプション   | 型                              | 既定    |
| ------------ | ------------------------------- | ------- |
| `permalinks` | `boolean` / `PermalinksOptions` | `false` |
| `cascade`    | `boolean` / `CascadeOptions`    | `false` |

## パーマリンク

`permalinks` がオンのとき、frontmatter でファイルツリー URL を置き換えられます。

```md
---
title: Getting Started
permalink: /getting-started
---
```

`slug` は最後のパスセグメントだけを置き換えます。

```md
---
title: Install
slug: install
---
```

`guide/intro.md` に `slug: install` があると `guide/install` になります。両方あるときは `permalink` が勝ちます。

安全でない値は拒否し、ファイルツリー URL を残します。

- `../` または `.` のパスセグメント
- 絶対ファイルシステムパス（`C:\`、ドライブ文字）
- `javascript:`、`data:`、`vbscript:`、`file:`
- プロトコル相対の `//` URL

2 ページが同じ URL に解決すると、ビルドはエラーを記録し、先のページを残して後のページを飛ばします。

HTML 属性に書き出す値はエスケープされます。

## Cascade

`cascade` がオンのとき、`_index.md`（および `_index.mdx` / `_index.markdown`）がそのディレクトリ以下のページに既定 frontmatter を渡します。子がすでに持っているキーはそのままです。`permalink` と `slug` は継承しないので、セクション索引がすべての子を同じ URL に押し込めません。

```md
<!-- guide/_index.md -->

---

sidebar: Guide
---
```

```md
<!-- guide/install.md -->

---

title: Install
---
```

`guide/install.md` は `sidebar: Guide` を継承し、自分のタイトルは残します。

## 関連

- [サイト生成](./site-generation.md)
- [コレクション](./collections.md)
- [組み込み機能の一覧](../built-in-features.md)
