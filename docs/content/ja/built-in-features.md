---
title: 組み込み機能
description: @ox-content/vite-plugin で使える既定機能とオプトイン機能。
---

# 組み込み機能

Ox Content は、よく使うドキュメントの挙動を既定で載せ、非標準の Markdown や HTML 拡張はオプトインにします。

下の既定は `@ox-content/vite-plugin` と一致します。速い静的ベースライン向けです。パース、静的埋め込み、ソース docs、検索インデックスは変換時またはビルド時に走り、追加構文と実行時挙動は明示的にオンにします。

`false` または省略はオフ、`true` は既定でオン、オブジェクトはオンにしたうえで指定したフィールドだけ上書きします。

このドキュメントサイト自体が Ox Content でできています。下の機能ガイドは説明するだけでなく、機能をオンにして **実例をインラインで描画します**。日本語ガイドは英語と同じ情報を載せます。

## 機能ガイド

| ガイド                                                   | 内容                                                                           |
| -------------------------------------------------------- | ------------------------------------------------------------------------------ |
| [Markdown の土台](./built-in/markdown.md)                | GFM、表、タスクリスト、脚注、autolink、frontmatter、TOC                        |
| [見出しパーマリンク](./built-in/heading-permalinks.md)   | 生成済み見出し id を再利用するオプトインの可視 `#` リンク                      |
| [構文拡張](./built-in/syntax-extensions.md)              | 絵文字ショートコード、Wiki リンク、属性構文、CJK 強調                          |
| [カスタムコンテナ](./built-in/containers.md)             | オプトインの `::: tip` / `::: details`                                         |
| [カード](./built-in/cards.md)                            | オプトインの `::: card` / `::: link-card` / `::: card-grid`                    |
| [手順リスト](./built-in/steps.md)                        | オプトインの `::: steps`                                                       |
| [ファイル取り込み](./built-in/includes.md)               | オプトインの `<!-- @include -->`                                               |
| [ファイルツリー](./built-in/file-tree.md)                | オプトインの静的ディレクトリ図                                                 |
| [インラインバッジ](./built-in/badges.md)                 | オプトインの `{badge:tip}`                                                     |
| [マジックリンク](./built-in/magic-links.md)              | オプトインの `{link:@user}` / エイリアス / `label\|url` リッチリンク           |
| [画像](./built-in/images.md)                             | 図、キャプション、遅延読み込み、安全な寸法                                     |
| [ページリソース](./built-in/resources.md)                | ページバンドル資産とリサイズ・クロップ・形式変換                               |
| [コードブロック](./built-in/code-blocks.md)              | ハイライト、注釈、ソース取り込み                                               |
| [コードグループ](./built-in/code-groups.md)              | オプトインの VitePress 風 `::: code-group` フェンスタブ                        |
| [埋め込み](./built-in/embeds.md)                         | GitHub / OG カード、パッケージマネージャタブ、YouTube、SNS                     |
| [Mermaid](./built-in/mermaid.md)                         | フェンスを静的 SVG に描画                                                      |
| [数式](./built-in/math.md)                               | オプトインの `$…$` / `$$…$$`。任意依存の KaTeX で組版                          |
| [検索](./built-in/search.md)                             | 静的 BM25 インデックスとクライアント API                                       |
| [コレクション](./built-in/collections.md)                | Markdown を SQL 風ビルダで問い合わせ                                           |
| [品質チェック](./built-in/quality-checks.md)             | lint、型チェック、docs テスト、HTML サニタイズ                                 |
| [型ホバー](./built-in/typed-hover.md)                    | `twoslash` フェンスのビルド時 TypeScript 型オーバーレイ                        |
| [サイト生成](./built-in/site-generation.md)              | SSG、OG 画像、編集リンク、API ドキュメント                                     |
| [コンポーネント CSS](./built-in/component-styles.md)     | `ssg: false` と `transformAllPlugins()` 向けの公式 CSS                         |
| [ページ head](./built-in/page-head.md)                   | ビルド時の Unhead 互換 title / meta / link / JSON-LD API                       |
| [SEO](./built-in/seo.md)                                 | その API 上の canonical、robots、hreflang、検証                                |
| [前へ / 次へ](./built-in/pagination.md)                  | サイドバー順の前後リンク                                                       |
| [パンくず](./built-in/breadcrumbs.md)                    | ルートからサイドバー祖先までの道筋                                             |
| [JSON-LD](./built-in/json-ld.md)                         | オプトインの TechArticle / WebSite / BreadcrumbList                            |
| [リーダー chrome](./built-in/reader-chrome.md)           | コピー、外部リンクアイコン、先頭へ戻る                                         |
| [ロケールスイッチャー](./built-in/locale-switcher.md)    | 設定したロケールへのヘッダー導線                                               |
| [アクセシビリティ](./built-in/a11y.md)                   | スキップリンクと印刷スタイル                                                   |
| [ヘッダー chrome](./built-in/header-chrome.md)           | ナビ、告知バー、ページ単位の chrome                                            |
| [Sitemap / robots / llms.txt](./built-in/site-maps.md)   | クロール用マニフェスト                                                         |
| [Markdown ソースの併記](./built-in/markdown-source.md)   | 生成 HTML の横に元の Markdown をオプトインで書き出す                           |
| [下書き / 非公開 / 予約公開](./built-in/drafts.md)       | frontmatter の公開状態                                                         |
| [パーマリンクと Cascade](./built-in/permalinks.md)       | 独自 URL とディレクトリ既定 frontmatter                                        |
| [リダイレクトとエイリアス](./built-in/redirects.md)      | 静的 HTML リダイレクト                                                         |
| [カスタム 404](./built-in/not-found.md)                  | テーマ付き 404                                                                 |
| [RSS / Atom / JSON フィード](./built-in/feeds.md)        | コレクションからフィードを出力                                                 |
| [ブログ](./built-in/blog.md)                             | ページ送り索引、著者、タグ、アーカイブ、任意の外部フィード                     |
| [PWA マニフェストとサービスワーカー](./built-in/pwa.md)  | Web アプリマニフェストと保守的なオフラインキャッシュ（クライアント JS を追加） |
| [セルフホスト Iconify CSS](./built-in/icons.md)          | 使った Iconify アイコンのオプトイン CSS マスク（api.iconify.design なし）      |
| [タクソノミー](./built-in/taxonomies.md)                 | タグ / カテゴリの用語ページと関連ページ                                        |
| [ドキュメントのバージョン管理](./built-in/versioning.md) | プレフィックス、凍結スナップショット、切替 UI                                  |
| [チーム / メンバー](./built-in/team.md)                  | `layout: team` の静的カード                                                    |
| [Git コントリビューター](./built-in/contributors.md)     | 各記事の下に一意の git 作者を出すオプトイン                                    |
| [セクション索引ページ](./built-in/section-index.md)      | `index.md` がないディレクトリ向けの生成一覧                                    |

## 既定とオプトイン

| 領域                 | オプション                                                                                                    | 既定               | ガイド                                                   |
| -------------------- | ------------------------------------------------------------------------------------------------------------- | ------------------ | -------------------------------------------------------- |
| Markdown 土台        | `gfm`, `footnotes`, `tables`, `taskLists`, `strikethrough`, `autolinks`                                       | `true`             | [Markdown の土台](./built-in/markdown.md)                |
| 意味的な脚注         | `semanticFootnotes`                                                                                           | `false`            | [Markdown の土台](./built-in/markdown.md)                |
| ページメタ           | `frontmatter`                                                                                                 | `true`             | [Markdown の土台](./built-in/markdown.md)                |
| ナビゲーション       | `toc`, `tocMaxDepth`                                                                                          | `true`, `3`        | [Markdown の土台](./built-in/markdown.md)                |
| 見出しリンク         | `headingPermalinks` / `theme.headingPermalink`                                                                | `false`, `"hover"` | [見出しパーマリンク](./built-in/heading-permalinks.md)   |
| 静的サイト           | `ssg`                                                                                                         | `{ enabled }`      | [サイト生成](./built-in/site-generation.md)              |
| API ドキュメント     | `docs`                                                                                                        | `{ enabled }`      | [サイト生成](./built-in/site-generation.md)              |
| 検索                 | `search`                                                                                                      | `{ enabled }`      | [検索](./built-in/search.md)                             |
| コレクション         | `collections`                                                                                                 | `content`          | [コレクション](./built-in/collections.md)                |
| 静的埋め込み         | `embeds.github`, `embeds.openGraph`                                                                           | `true`             | [埋め込み](./built-in/embeds.md)                         |
| オプトイン埋め込み   | `embeds.pm`, `embeds.twitter`, `embeds.bluesky`, `embeds.spotify`, `embeds.stackBlitz`, `embeds.webContainer` | `false`            | [埋め込み](./built-in/embeds.md)                         |
| 構文ハイライト       | `highlight`                                                                                                   | `false`            | [コードブロック](./built-in/code-blocks.md)              |
| コード執筆           | `codeAnnotations`, `codeImports`                                                                              | `false`            | [コードブロック](./built-in/code-blocks.md)              |
| コードグループ       | `codeGroups`                                                                                                  | `false`            | [コードグループ](./built-in/code-groups.md)              |
| 追加構文             | `wikiLinks`, `emojiShortcodes`, `attrs`, `cjkEmphasis`, `containers`, `badges`, `magicLinks`                  | `false`            | [構文拡張](./built-in/syntax-extensions.md)              |
| ファイル取り込み     | `includes`                                                                                                    | `false`            | [ファイル取り込み](./built-in/includes.md)               |
| カード               | `cards`                                                                                                       | `false`            | [カード](./built-in/cards.md)                            |
| 手順リスト           | `steps`                                                                                                       | `false`            | [手順リスト](./built-in/steps.md)                        |
| ファイルツリー       | `fileTree`                                                                                                    | `false`            | [ファイルツリー](./built-in/file-tree.md)                |
| 画像                 | `images`                                                                                                      | `false`            | [画像](./built-in/images.md)                             |
| ページリソース       | `resources`                                                                                                   | `false`            | [ページリソース](./built-in/resources.md)                |
| 図                   | `mermaid`                                                                                                     | `false`            | [Mermaid](./built-in/mermaid.md)                         |
| 数式                 | `math`                                                                                                        | `false`            | [数式](./built-in/math.md)                               |
| OG 画像              | `ogImage`                                                                                                     | `false`            | [サイト生成](./built-in/site-generation.md)              |
| HTML 安全            | `sanitize`                                                                                                    | `false`            | [品質チェック](./built-in/quality-checks.md)             |
| 編集リンク           | `editThisPage`                                                                                                | `false`            | [サイト生成](./built-in/site-generation.md)              |
| ページ送り           | `ssg.pagination`                                                                                              | `false`            | [前へ / 次へ](./built-in/pagination.md)                  |
| パンくず             | `ssg.breadcrumbs` / `theme.breadcrumbs`                                                                       | `false`            | [パンくず](./built-in/breadcrumbs.md)                    |
| ページ head          | `renderHead`                                                                                                  | ビルド時           | [ページ head](./built-in/page-head.md)                   |
| SEO タグ             | `ssg.siteUrl`、frontmatter `robots` / `canonical`                                                             | 設定時のみ         | [SEO](./built-in/seo.md)                                 |
| head 検証            | `ssg.headValidation`                                                                                          | `false`            | [SEO](./built-in/seo.md)                                 |
| 構造化データ         | `ssg.jsonLd`                                                                                                  | `false`            | [JSON-LD](./built-in/json-ld.md)                         |
| リーダー chrome      | `ssg.readerChrome`                                                                                            | `false`            | [リーダー chrome](./built-in/reader-chrome.md)           |
| ロケールスイッチャー | `ssg.localeSwitcher`                                                                                          | `false`            | [ロケールスイッチャー](./built-in/locale-switcher.md)    |
| アクセシビリティ     | `ssg.a11y`                                                                                                    | `false`            | [アクセシビリティ](./built-in/a11y.md)                   |
| ヘッダー chrome      | `theme.nav`, `theme.announcement`, `ssg.pageChrome`                                                           | オフ               | [ヘッダー chrome](./built-in/header-chrome.md)           |
| クロール             | `siteMaps`                                                                                                    | `false`            | [Sitemap / robots / llms.txt](./built-in/site-maps.md)   |
| Markdown ソース      | `ssg.markdownSource`                                                                                          | `false`            | [Markdown ソースの併記](./built-in/markdown-source.md)   |
| 公開状態             | `publishState`                                                                                                | `false`            | [下書き / 非公開 / 予約公開](./built-in/drafts.md)       |
| パーマリンク         | `permalinks`                                                                                                  | `false`            | [パーマリンクと Cascade](./built-in/permalinks.md)       |
| frontmatter 継承     | `cascade`                                                                                                     | `false`            | [パーマリンクと Cascade](./built-in/permalinks.md)       |
| リダイレクト         | `redirects`                                                                                                   | `false`            | [リダイレクトとエイリアス](./built-in/redirects.md)      |
| カスタム 404         | `ssg.notFound`                                                                                                | `false`            | [カスタム 404](./built-in/not-found.md)                  |
| フィード             | `feeds`                                                                                                       | `false`            | [RSS / Atom / JSON フィード](./built-in/feeds.md)        |
| ブログ               | `blog` / `ssg.blog`                                                                                           | `false`            | [ブログ](./built-in/blog.md)                             |
| PWA                  | `pwa`                                                                                                         | `false`            | [PWA マニフェストとサービスワーカー](./built-in/pwa.md)  |
| セルフホストアイコン | `icons`                                                                                                       | `false`            | [セルフホスト Iconify CSS](./built-in/icons.md)          |
| ドキュメント版       | `versions`                                                                                                    | `false`            | [ドキュメントのバージョン管理](./built-in/versioning.md) |
| タクソノミー         | `taxonomies`                                                                                                  | `false`            | [タクソノミー](./built-in/taxonomies.md)                 |
| チームページ         | `ssg.team`                                                                                                    | `false`            | [チーム / メンバー](./built-in/team.md)                  |
| Git 作者             | `ssg.contributors`                                                                                            | `false`            | [Git コントリビューター](./built-in/contributors.md)     |
| セクション索引       | `ssg.sectionIndex`                                                                                            | `false`            | [セクション索引ページ](./built-in/section-index.md)      |
| コード検査           | `codeBlockLint`, `codeBlockTypecheck`, `docsTests`                                                            | `false`            | [品質チェック](./built-in/quality-checks.md)             |
| 型ホバー             | `typedHover`                                                                                                  | `false`            | [型ホバー](./built-in/typed-hover.md)                    |
| 独自パイプライン     | `transformers`                                                                                                | `[]`               | [サイト生成](./built-in/site-generation.md)              |

タブと YouTube 埋め込みにオプションはありません。SSG と dev preview では常に処理されます。[埋め込み](./built-in/embeds.md) を見てください。

## 設定例

```ts
import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      highlight: true,
      emojiShortcodes: true,
      codeAnnotations: {
        notation: "both",
      },
      embeds: {
        pm: { sync: true },
        twitter: { fetch: true },
        bluesky: true,
      },
    }),
  ],
});
```

どのオプションも同じ約束です。`false` は機能を切り、`true` は既定でオン、オブジェクトはオンにしたうえで指定したフィールドだけ上書きします。

コピーできる執筆例はリポジトリの `examples/builtin-features/content/` にあります。[事例](./examples/index.md) のページは、いくつかの機能を実行可能なプロジェクトで見せます。
