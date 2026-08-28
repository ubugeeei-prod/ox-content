---
title: テーマ
description: Theme API でドキュメントサイトの見た目を変える。
---

# テーマ

ox-content は、ドキュメントサイトの見た目を変えられる柔軟な Theme API を提供します。簡単なカスタムは CSS 変数、完全な制御は JSX テーマです。

いちから作らなくても、[テーマプリセット](./theme-presets.md) の **公式カタログ** に 27 スキンと 45 配色があります。`@ox-content/theme-*` と `@ox-content/theme-color-*` を `ssg.theme` で合成します。互換契約（必須トークン、ライトとダーク、スクリーンショット、スキンが色をハードコードしてはいけない規則）は [パッケージを書く](./theme-presets.md#パッケージを書く) を見てください。

## 安定した MPA ナビゲーション

組み込みテーマは、保存されたライト、ダーク、またはシステムの配色を初回描画前に復元します。cross-document View Transitions に対応するブラウザでは、同一オリジンのページ遷移中も現在の画面を維持し、次の生成ページを読み込みます。これは MPA のままです。リンクは通常どおり文書遷移し、未対応ブラウザはネイティブのフォールバックを使います。

`prefers-reduced-motion: reduce` ではトランジションは自動でオフになります。テーマ単位で切るときは `viewTransitions: false` です。

```ts
defineTheme({
  viewTransitions: false,
});
```

外部リンク、ダウンロード、ハッシュのみのリンクは、普通のブラウザ挙動のままです。

## ローカライズしたサイドバーラベル

すべてのサイドバー `text` は、1 つの文字列でもロケールマップでも構いません。同じマップはトップレベルグループ、リンク付き親、入れ子項目で動きます。

```ts
defineTheme({
  sidebar: [
    {
      text: { en: "Guide", ja: "ガイド" },
      collapsed: true,
      stickyCollapsed: true,
      items: [
        {
          text: { en: "Built-in features", ja: "組み込み機能" },
          link: "/built-in-features.md",
          items: [{ text: { en: "Cards", ja: "カード" }, link: "/cards.md" }],
        },
      ],
    },
  ],
});
```

解決は決定的です。正確なページロケール、その言語サブタグ、設定した既定ロケール、その言語サブタグ、それから最初の空でないマップ値です。ラベルは HTML エスケープされます。ローカライズしたリンクは存在する兄弟ページを使います。兄弟がなければ、書いた href が有効なままです。折りたたみの sticky 状態はナビ木の位置を使うので、ロケールを変えてもリセットされません。

## クイックスタート

### CSS 変数でカスタム

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent, defineTheme, defaultTheme } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      ssg: {
        siteName: "My Docs",
        theme: defineTheme({
          extends: defaultTheme,
          colors: {
            primary: "#3498db",
          },
          socialLinks: {
            github: "https://github.com/your/repo",
          },
          footer: {
            message: "Released under the MIT License.",
            copyright: "Copyright © 2024 My Company",
          },
        }),
      },
    }),
  ],
});
```

### JSX テーマ（完全制御）

ox-content は JSX / TSX テーマをサポートし、**クライアント側 JavaScript なし** で静的 HTML に描画します（既定）。

```tsx
// theme/Layout.tsx
import { usePageProps, useSiteConfig, useNav, raw, each } from "@ox-content/vite-plugin";

export function Layout({ children }) {
  const page = usePageProps();
  const site = useSiteConfig();
  const nav = useNav();

  return (
    <html lang="en">
      <head>
        <meta charset="UTF-8" />
        <title>
          {page.title} - {site.name}
        </title>
      </head>
      <body>
        <nav>
          {each(nav, (group) => (
            <div>
              <h3>{group.title}</h3>
              <ul>
                {each(group.items, (item) => (
                  <li>
                    <a href={item.href}>{item.title}</a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </nav>
        <main>{children}</main>
      </body>
    </html>
  );
}
```

JSX 向けに `tsconfig.json` を設定します。

```json
{
  "compilerOptions": {
    "jsx": "react-jsx",
    "jsxImportSource": "@ox-content/vite-plugin"
  }
}
```

## CSS 変数リファレンス

テーマの色、レイアウト寸法、フォントスタックはすべて、`:root` 上の `--octc-` プレフィックス付き CSS カスタムプロパティとして出ます。下のテーマ設定からセットしても、[独自 CSS](#独自-css-と-javascript) から直接上書きしても構いません。どちらでも変数が単一の真実です。

### 色

| オプション                 | CSS 変数                     | 説明                                                     |
| -------------------------- | ---------------------------- | -------------------------------------------------------- |
| `colors.primary`           | `--octc-color-primary`       | リンクやアクティブ状態の主アクセント                     |
| `colors.primaryHover`      | `--octc-color-primary-hover` | ホバー時の主色                                           |
| `colors.background`        | `--octc-color-bg`            | メイン背景色                                             |
| `colors.backgroundAlt`     | `--octc-color-bg-alt`        | 代替背景（サイドバー、コードブロック）                   |
| `colors.text`              | `--octc-color-text`          | メイン文字色                                             |
| `colors.textMuted`         | `--octc-color-text-muted`    | 控えめ / 副次の文字色                                    |
| `colors.border`            | `--octc-color-border`        | 境界色                                                   |
| `colors.codeBackground`    | `--octc-color-code-bg`       | コードブロック背景                                       |
| `colors.codeBackgroundTop` | `--octc-color-code-bg-top`   | コードブロック勾配の上。省略時は `codeBackground` に従う |
| `colors.codeText`          | `--octc-color-code-text`     | コードブロック文字色                                     |

### レイアウト

| オプション               | CSS 変数                   | 説明                              |
| ------------------------ | -------------------------- | --------------------------------- |
| `layout.sidebarWidth`    | `--octc-sidebar-width`     | サイドバー幅（既定: `260px`）     |
| `layout.headerHeight`    | `--octc-header-height`     | ヘッダー高さ（既定: `60px`）      |
| `layout.maxContentWidth` | `--octc-max-content-width` | コンテンツ最大幅（既定: `960px`） |

### フォント

| オプション    | CSS 変数             | 説明                                                       |
| ------------- | -------------------- | ---------------------------------------------------------- |
| `fonts.sans`  | `--octc-font-sans`   | サンセリフのフォントスタック、またはセルフホストファミリー |
| `fonts.mono`  | `--octc-font-mono`   | 等幅のフォントスタック、またはセルフホストファミリー       |
| `fonts.named` | `--octc-font-<name>` | カスタム CSS 向けの追加ファミリー                          |

`sans` と `mono` は CSS スタック文字列か Web フォントオブジェクトのどちらかです。文字列形式はこれまでどおりです。

```ts
fonts: {
  sans: "Inter, sans-serif",
  mono: "DM Mono, monospace",
}
```

オブジェクト形式はファミリー名を指定します。`selfHost: true` のとき、Ox Content は要求されたウェイトとサブセットを SSG 出力へコピーし `@font-face` を出すので、公開サイトは実行時に Google Fonts へリクエストしません。

```ts
fonts: {
  sans: {
    family: "Inter",
    provider: "google",
    weights: [400, 600],
    subsets: ["latin"],
    display: "swap",
    selfHost: true,
  },
  mono: "DM Mono, monospace",
  named: {
    code: {
      family: "JetBrains Mono",
      provider: "google",
      weights: [400],
      selfHost: true,
    },
  },
}
```

- `sans` / `mono` はこれまでどおり `--octc-font-sans` と `--octc-font-mono` に対応します。
- `named` のファミリーは `--octc-font-<name>`（例: `--octc-font-code`）として出ます。
- `provider: "local"` はファイルまたは `@fontsource/*` ディレクトリを読み、ネットワーク不要です。CI や、すでにファイルを同梱している場合に使います。
- `preload: true`（またはウェイトの配列）は、そのフェイスに `<link rel="preload">` を出します。
- ダウンロードは `node_modules/.cache/ox-content/fonts` にキャッシュされます。

`selfHost: true` がないオブジェクトは CSS スタックだけを設定し、ファイルの取得や出力はしません。

セットしたキーだけが出ます。省略した色、フォント、レイアウトは [既定テーマの値](#既定テーマの値) に落ちるので、アクセント 1 つを上書きするためにパレット全体を書き直す必要はありません。

## ダークモード

`colors` がライトパレット、`darkColors` がダークパレットです。Ox Content は 1 回のビルドから両方を出し、2 つのセレクタで切り替えます。

- `[data-theme="dark"]` — ページ（または読者が、組み込みヘッダーのテーマ切替で）明示的にダークを選んだとき。切替は `localStorage` に残すので、遷移しても保たれます。
- `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { … } }` — 読者が明示的にライトを選んでいない限り、OS の設定を尊重します。

```ts
defineTheme({
  extends: defaultTheme,
  colors: { primary: "#3b82f6", background: "#ffffff" },
  darkColors: { primary: "#60a5fa", background: "#060816" },
});
```

`darkColors` も `colors` と同じキー単位のフォールバックです。省略したキーは既定のダークパレットを継承します。

## bare / 独自ホストでのテーマトークン

`ssg.bare: true` と独自ホストは自分で document を組み立てるので、Ox Content はテーマのスタイルシートを出しません。`renderThemeTokenCss()` は組み込み SSG が書くはずだった `--octc-*` 宣言をそのまま返します。Vite プラグインも SSG も、ネイティブバインディングもファイルシステム API も引き込まないサブパスから import できます。

```ts
import { renderThemeTokenCss } from "@ox-content/vite-plugin/theme-tokens";
import { kanagawa } from "@ox-content/theme-color-kanagawa";

const css = renderThemeTokenCss(kanagawa);
```

Kanagawa の既定 export は Lotus light + Wave dark のままです。独自ホストで
Lotus light と canonical な Dragon dark syntax palette を使いたい場合は、同じ
パッケージから `kanagawaDragon` を import します。

```ts
import { kanagawaDragon } from "@ox-content/theme-color-kanagawa";

const css = renderThemeTokenCss(kanagawaDragon);
```

組み込みハイライタは `var(--octc-syntax-*)` を参照するので、ページのパレット・タイポグラフィ・レイアウトは自前のまま、配色のコードカラーだけ borrow したいホストはトークン名で絞り込めます。名前は `--octc-` プレフィックスなしで渡ってきます。

```ts
const syntaxOnly = renderThemeTokenCss(kanagawa, {
  include: (name) => name.startsWith("syntax-"),
});
```

出力は上の「ダークモード」で説明した 3 つのセレクタ（`:root`、`[data-theme="dark"]`、明示的なライト選択が勝つ `prefers-color-scheme` フォールバック）を使います。組み込み SSG が呼ぶのと同じレンダラだからです。

レイヤの合成は `resolveTheme()` と同じです。配列を渡せばスキンとカラースキームを重ねられ、各レイヤの `extends` チェーンはベースから順に平坦化されます。

```ts
import { pixel } from "@ox-content/theme-pixel";

const css = renderThemeTokenCss([pixel, kanagawa]);
```

トークン名は小文字のケバブケースです。空や不正な名前は壊れたカスタムプロパティを出す代わりに throw し、値が空のトークンはスキップされます。プラグイン本体を既に import しているなら、この関数は低レベルの `tokensToCss(light, dark)` と並んでパッケージルートからも export されています。

## エントリページのモード

既定テーマはランディングページのモードを 2 つ持っています。

- `default` — よりブランド寄りの、マーケティング風エントリページ
- `subtle` — docs.rs に近い、余白を詰めた控えめなヒーロー

```ts
defineTheme({
  extends: defaultTheme,
  entryPage: {
    mode: "subtle",
  },
});
```

## ページアウトライン

既定テーマは、ページ見出しから右側の「このページ」アウトラインを描けます。**既定はオフ** です。`aside: true` でオンにします。アウトラインが出るのは TOC エントリがあるページだけです。エントリページはアウトラインを出しません。

```ts
defineTheme({
  extends: defaultTheme,
  aside: true,
});
```

オンにすると、マークアップは `<aside class="toc">` と、記事カラムの `main--with-toc` のままです。オプトインになる前と同じ chrome です。アウトラインが欲しい既存サイトは `theme.aside: true` を設定する必要があります。

## 見出しパーマリンク

見出し横の可視 `#` リンクは **既定オフ** です。`headingPermalinks: true` で、
レンダラが生成済み id を使って `<a class="header-anchor" href="#id">` を付けます。
`theme.headingPermalink` は CSS の見え方だけを選びます。

```ts
defineTheme({
  extends: defaultTheme,
  headingPermalink: "always",
});
```

`"hover"`（既定）は hover と `:focus-visible` で出し、タッチでは常時表示です。
`"always"` は常に出します。見出し HTML は変わりません。
[見出しパーマリンク](./built-in/heading-permalinks.md) を見てください。

## ページ props とフック

テーマコンポーネントではフックでページデータに触れます。

### `usePageProps()`

現在ページのデータを返します。

```tsx
function PageHeader() {
  const page = usePageProps();

  return (
    <header>
      <h1>{page.title}</h1>
      {page.description && <p>{page.description}</p>}
    </header>
  );
}
```

**使えるプロパティ:**

- `title` — ページタイトル
- `description` — ページ説明
- `html` — 描画済み HTML
- `toc` — 目次
- `path` — ソースファイルパス
- `url` — 出力 URL
- `frontmatter` — 生の frontmatter オブジェクト
- `layout` — レイアウト名

### `useSiteConfig()`

サイト全体の設定を返します。

```tsx
function SiteHeader() {
  const site = useSiteConfig();

  return <header>{site.name}</header>;
}
```

### `useNav()`

ナビグループを返します。

```tsx
function Sidebar() {
  const nav = useNav();

  return (
    <nav>
      {each(nav, (group) => (
        <section>
          <h3>{group.title}</h3>
          {each(group.items, (item) => (
            <a href={item.href}>{item.title}</a>
          ))}
        </section>
      ))}
    </nav>
  );
}
```

### `useIsActive(path)`

パスが現在ページかどうかを調べます。

```tsx
function NavLink({ href, children }) {
  const isActive = useIsActive(href);

  return (
    <a href={href} class={isActive ? "active" : ""}>
      {children}
    </a>
  );
}
```

## JSX ユーティリティ

### `raw(html)`

エスケープせずに生 HTML を描画します。

```tsx
<div>{raw(page.html)}</div>
```

### `each(items, render)`

配列を写します。

```tsx
{
  each(items, (item, index) => <li key={index}>{item.name}</li>);
}
```

### `when(condition, content)`

条件付き描画です。

```tsx
{
  when(page.toc.length > 0, <aside class="toc">...</aside>);
}
```

## 型生成

ox-content はページの frontmatter から TypeScript 型を自動生成します。生成型は出力ディレクトリに保存されます。

```ts
// Generated: page-props.d.ts
export interface PageFrontmatter {
  title: string;
  description?: string;
  layout?: string;
  // ... other fields from your frontmatter
}

export type PageProps = import("@ox-content/vite-plugin").PageProps<PageFrontmatter>;
```

生成型の使い方:

```tsx
import type { PageProps } from "./page-props";

function Layout() {
  const page = usePageProps<PageProps["frontmatter"]>();
  // page.frontmatter is now fully typed
}
```

## レイアウト切替

frontmatter に応じて複数レイアウトを使えます。

```tsx
// theme/index.tsx
import { createTheme } from "@ox-content/vite-plugin";
import { DefaultLayout } from "./layouts/Default";
import { EntryLayout } from "./layouts/Entry";
import { BlogLayout } from "./layouts/Blog";

export default createTheme({
  layouts: {
    default: DefaultLayout,
    entry: EntryLayout,
    blog: BlogLayout,
  },
});
```

Markdown 側:

```md
---
layout: entry
title: Welcome
---

# Welcome to My Docs
```

## ソーシャルリンク

ヘッダーにソーシャルリンクを足します。短縮形はよく使うネットワークをカバーします。

```ts
defineTheme({
  extends: defaultTheme,
  socialLinks: {
    github: "https://github.com/your/repo",
    twitter: "https://twitter.com/yourhandle",
    discord: "https://discord.gg/yourserver",
  },
});
```

それ以外は `{ icon, link, label? }` の配列を渡します。`icon` は次の形式を受け付けます。

| 形式                  | 例                            | 描画                                     |
| --------------------- | ----------------------------- | ---------------------------------------- |
| Iconify `prefix:name` | `"mdi:mastodon"`              | Iconify アイコン（任意のセット）、色対応 |
| Lucide                | `"lucide:rss"`                | Iconify 経由の Lucide アイコン           |
| 画像 URL              | `"https://example.com/x.svg"` | そのソースの `<img>`                     |
| ローカルパス          | `"/icons/x.svg"`              | サイト `base` に対して解決した `<img>`   |
| 絵文字 / テキスト     | `"📡"`                        | そのままインライン描画                   |

```ts
defineTheme({
  extends: defaultTheme,
  socialLinks: [
    { icon: "mdi:mastodon", link: "https://mastodon.social/@you", label: "Mastodon" },
    { icon: "lucide:rss", link: "/feed.xml", label: "RSS" },
  ],
});
```

アイコンとして渡したインライン SVG はサニタイズされます。`<script>` は除くので、アイコン文字列が実行可能なマークアップを注入することはありません。

[`icons`](./built-in/icons.md) を有効にすると、これらの Iconify 名（とエントリページの feature アイコン）は `api.iconify.design` ではなくローカルな CSS マスクになります。

## 埋め込み HTML（スロット）

`embed` オプションは、ページレイアウトの決まった位置に生 HTML を注入します。9 箇所すべて任意です。

| フィールド      | 描画先                                           |
| --------------- | ------------------------------------------------ |
| `head`          | `<head>` 内（分析、`preconnect`、独自 `<meta>`） |
| `headerBefore`  | ヘッダーバーの直前                               |
| `headerAfter`   | ヘッダーバーの直後                               |
| `sidebarBefore` | サイドバー先頭、ナビの前                         |
| `sidebarAfter`  | サイドバー末尾、ナビのあと                       |
| `contentBefore` | メインコンテンツの前（記事の上）                 |
| `contentAfter`  | メインコンテンツのあと（記事の下）               |
| `footerBefore`  | フッターの直前                                   |
| `footer`        | 既定フッターを丸ごと置き換える                   |

```ts
defineTheme({
  extends: defaultTheme,
  embed: {
    head: '<link rel="preconnect" href="https://fonts.googleapis.com">',
    headerBefore: '<div class="announcement">New version!</div>',
    contentAfter: '<div class="feedback">Was this helpful?</div>',
    footer: '<footer class="custom">© My Project</footer>',
  },
});
```

埋め込み HTML はそのまま挿入されるので、信頼できるマークアップだけを渡してください。

## 独自 CSS と JavaScript

`css` は生成された `--octc-*` 変数上書きの **あと** に付くので、特異度が同じときは自分の規則が勝ち、変数を自由に読んだり再定義したりできます。`js` はすべてのページにインラインスクリプトとして注入されます。

```ts
defineTheme({
  extends: defaultTheme,
  css: `
    /* Override a generated variable for every page… */
    :root {
      --octc-max-content-width: 1100px;
    }
    /* …or target the rendered markup directly. */
    .content h1 {
      color: var(--octc-color-primary);
      letter-spacing: -0.04em;
    }
  `,
  js: `
    console.log('Page loaded');
  `,
});
```

一度きりの調整なら、テーマ全体を定義せず `ssg` プラグインオプションに直接 `css` を渡せます。同じようにマージされます。

```ts
oxContent({
  ssg: {
    theme: { css: ".hero-name { letter-spacing: -0.04em; }" },
  },
});
```

## 既定テーマの値

```ts
const defaultTheme = {
  name: "default",
  aside: false,
  headingPermalink: "hover",
  colors: {
    primary: "#3b82f6",
    primaryHover: "#2563eb",
    background: "#ffffff",
    backgroundAlt: "#f5f7fb",
    text: "#131a30",
    textMuted: "#4f607b",
    border: "#d2dbea",
    codeBackground: "#0b1328",
    codeText: "#eaf2ff",
  },
  darkColors: {
    primary: "#60a5fa",
    primaryHover: "#93c5fd",
    background: "#060816",
    backgroundAlt: "#0d1528",
    text: "#ebf2ff",
    textMuted: "#8ea0bf",
    border: "#223252",
    codeBackground: "#0a1020",
    codeText: "#e7f0ff",
  },
  fonts: {
    sans: '"IBM Plex Sans", "Avenir Next", "Segoe UI Variable", "Segoe UI", sans-serif',
    mono: '"IBM Plex Mono", "SFMono-Regular", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "260px",
    headerHeight: "60px",
    maxContentWidth: "960px",
  },
  socialLinks: {},
};
```

## TypeScript 対応

型はすべて export されます。

```ts
import type {
  ThemeConfig,
  ThemeColors,
  ThemeLayout,
  ThemeFonts,
  ThemeHeader,
  ThemeFooter,
  SocialLinks,
  ThemeEmbed,
  ResolvedThemeConfig,
  PageProps,
  BasePageProps,
  SiteConfig,
  NavGroup,
  NavItem,
  ThemeComponent,
  ThemeProps,
} from "@ox-content/vite-plugin";
```
