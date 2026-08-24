---
title: パーマリンクとカスケード
description: オプトインの frontmatter permalink / slug ルーティングと、ディレクトリ単位の既定 frontmatter です。
---

# パーマリンクとカスケード

ページ URL は通常 Markdown ファイルツリーに従います。ページに別の公開パスが必要なときは
`permalinks` を、ディレクトリがその子と既定 frontmatter を共有するときは `cascade` を有効にします。

どちらも、オンにするまでオフです。既存サイトは変わりません。

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

`false` または省略すると機能はオフのままです。`true` は既定を有効にします。
オブジェクトは機能を有効にし、設定した欄だけ上書きします。

| オプション   | 型                              | 既定    |
| ------------ | ------------------------------- | ------- |
| `permalinks` | `boolean` / `PermalinksOptions` | `false` |
| `cascade`    | `boolean` / `CascadeOptions`    | `false` |

## パーマリンク

`permalinks` がオンのとき、frontmatter はファイルツリー URL を置き換えられます。

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

`guide/intro.md` に `slug: install` があると `guide/install` になります。両方のキーがあるときは
`permalink` が勝ちます。

安全でない値は拒否され、ファイルツリー URL が残ります。

- `../` または `.` パスセグメント
- 絶対ファイルシステムパス（`C:\`、ドライブレター）
- `javascript:`、`data:`、`vbscript:`、`file:`
- プロトコル相対の `//` URL

2 ページが同じ URL に解決すると、ビルドはエラーを記録し、
最初のページを残して後のページをスキップします。

HTML 属性へ書く値はエスケープされます。

## カスケード

`cascade` がオンのとき、`_index.md`（および `_index.mdx` / `_index.markdown`）
ファイルは、そのディレクトリ以下のページへ既定 frontmatter を供給します。
子はすでに設定したキーを保ちます。`permalink` と `slug` は決して
継承されないので、セクション索引がすべての子を同じ URL へ強制できません。

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

`guide/install.md` は `sidebar: Guide` を継承し、自分のタイトルは保ちます。

## 関連

- [サイト生成](./site-generation.md)
- [コレクション](./collections.md)
- [組み込み機能の概要](../built-in-features.md)
