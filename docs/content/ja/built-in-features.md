---
title: 組み込み機能
description: @ox-content/vite-plugin で使える既定機能とオプトイン機能。
---

# 組み込み機能

Ox Content は、よく使うドキュメントの挙動を既定で載せ、非標準の Markdown や追加のサイト挙動はオプトインにします。

`false` または省略はオフ、`true` はデフォルトでオン、オブジェクトはオンにしたうえで指定したフィールドだけ上書きします。

このドキュメントサイト自体が Ox Content でできています。英語の各ガイドには実例がインラインで描画されます。日本語ガイドは契約と設定を先に揃えています。

## 機能ガイド

| ガイド                                                   | 内容                                                                           |
| -------------------------------------------------------- | ------------------------------------------------------------------------------ |
| [Markdown の土台](./built-in/markdown.md)                | GFM、表、タスクリスト、脚注、autolink、frontmatter、TOC                        |
| [構文拡張](./built-in/syntax-extensions.md)              | 絵文字ショートコード、Wiki リンク、属性構文、CJK 強調                          |
| [カスタムコンテナ](./built-in/containers.md)             | オプトインの `::: tip` / `::: details`                                         |
| [カード](./built-in/cards.md)                            | オプトインの `::: card` / `::: link-card` / `::: card-grid`                    |
| [手順リスト](./built-in/steps.md)                        | オプトインの `::: steps`                                                       |
| [ファイル取り込み](./built-in/includes.md)               | オプトインの `<!-- @include -->`                                               |
| [ファイルツリー](./built-in/file-tree.md)                | オプトインの静的ディレクトリ図                                                 |
| [インラインバッジ](./built-in/badges.md)                 | オプトインの `{badge:tip}`                                                     |
| [画像](./built-in/images.md)                             | 図、キャプション、遅延読み込み、安全な寸法                                     |
| [コードブロック](./built-in/code-blocks.md)              | ハイライト、注釈、ソース取り込み                                               |
| [埋め込み](./built-in/embeds.md)                         | GitHub / OG カード、パッケージマネージャタブ、YouTube、SNS                     |
| [Mermaid](./built-in/mermaid.md)                         | フェンスを静的 SVG に描画                                                      |
| [数式](./built-in/math.md)                               | オプトインの `$…$` / `$$…$$`                                                   |
| [検索](./built-in/search.md)                             | 静的 BM25 インデックスとクライアント API                                       |
| [コレクション](./built-in/collections.md)                | Markdown を SQL 風ビルダで問い合わせ                                           |
| [品質チェック](./built-in/quality-checks.md)             | lint、型チェック、docs テスト、HTML サニタイズ                                 |
| [サイト生成](./built-in/site-generation.md)              | SSG、OG 画像、編集リンク、API ドキュメント                                     |
| [前へ / 次へ](./built-in/pagination.md)                  | サイドバー順の前後リンク                                                       |
| [パンくず](./built-in/breadcrumbs.md)                    | ルートからサイドバー祖先までの道筋                                             |
| [リーダー chrome](./built-in/reader-chrome.md)           | コピー、外部リンクアイコン、先頭へ戻る                                         |
| [ロケールスイッチャー](./built-in/locale-switcher.md)    | 設定したロケールへのヘッダー導線                                               |
| [アクセシビリティ](./built-in/a11y.md)                   | スキップリンクと印刷スタイル                                                   |
| [ヘッダー chrome](./built-in/header-chrome.md)           | ナビ、告知バー、ページ単位の chrome                                            |
| [Sitemap / robots / llms.txt](./built-in/site-maps.md)   | クロール用マニフェスト                                                         |
| [下書き / 非公開 / 予約公開](./built-in/drafts.md)       | frontmatter の公開状態                                                         |
| [パーマリンクと Cascade](./built-in/permalinks.md)       | 独自 URL とディレクトリ既定 frontmatter                                        |
| [リダイレクトとエイリアス](./built-in/redirects.md)      | 静的 HTML リダイレクト                                                         |
| [カスタム 404](./built-in/not-found.md)                  | テーマ付き 404                                                                 |
| [RSS / Atom / JSON フィード](./built-in/feeds.md)        | コレクションからフィードを出力                                                 |
| [PWA マニフェストとサービスワーカー](./built-in/pwa.md)  | Web アプリマニフェストと保守的なオフラインキャッシュ（クライアント JS を追加） |
| [タクソノミー](./built-in/taxonomies.md)                 | タグ / カテゴリの用語ページと関連ページ                                        |
| [ドキュメントのバージョン管理](./built-in/versioning.md) | プレフィックス、凍結スナップショット、切替 UI                                  |
| [チーム / メンバー](./built-in/team.md)                  | `layout: team` の静的カード                                                    |

## 既定とオプトイン

| 領域                 | オプション                                                                     | 既定          | ガイド                                                   |
| -------------------- | ------------------------------------------------------------------------------ | ------------- | -------------------------------------------------------- |
| Markdown 土台        | `gfm`, `footnotes`, `tables`, `taskLists`, `strikethrough`, `autolinks`        | `true`        | [Markdown の土台](./built-in/markdown.md)                |
| ページメタ           | `frontmatter`                                                                  | `true`        | [Markdown の土台](./built-in/markdown.md)                |
| ナビゲーション       | `toc`, `tocMaxDepth`                                                           | `true`, `3`   | [Markdown の土台](./built-in/markdown.md)                |
| 静的サイト           | `ssg`                                                                          | `{ enabled }` | [サイト生成](./built-in/site-generation.md)              |
| API ドキュメント     | `docs`                                                                         | `{ enabled }` | [サイト生成](./built-in/site-generation.md)              |
| 検索                 | `search`                                                                       | `{ enabled }` | [検索](./built-in/search.md)                             |
| コレクション         | `collections`                                                                  | `content`     | [コレクション](./built-in/collections.md)                |
| 静的埋め込み         | `embeds.github`, `embeds.openGraph`                                            | `true`        | [埋め込み](./built-in/embeds.md)                         |
| オプトイン埋め込み   | `embeds.pm` ほか                                                               | `false`       | [埋め込み](./built-in/embeds.md)                         |
| 構文ハイライト       | `highlight`                                                                    | `false`       | [コードブロック](./built-in/code-blocks.md)              |
| コード執筆           | `codeAnnotations`, `codeImports`                                               | `false`       | [コードブロック](./built-in/code-blocks.md)              |
| 追加構文             | `wikiLinks`, `emojiShortcodes`, `attrs`, `cjkEmphasis`, `containers`, `badges` | `false`       | [構文拡張](./built-in/syntax-extensions.md)              |
| ファイル取り込み     | `includes`                                                                     | `false`       | [ファイル取り込み](./built-in/includes.md)               |
| カード               | `cards`                                                                        | `false`       | [カード](./built-in/cards.md)                            |
| 手順リスト           | `steps`                                                                        | `false`       | [手順リスト](./built-in/steps.md)                        |
| ファイルツリー       | `fileTree`                                                                     | `false`       | [ファイルツリー](./built-in/file-tree.md)                |
| 画像                 | `images`                                                                       | `false`       | [画像](./built-in/images.md)                             |
| 図                   | `mermaid`                                                                      | `false`       | [Mermaid](./built-in/mermaid.md)                         |
| 数式                 | `math`                                                                         | `false`       | [数式](./built-in/math.md)                               |
| OG 画像              | `ogImage`                                                                      | `false`       | [サイト生成](./built-in/site-generation.md)              |
| HTML 安全            | `sanitize`                                                                     | `false`       | [品質チェック](./built-in/quality-checks.md)             |
| 編集リンク           | `editThisPage`                                                                 | `false`       | [サイト生成](./built-in/site-generation.md)              |
| ページ送り           | `ssg.pagination`                                                               | `false`       | [前へ / 次へ](./built-in/pagination.md)                  |
| パンくず             | `ssg.breadcrumbs`                                                              | `false`       | [パンくず](./built-in/breadcrumbs.md)                    |
| リーダー chrome      | `ssg.readerChrome`                                                             | `false`       | [リーダー chrome](./built-in/reader-chrome.md)           |
| ロケールスイッチャー | `ssg.localeSwitcher`                                                           | `false`       | [ロケールスイッチャー](./built-in/locale-switcher.md)    |
| アクセシビリティ     | `ssg.a11y`                                                                     | `false`       | [アクセシビリティ](./built-in/a11y.md)                   |
| ヘッダー chrome      | `theme.nav`, `theme.announcement`, `ssg.pageChrome`                            | オフ          | [ヘッダー chrome](./built-in/header-chrome.md)           |
| クロール             | `siteMaps`                                                                     | `false`       | [Sitemap / robots / llms.txt](./built-in/site-maps.md)   |
| 公開状態             | `publishState`                                                                 | `false`       | [下書き / 非公開 / 予約公開](./built-in/drafts.md)       |
| パーマリンク         | `permalinks`                                                                   | `false`       | [パーマリンクと Cascade](./built-in/permalinks.md)       |
| frontmatter 継承     | `cascade`                                                                      | `false`       | [パーマリンクと Cascade](./built-in/permalinks.md)       |
| リダイレクト         | `redirects`                                                                    | `false`       | [リダイレクトとエイリアス](./built-in/redirects.md)      |
| カスタム 404         | `ssg.notFound`                                                                 | `false`       | [カスタム 404](./built-in/not-found.md)                  |
| フィード             | `feeds`                                                                        | `false`       | [RSS / Atom / JSON フィード](./built-in/feeds.md)        |
| PWA                  | `pwa`                                                                          | `false`       | [PWA マニフェストとサービスワーカー](./built-in/pwa.md)  |
| ドキュメント版       | `versions`                                                                     | `false`       | [ドキュメントのバージョン管理](./built-in/versioning.md) |
| タクソノミー         | `taxonomies`                                                                   | `false`       | [タクソノミー](./built-in/taxonomies.md)                 |
| チームページ         | `ssg.team`                                                                     | `false`       | [チーム / メンバー](./built-in/team.md)                  |
| コード検査           | `codeBlockLint`, `codeBlockTypecheck`, `docsTests`                             | `false`       | [品質チェック](./built-in/quality-checks.md)             |
| 独自パイプライン     | `transformers`                                                                 | `[]`          | [サイト生成](./built-in/site-generation.md)              |

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

コピーできる執筆例はリポジトリの `examples/builtin-features/content/` にあります。[事例](./examples/index.md) も見てください。

英語の一覧は [Built-in Features](/built-in-features.md) です。
