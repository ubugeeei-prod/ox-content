---
title: 生成セクション索引ページ
description: 子ページはあるが index.md がないディレクトリ向けの、オプトイン静的索引。
---

# 生成セクション索引ページ

`ssg.sectionIndex` を有効にすると、SSG ビルドは収集済みページとファイルツリーを辿ります。子ページがあるのに既存の索引ルート（`index.md`、`index.mdx`、またはすでに生成されたディレクトリ索引）がない各ディレクトリに対して、子ページをカードまたはリストで並べたテーマ付きランディングページを書きます。

一覧は frontmatter の `title` を使います（なければファイル名）。ナビ、検索、サイドバーなどのテーマ chrome は、通常ページと同じ `generateHtmlPage` 経路です。既存の `index.md` / `index.mdx` は上書きしません。

省略または `false` ではオフです。既存サイトの挙動は変わりません。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        sectionIndex: true,
      },
    }),
  ],
};
```

`false` または省略では何も書きません。`true` はカード一覧でオンです。オブジェクトは機能をオンにしたうえで、指定したフィールドだけ上書きします。

```ts
oxContent({
  ssg: {
    sectionIndex: {
      style: "list",
    },
  },
});
```

| オプション         | 型                                | 既定      |
| ------------------ | --------------------------------- | --------- |
| `ssg.sectionIndex` | `boolean` / `SectionIndexOptions` | `false`   |
| `style`            | `"list"` / `"cards"`              | `"cards"` |

## 何が生成されるか

次のツリーで `guide/index.md` がない場合:

```
content/
  index.md
  guide/
    a.md
    b.md
```

ビルドは `dist/guide/index.html` を書きます。子のタイトルは frontmatter があればそれを使います。

```md
---
title: Install
---

# Install
```

`title` を省略するとファイル名を整形します（`getting-started.md` → "Getting Started"）。それ自身がページを持つ入れ子ディレクトリは、そのディレクトリの索引（手書きまたは生成）へリンクする 1 件の子として現れます。

ルートの `index.md` はそのままです。子が下書きまたは非公開ページだけのディレクトリには、生成索引は出ません。

## 既存の索引は残す

`guide/index.md` または `guide/index.mdx` がすでにある場合、それがセクションのランディングページです。オプションがオンでも、そのディレクトリは生成対象から外れます。パーマリンクがすでにディレクトリ URL を占めている場合も同じです。

この機能で手書きの索引を包んだり置き換えたりしないでください。出したい Markdown を書くか、`index.md` を置かずに一覧を生成させてください。

## 下書き、非公開、敵対的な入力

`draft: true` または `unlisted: true` の子は一覧から外れます。`publishState` がオフでも同様です。予約公開 / 期限切れのページは、そのオプションがオンのとき `publishState` に従います。

タイトル、説明、href はエスケープされます。`javascript:`、`data:`、`vbscript:`、`file:`、プロトコル相対の `//` の href はマークアップから落とします。`"/guide" onclick="alert(1)"` のような属性脱出はエスケープされ、`href` 属性の外へ出られません。

カードは `.ox-section-index` クラスを使います。リストスタイルは `.ox-section-index--list` です。bare モードでも、一覧 HTML を `generateHtmlPage` に通すと同じエスケープと href 規則が走ります。

## 関連

- [サイト生成](./site-generation.md)
- [下書き / 非公開 / 予約公開](./drafts.md)
- [パーマリンクと Cascade](./permalinks.md)
- [組み込み機能の概要](../built-in-features.md)
