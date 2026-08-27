---
title: サイト生成
description: 静的サイト生成、OG 画像、編集リンク、コンテンツコレクション、API ドキュメント、独自トランスフォーマ。
---

# サイト生成

ページ単位の Markdown 変換に加え、プラグインはドキュメントサイトが必要とするビルド単位の機能を載せます。静的 HTML 生成、ページごとの Open Graph 画像、コンテンツコレクション、生成 API ドキュメントです。

| オプション     | 既定                   | 目的                                             |
| -------------- | ---------------------- | ------------------------------------------------ |
| `ssg`          | `{ enabled: true }`    | ビルド中に静的 HTML ページを生成する。           |
| `ogImage`      | `false`                | ページごとの Open Graph 画像を生成する。         |
| `editThisPage` | `false`                | 「このページを編集」リンクを付ける。             |
| `collections`  | `content` コレクション | クライアントコードから Markdown を問い合わせる。 |
| `permalinks`   | `false`                | frontmatter の `permalink` / `slug` URL。        |
| `cascade`      | `false`                | ディレクトリ `_index` の既定 frontmatter。       |
| `docs`         | `{ enabled: true }`    | JSDoc / TSDoc から API ドキュメントを生成。      |
| `transformers` | `[]`                   | 独自の Markdown AST 変換。                       |

## 静的サイト生成

SSG は既定でオンです。`srcDir` 以下のすべての Markdown が、既定テーマ、ナビ、検索 UI 付きの静的 HTML ページになります。いま読んでいるサイトも、まさにこの方法で生成しています。

```ts
import { defineConfig } from "vite-plus";
import { oxContent, defineTheme, defaultTheme } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "content",
      outDir: "dist/docs",
      ssg: {
        siteName: "Ox Content",
        siteUrl: "https://example.com",
        lastUpdated: true,
        theme: defineTheme({
          extends: defaultTheme,
          sidebar: [
            {
              text: "Guide",
              items: [{ text: "Getting Started", link: "/getting-started.md" }],
            },
          ],
        }),
      },
    }),
  ],
});
```

| オプション        | 既定           | 目的                                                                                                                 |
| ----------------- | -------------- | -------------------------------------------------------------------------------------------------------------------- |
| `enabled`         | `true`         | `.md` モジュールだけ残すときは `ssg: false`。ホスト側で [コンポーネント CSS](./component-styles.md) を import する。 |
| `extension`       | `".html"`      | 生成ページの拡張子。                                                                                                 |
| `routePrefix`     | —              | `base` や `outDir` を変えずにページルートをマウントする。                                                            |
| `clean`           | `false`        | 書き出す前に生成物を消す。                                                                                           |
| `bare`            | `false`        | ナビなしの、テーマなし HTML を出す。                                                                                 |
| `render`          | —              | 文書全体を所有する JSX コンポーネント。                                                                              |
| `lang`            | `"en"`         | `<html>` の `lang` 属性（bare モード）。                                                                             |
| `head`            | —              | `<head>` に足す生マークアップ（bare モード）。                                                                       |
| `bodyStart`       | —              | `<body>` の直後に足す生マークアップ（bare モード）。                                                                 |
| `bodyEnd`         | —              | `</body>` の直前に足す生マークアップ（bare モード）。                                                                |
| `siteName`        | —              | `<title>` の接尾辞と OG サイト名。                                                                                   |
| `siteUrl`         | —              | 絶対 OG URL、canonical、hreflang に使うオリジン。[SEO](./seo.md)。                                                   |
| `headValidation`  | `false`        | 不正な head デスクリプタの `warn` / `strict`。[SEO](./seo.md)。                                                      |
| `ogImage`         | —              | 静的フォールバックの OG 画像 URL。                                                                                   |
| `generateOgImage` | `false`        | ページごとの OG 画像（後述）。                                                                                       |
| `lastUpdated`     | `false`        | ページごとの git 最終コミット時刻を出す。                                                                            |
| `contributors`    | `false`        | ページごとの一意な git 作者。[コントリビューター](./contributors.md) を見てください。                                |
| `pagination`      | `false`        | 記事のあとに前へ / 次へリンク。                                                                                      |
| `breadcrumbs`     | `false`        | サイトルートからサイドバー祖先までの道筋。                                                                           |
| `jsonLd`          | `false`        | TechArticle / WebSite / BreadcrumbList の JSON-LD。                                                                  |
| `readerChrome`    | `false`        | コピー、外部リンクアイコン、先頭へ戻る。                                                                             |
| `localeSwitcher`  | `false`        | i18n ロケールがあるときのヘッダーロケールドロップダウン。                                                            |
| `a11y`            | `false`        | スキップリンクと印刷スタイル。                                                                                       |
| `notFound`        | `false`        | テーマ付き 404。 [カスタム 404](./not-found.md) を見てください。                                                     |
| `team`            | `false`        | `layout: team` のメンバーカード。[チーム](./team.md) を見てください。                                                |
| `blog`            | `false`        | ページ送り索引、著者、タグ、アーカイブ、任意の外部フィード。[ブログ](./blog.md) を見てください。                     |
| `sectionIndex`    | `false`        | `index.md` がないディレクトリ向けの生成一覧。[セクション索引ページ](./section-index.md) を見てください。             |
| `pageChrome`      | `false`        | ページ単位の frontmatter chrome フラグを尊重する。                                                                   |
| `markdownSource`  | `false`        | 各ページの横に元の Markdown を公開する。[Markdown ソースの併記](./markdown-source.md)。                              |
| `theme`           | `defaultTheme` | `defineTheme()` によるテーマ設定。                                                                                   |
| `navigation`      | 派生           | ファイルツリーの代わりに明示的なナビグループ。                                                                       |

`ssg.routePrefix` はデプロイの `base` やルートのホストファイルを動かさずに、Markdown ページルートを `/blog` のようなパスへマウントします。`blog`、`/blog`、`/blog/` はどれも `/blog` 配下になります。ページ HTML とページ単位のアセットはプレフィックスに従い、`_redirects`、`_headers`、ルートのフィード、サイトマップ index は `outDir` に残ります。`base` は公開 URL のプレフィックスのままです。[パーマリンク](./permalinks.md) がオンのとき、frontmatter の `permalink` が勝ちます。

```ts
oxContent({
  srcDir: "content/blog",
  outDir: "build",
  ssg: { routePrefix: "/blog" },
  redirects: { netlify: true },
  feeds: { path: "/" },
});
```

これは `build/blog/first-post/index.html` を書き、`build/_redirects` と `build/feed.xml` はデプロイルートに残します。

テーマ — 色、フォント、ヘッダー、フッター、サイドバー、独自 CSS、オプトインのページアウトライン（`theme.aside`、既定 `false`）— は別トピックです。[テーマ](../theming.md#ページアウトライン) を見てください。

## 独自テーマコンポーネント

`ssg.render` は文書全体を JSX コンポーネントに渡します。コンポーネントは `<html>` から下を所有するので、`theme`、`bare`、head メタデータのオプションは効きません。書いていないものは注入されません。

```tsx
import { createTheme, usePageProps, useSiteConfig } from "@ox-content/vite-plugin";

function DefaultLayout({ children }) {
  const page = usePageProps();
  const site = useSiteConfig();
  return (
    <html lang="ja">
      <head>
        <title>{`${page.title} | ${site.name}`}</title>
        <link rel="stylesheet" href="/assets/site.css" />
      </head>
      <body>{children}</body>
    </html>
  );
}

oxContent({
  ssg: { render: createTheme({ layouts: { default: DefaultLayout } }) },
});
```

`createTheme()` は各ページの `layout` frontmatter が指すレイアウトを選び、なければ `default` です。レイアウト内では `usePageProps()` が現在ページを、`useSiteConfig()` がナビと他の全ページを含むサイト全体設定を返します。

これは組み込み JSX ランタイムを使うので、[MDX と JSX](../mdx.md#テーマ内の静的-jsx) のとおり `jsxImportSource` を設定してください。

## Bare モード

`bare: true` は、ナビ、レイアウトシェル、テーマスタイルなしで描画した Markdown 本文を出します。プロジェクトが独自のデザインシステムを持つとき、または JavaScript なしのベースラインを測るときに使います。

bare ページでも、プラグインがすでに計算している head メタデータは載ります。説明、`og:*` と `twitter:*` タグ、canonical リンク、生成 OG 画像です。言うことがあるときだけ出ます。説明も `siteUrl` も OG 画像もないページは、bare モードが昔から出してきた最小文書そのものなので、サイズのベースラインは正直なままです。

それ以外は自分で注入します。

```ts
oxContent({
  ssg: {
    bare: true,
    lang: "ja",
    siteUrl: "https://example.com",
    head: '<link rel="stylesheet" href="/assets/site.css">',
    bodyStart: "<header>…</header><main>",
    bodyEnd: "</main><footer>…</footer>",
  },
});
```

`siteUrl` があると `<link rel="canonical">` と絶対 `og:url` がオンになります。なければ推測せず、それらのタグを省きます。

## OG 画像

ビルド時にページごとのソーシャルプレビュー画像を生成します。

```ts
oxContent({
  ogImage: true,
  ssg: {
    generateOgImage: true,
    siteUrl: "https://example.com",
  },
});
```

各ページはタイトルと説明から描画した画像を得ます。このページの生成画像は次のように見えます。

![このページの生成 Open Graph 画像](/screenshots/og-image-example.png)

| `ogImageOptions`            | 既定         | 目的                                                             |
| --------------------------- | ------------ | ---------------------------------------------------------------- |
| `renderer`                  | `"chromium"` | `"chromium"` はブラウザ CSS 互換、`"satori"` は高速な SVG 描画。 |
| `template`                  | 組み込み     | 独自テンプレート: `.ts`、`.vue`、`.svelte`、`.tsx`。             |
| `width`                     | `1200`       | 画像幅（ピクセル）。                                             |
| `height`                    | `630`        | 画像高さ（ピクセル）。                                           |
| `cache`                     | `true`       | 変わっていないページの再描画を飛ばす。                           |
| `concurrency`               | CPU 依存     | 並列の画像描画。既定は `min(4, cores - 1)`。                     |
| `satori.fonts`              | `[]`         | Satori に渡すフォントファイル（`.ttf`、`.otf`、`.woff`）。       |
| `satori.systemFontFallback` | `true`       | `satori.fonts` が空のとき、よくある OS フォントパスを探す。      |

描画した画像は `.cache/og-images` にキャッシュします。キーはテンプレートのソースとページの props のハッシュなので、変更のないページは再描画せずキャッシュからコピーします。描画コストを払うのはタイトル・説明・frontmatter が変わったページだけです。このディレクトリは gitignore されているため、復元しない CI ジョブは毎回すべてのページを描画します。`node_modules` と同じように、実行間でキャッシュしてください。

Chromium は互換性重視のレンダラーです。通常のブラウザ CSS、ローカル public
アセット、フレームワークテンプレートをページと同じ感覚で使えます。Satori は
ブラウザを起動せず HTML を SVG にし、そこから PNG を作るので、大きいサイト
ほど速くなります。その代わり、テンプレートは Satori が対応する HTML / CSS
サブセットに収め、テキスト用のフォントを少なくとも 1 つ読み込める必要があります。

```ts
oxContent({
  ogImage: true,
  ogImageOptions: {
    renderer: "satori",
    satori: {
      fonts: [{ path: "public/fonts/Inter-Regular.ttf", name: "Inter" }],
    },
  },
  ssg: {
    generateOgImage: true,
    siteUrl: "https://example.com",
  },
});
```

`bare` では画像を **生成し、参照します**。bare ページはテーマ付きページと同じ `og:image` / `twitter:image` タグを持ちます。`buildSsg()` はソースパスから画像 URL への `ogImages` マップも返すので、後処理が出力ツリーで `og-image.png` を探す必要はありません。

開発中は `/__og-viewer` が各ページの Open Graph メタデータと画像をプレビューします（`ogViewer` オプション、既定オン）。

![開発中の OG ビューア](/screenshots/og-viewer.png)

独自テンプレートはページ frontmatter を props として受け取ります。[Custom OG Image Templates](/examples/og-image-custom.md) を見てください。

## このページを編集

すべてのページに「編集を提案」リンクを付けます。オプションは `repoUrl` を渡したときだけオンになります。裸の `editThisPage: true` はリンク先がないのでオフのままです。

```ts
oxContent({
  editThisPage: {
    repoUrl: "https://github.com/ubugeeei-prod/ox-content",
    branch: "main",
    label: "Edit this page",
  },
});
```

描画されたリンクは、そのページを作ったファイルを指します。

```html
<p class="ox-edit-this-page">
  <a
    href="https://github.com/ubugeeei-prod/ox-content/edit/main/docs/content/example.md"
    target="_blank"
    rel="noopener noreferrer"
    >Edit this page</a
  >
</p>
```

編集 URL に結合する前にソースパスからプレフィックスを除く必要があるときは `rootDir` を設定します。

## コレクション

コレクションは Markdown ファイルを、遅延読み込み可能な問い合わせマニフェストとして出します。ブログ索引、変更履歴、「関連ページ」一覧に使えます。すべての Markdown を覆う既定の `content` コレクションは最初からあります。

```ts
import { queryCollection } from "virtual:ox-content/collections";

const guides = await queryCollection("content")
  .where("path", "LIKE", "/guide/%")
  .order("title", "ASC")
  .limit(10)
  .all();
```

クエリビルダ全体 — 演算子、グループ条件、frontmatter へのドットパス、`select` / `order` / `limit` — は専用ページです。[コレクション](./collections.md) を見てください。

## API ドキュメント

`docs` は JSDoc / TSDoc コメントから Markdown の API リファレンスを生成します。TypeScript 向けの `cargo doc` です。既定はオン（`docs: false` でオプトアウト）で、`srcDir` に書き出すので生成ページがサイトに混ざります。

```ts
oxContent({
  docs: {
    src: ["./src"],
    out: "content/api",
    include: ["**/*.ts"],
    exclude: ["**/*.test.*"],
    githubUrl: "https://github.com/owner/repo",
    generateNav: true,
  },
});
```

このサイトの [API リファレンス](/api/index.md) は、プラグイン自身のソースからこのパイプラインで生成しています。エントリポイント、グループ化、ソート、リンクスタイル、種類ごとの描画形式を含むオプション一式は [JSDoc から作る API ドキュメント](../jsdoc.md) です。

## 独自トランスフォーマ

`transformers` はパースと描画のあいだに Markdown AST に対して走ります。ページ本文に置きたくない、プロジェクト固有の書き換え向けです。

```ts
import type { MarkdownTransformer } from "@ox-content/vite-plugin";

const stampDrafts: MarkdownTransformer = {
  name: "stamp-drafts",
  transform(ast, context) {
    if (context.frontmatter.draft) {
      ast.children.unshift({
        type: "paragraph",
        children: [{ type: "text", value: "🚧 Draft — not published yet." }],
      });
    }
    return ast;
  },
};

oxContent({
  transformers: [stampDrafts],
});
```

各トランスフォーマはパース済み AST と `{ filePath, frontmatter, options }` を受け取り、（置き換えたかもしれない）AST を返します。トランスフォーマは配列順で合成されます。

## 独自ホスト（`ssg: false`）

ページテンプレートを自分で持つホストでも、リソース指紋、Markdown 併記、
フィード、sitemap、git lastmod は再利用できます。`planSsgOutputs` と
対応する writer を呼んでください。[SSG 出力プリミティブ](./ssg-output.md)
を見てください。これらのフィールドを解決したいときは
`ssg: { enabled: false, markdownSource, lastUpdated, siteUrl }` を使い、
boolean の `ssg: false` はそれらを消します。

## 関連

- [SSG 出力プリミティブ](./ssg-output.md) — 既定テーマなしで出力を計画して書き出す。
- [前へ / 次へ](./pagination.md) — オプトインの前後ページリンク。
- [パンくず](./breadcrumbs.md) — サイトルートからサイドバー祖先までのオプトインの道筋。
- [リーダー chrome](./reader-chrome.md) — オプトインのコピー、外部アイコン、先頭へ戻る。
- [ロケールスイッチャー](./locale-switcher.md) — オプトインのヘッダーロケール一覧。
- [アクセシビリティ](./a11y.md) — オプトインのスキップリンクと印刷スタイル。
- [チーム / メンバー](./team.md) — `layout: team` のオプトインメンバーカード。
- [セクション索引ページ](./section-index.md) — `index.md` がないディレクトリ向けのオプトイン一覧。
- [ヘッダー chrome](./header-chrome.md) — オプトインのヘッダーナビ、告知、ページフラグ。
- [Sitemap / robots / llms.txt](./site-maps.md) — オプトインのクロール用マニフェスト。
- [Markdown ソースの併記](./markdown-source.md) — 各ページの横に元の Markdown をオプトインで出す。
- [リダイレクトとエイリアス](./redirects.md) — オプトインの静的 HTML リダイレクト。
- [RSS / Atom / JSON フィード](./feeds.md) — オプトインのコレクションフィード。
- [PWA マニフェストとサービスワーカー](./pwa.md) — オプトインのマニフェストと保守的なオフラインキャッシュ（クライアント JS を追加）。
- [セルフホスト Iconify CSS](./icons.md) — 使った Iconify アイコン向けのオプトイン CSS マスク。
- [テーマ](../theming.md) — SSG が使うテーマシステム。
- [JSDoc から作る API ドキュメント](../jsdoc.md) — `docs` オプションの全体。
- [国際化](../i18n.md) — SSG の上のロケール対応サイト。
- [パーマリンクと Cascade](./permalinks.md) — 独自 URL とディレクトリ既定 frontmatter。
