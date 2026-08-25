---
title: ブログ
description: オプトインのページ送り索引、著者、タグ、読了時間、アーカイブ。
---

# ブログ

`blog` を有効にすると（トップレベルまたは `ssg.blog`）、コレクションの上に
ブログ用レイアウトが載ります。

- `/blog/` のページ送り付き索引（2 ページ目以降は `/blog/page/2/`）
- 各投稿の著者と読了時間
- `/blog/tags/{tag}/` のタグページ
- `/blog/archive/`、`/blog/archive/{yyyy}/`、`/blog/archive/{yyyy}/{mm}/`
  の年次・月次アーカイブ
- 同じ索引へマージする任意の外部 RSS / Atom ソース

省略または `false` ではオフです。既存サイトの出力は変わりません。
タグとアーカイブはこの機能の中で実装します。タクソノミー（#687）を待ちません。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      blog: true,
    }),
  ],
};
```

`ssg.blog` でも同じです。両方あるときはトップレベルの `blog` が優先されます。

```ts
oxContent({
  ssg: {
    blog: true,
  },
});
```

`false` または省略では追加ページも投稿 chrome も出ません。`true` は既定値で
オンです。オブジェクトはオンにしたうえで指定したフィールドだけ上書きします。

```ts
oxContent({
  blog: {
    collection: "posts",
    pageSize: 5,
    authors: {
      ada: {
        name: "Ada Lovelace",
        bio: "数学者",
        url: "https://example.com/ada",
      },
    },
  },
});
```

| オプション   | 型                             | 既定                                                     |
| ------------ | ------------------------------ | -------------------------------------------------------- |
| `blog`       | `boolean` / `BlogOptions`      | `false`                                                  |
| `ssg.blog`   | `boolean` / `BlogOptions`      | `false`                                                  |
| `collection` | `string`                       | 名前が `blog` のコレクション、なければ唯一のコレクション |
| `authors`    | `Record<string, BlogAuthor>`   | `{}`                                                     |
| `pageSize`   | `number`                       | `10`                                                     |
| `feeds`      | `(string \| BlogFeedSource)[]` | `[]`（取得しない）                                       |

## コレクション

投稿は名前付きコレクションから取ります。

1. 明示した `collection` が常に優先されます。
2. なければ名前が `blog` のコレクションを使います。
3. なければ設定済みコレクションが 1 つだけのとき、それを使います。
4. コレクションが複数あり、どれも `blog` でないときは `collection` を
   指定してください。指定が無いと追加ページは書きません（ビルドは続き、
   警告が出ます）。

コレクションが無効なときは、一覧対象の全ページを投稿として扱います。

## ページ送り

索引は新しい投稿が先です。並べ替えキーは frontmatter の `date`、同じ日付なら
href です。`pageSize` 件が `/blog/` に載ります。続きは `/blog/page/2/`、
`/blog/page/3/` です。1 ページ目に `/blog/page/1/` は使いません。
ページャーのラベルは Newer と Older です。

`pageSize` が 1 未満のときは 10 に戻します。

## 著者

著者は設定のマップと、frontmatter の `author` / `authors` から決まります。
文字列または文字列配列を受け付けます。各値は `blog.authors` で引きます。
マップに無いキーはそのまま表示名になります。

```md
---
title: メモ
date: 2024-03-01
author: ada
authors:
  - grace
---
```

名前と bio は HTML エスケープされます。`url` は `https:`、または `/` で始まり
`//` ではないサイト相対パスだけです。拒否した URL（`javascript:`、`data:`、
`http:`、protocol-relative の `//`）は出さず、名前だけ平文で残します。

## 読了時間

読了時間は決定的です。同じ Markdown は常に同じ整数分になります。計算は次の
とおりです。

1. YAML frontmatter（`---` … `---`）を除く。
2. フェンスコード（` ``` ` … ` ``` `。閉じが無いフェンスはファイル末尾まで）と
   インラインコード（`` `…` ``）を除く。
3. ラテン語の語を数える: `[A-Za-z0-9]+`。アポストロフィで繋がった 2 部分は
   1 語です（`don't` は 1 語）。
4. CJK 文字を数える: ひらがな、カタカナ、CJK 統合漢字（拡張 A と互換漢字を
   含む）、ハングル。
5. `minutes = ceil(latin_words / 200 + cjk_chars / 500)`
6. 除去後に何も残らなければ `0`。何か残れば最低 1 分です。

フェンスやコードスパンの中のタグ・著者は読了時間に入りません。値は
`.ox-blog-meta` に `N min read` として前置されます。

## タグ

用語は frontmatter の `tags` だけです。文字列または文字列配列を受け付けます。
フェンスやインラインコードの中の `tags` はページを作りません。

```md
---
title: インストール
date: 2024-01-15
tags:
  - rust
  - napi
---
```

各用語は `/blog/tags/{slug}/` になります。スラッグは安定で `[a-z0-9-]` だけ
です。`javascript:`、`../`、`//evil.com` のような危険な値は href から落とします。
ラベル、タイトル、href はすべて HTML エスケープされます。

## アーカイブ

アーカイブは frontmatter の `date`（`YYYY-MM-DD` または ISO-8601）を使います。
年と月はパースした UTC の暦日から取るので、同じ `date` は常に同じパスになります。

- `/blog/archive/` — 日付付き投稿がある年
- `/blog/archive/{yyyy}/` — その年の月と投稿
- `/blog/archive/{yyyy}/{mm}/` — その月の投稿（`mm` はゼロ埋め）

パースできない `date` の投稿は索引とタグページにだけ載ります。

## 外部フィード

`feeds` は空でない配列を書いたときだけ動きます。ビルドが取るのはその
設定 URL だけです。Markdown や HTML の中のリンクは取りに行きません。

```ts
oxContent({
  blog: {
    feeds: [
      "https://example.com/rss.xml",
      {
        url: "https://example.com/atom.xml",
        language: "ja",
        author: "ada",
        onError: "warn",
      },
    ],
  },
});
```

| フィールド | 型                 | 既定   | 役割                                                 |
| ---------- | ------------------ | ------ | ---------------------------------------------------- |
| `url`      | `string`           | —      | 絶対 `https:` の RSS または Atom URL                 |
| `language` | `string`           | —      | 項目に言語が無いときの既定                           |
| `author`   | `string`           | —      | 項目に著者が無いときの既定                           |
| `onError`  | `"warn"`/`"error"` | `warn` | ソースを飛ばすか、他ソースのあとでビルドを失敗させる |

文字列エントリは `{ url, onError: "warn" }` です。同じ URL はビルドあたり
1 回だけ取ります。ページごとではありません。タイムアウト、リダイレクト回数、
応答サイズ、`https:` の公開ホストだけ、が効きます。ループバック、プライベート、
リンクローカルは DNS のあとで拒否します。HTML ページはパースしません。

`warn` の失敗ソースは飛ばし、成功したソースはマージします。1 つの失敗で
ブログ全体は落ちません。`onError: "error"` は残りのソースを終えたあとで
ビルドを失敗させます。

項目はタイトル、正規の `https:` リンク、公開日、安定 id、言語、要約がある
ときそれを残します。ローカル投稿とマージし、新しい順、同じなら href です。
重複は正規 URL か明示の安定 id で判定します。ローカル投稿が勝ちます。

外部項目には `external` マーカー（`class="ox-blog-external"`、
`rel="external"`）が付きます。テーマはリモート URL を残し、ローカル経路へ
書き換えてはいけません。

外部項目は生成する RSS / Atom / JSON フィードには**入りません**。
このリリースに取り込みスイッチはありません。

## 下書きと非公開

frontmatter の `draft: true` と `unlisted: true` は、`publishState` がオフでも
索引・タグ・アーカイブ・投稿 chrome から外れます。公開状態フィルタがオフなら、
ソース HTML 自体は書かれることがあります。

`publishState` がオンのときは、下書き・非公開・予約公開はその機能の
一覧ルールにも従います。

## 関連

- [コレクション](./collections.md)
- [下書き / 非公開 / 予約公開](./drafts.md)
- [RSS / Atom / JSON フィード](./feeds.md)
- [サイト生成](./site-generation.md)
- [組み込み機能の概要](../built-in-features.md)
