---
title: Markdown ソースの併記
description: 生成 HTML の横に、元の Markdown をオプトインで書き出す。
---

# Markdown ソースの併記

`ssg.markdownSource` を有効にすると、SSG ビルドは公開した各 HTML ページの横に
元の Markdown を書き出します。`vite dev` でも同じ URL で配信します。
省略またはオフのときは何も足しません。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        markdownSource: true,
      },
    }),
  ],
};
```

`false` または省略は追加ファイルを書きません。`true` は既定で有効にします。
オブジェクトにすると機能をオンにし、alternate リンクだけオフにしたり、
既定テーマの Copy as Markdown をオプトインしたりできます。

```ts
oxContent({
  ssg: {
    markdownSource: {
      alternate: false,
      copy: true,
    },
  },
});
```

| オプション       | 型                                  | 既定    |
| ---------------- | ----------------------------------- | ------- |
| `markdownSource` | `boolean` / `MarkdownSourceOptions` | `false` |
| `alternate`      | `boolean`                           | `true`  |
| `copy`           | `boolean`                           | `false` |

## URL の対応

併記ファイルはソースのファイルツリーではなく、**公開された**ページ URL に
従います。HTML の出力拡張子は変わりません。併記は常に `.md` です。

| 公開 HTML                                  | 併記                  |
| ------------------------------------------ | --------------------- |
| `/blog/slug/index.html`                    | `/blog/slug.md`       |
| `/index.html`                              | `/index.md`           |
| `/guide/index.htm`（独自拡張子）           | `/guide.md`           |
| `/docs/guide/index.html`（`base`）         | `/docs/guide.md`      |
| `/getting-started/index.html`（permalink） | `/getting-started.md` |
| `/ja/guide/index.html`（ロケール）         | `/ja/guide.md`        |

パス脱出（`..`）は拒否します。同じ併記に解決する 2 ページは先のページを残し、
後のページをスキップします。

## frontmatter

併記はソースファイルの **バイト列そのもの** で、YAML frontmatter も含みます。
取り除いたり書き換えたりしません。HTML から Markdown を復元すると執筆構文が
落ちます。この経路はページ変換ですでに読んだバイトをコピーし、併記のためだけに
Markdown を再パースしません。

## 下書きと除外

`draft: true` と `unlisted: true` のソースは、非公開ページの HTML が残る場合でも
書き出さず配信しません。[`publishState`](./drafts.md) がオンなら、予約公開と
期限切れもそのフィルタに従います。

著者ソースがない生成ページ（ブログ索引、タクソノミー、セクション索引、404）
には併記を付けません。

## alternate リンクとテーマ

`alternate` がオン（既定）のとき、生成 HTML には次が入ります。

```html
<link rel="alternate" type="text/markdown" href="/guide.md" />
```

独自レンダラとテーマは `usePageProps()` から同じ URL を読めます。

```tsx
const page = usePageProps();
return page.markdownSource ? <a href={page.markdownSource}>ソース</a> : null;
```

`copy` がオンのとき、既定テーマはタイトル付近に **Copy as Markdown** ボタンと
**View Markdown** リンクを付けます（「このページを編集」があるときはその隣）。
コピーは併記の `.md` URL を取得し、frontmatter を含む元のソースバイトを
クリップボードに書きます。View Markdown は静的な `<a href="…md">` なので
JavaScript なしでも開けます。併記がオンでも `copy` は明示しない限りオフです。

このリポジトリのドキュメントサイトは `markdownSource: { copy: true }` で
コントロールを有効にしています。

## 関連

- [サイト生成](./site-generation.md)
- [下書き / 非公開 / 予約公開](./drafts.md)
- [パーマリンクと Cascade](./permalinks.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [組み込み機能の一覧](../built-in-features.md)
