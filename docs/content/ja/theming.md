---
title: テーマ
description: ox-content の Theme API で、ドキュメントサイトの見た目をカスタマイズします。
---

# テーマ

ox-content は、ドキュメントサイトの見た目をカスタマイズできる柔軟な Theme API を提供します。簡単なカスタマイズには CSS 変数を使え、完全な制御にはフル JSX テーマを書けます。

ゼロから作りたくない場合は、[テーマプリセット](/theme-presets.md)
の **公式カタログ** が 27 のスキンと 45 のカラースキームを
`@ox-content/theme-*` と `@ox-content/theme-color-*` パッケージとして出荷しており、
`ssg.theme` で合成できます。互換契約（必須トークン、ライトとダーク、スクリーンショット、
スキンは色をハードコードしないこと）は [パッケージの書き方](/theme-presets.md#authoring-a-package)
を見てください。

## クイックスタート

### CSS 変数のカスタマイズ

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

ox-content は、既定で **クライアント側 JavaScript ゼロ** の静的 HTML に描画する JSX/TSX テーマをサポートします。

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

テーマの色、レイアウト寸法、フォントスタックはすべて、
`:root` 上の `--octc-` 接頭辞付き CSS カスタムプロパティとして出ます。テーマ設定（下）から設定するか、
[カスタム CSS](#custom-css-and-javascript) から直接上書きできます。
どちらでも変数が単一の真実です。

### 色

| オプション                 | CSS 変数                     | 説明                                                       |
| -------------------------- | ---------------------------- | ---------------------------------------------------------- |
| `colors.primary`           | `--octc-color-primary`       | リンクやアクティブ状態のプライマリアクセント色             |
| `colors.primaryHover`      | `--octc-color-primary-hover` | ホバー時のプライマリ色                                     |
| `colors.background`        | `--octc-color-bg`            | メイン背景色                                               |
| `colors.backgroundAlt`     | `--octc-color-bg-alt`        | 代替背景（サイドバー、コードブロック）                     |
| `colors.text`              | `--octc-color-text`          | メインテキスト色                                           |
| `colors.textMuted`         | `--octc-color-text-muted`    | 控えめ / 副次テキスト色                                    |
| `colors.border`            | `--octc-color-border`        | 境界色                                                     |
| `colors.codeBackground`    | `--octc-color-code-bg`       | コードブロック背景                                         |
| `colors.codeBackgroundTop` | `--octc-color-code-bg-top`   | コードブロック勾配の上。省略時は `codeBackground` に従う   |
| `colors.codeText`          | `--octc-color-code-text`     | コードブロックのテキスト色                                 |

### レイアウト

| オプション               | CSS 変数                   | 説明                                 |
| ------------------------ | -------------------------- | ------------------------------------ |
| `layout.sidebarWidth`    | `--octc-sidebar-width`     | サイドバー幅（既定: `260px`）        |
| `layout.headerHeight`    | `--octc-header-height`     | ヘッダー高さ（既定: `60px`）         |
| `layout.maxContentWidth` | `--octc-max-content-width` | コンテンツ最大幅（既定: `960px`）    |

### フォント

| オプション   | CSS 変数           | 説明                  |
| ------------ | ------------------ | --------------------- |
| `fonts.sans` | `--octc-font-sans` | サンセリフフォントスタック |
| `fonts.mono` | `--octc-font-mono` | 等幅フォントスタック  |

設定したキーだけが出ます。省略した色、フォント、レイアウト値は
[既定テーマの値](#default-theme-values) にフォールバックするので、アクセントひとつを上書きしても
パレットの残りを再宣言する必要はありません。

## ダークモード

`colors` はライトパレット、`darkColors` はダークパレットを定義します。Ox
Content はひとつのビルドから両方を出し、2 つのセレクターで切り替えます。

- `[data-theme="dark"]` — ページ（または読者が、組み込みヘッダーのテーマトグル経由で）
  明示的にダークモードを選んだときに使います。トグルは選択を `localStorage` に残すので、
  ナビをまたいでも保たれます。
- `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { … } }`
  — 読者が明示的にライトを選んでいない限り、OS の設定を尊重します。

```ts
defineTheme({
  extends: defaultTheme,
  colors: { primary: "#3b82f6", background: "#ffffff" },
  darkColors: { primary: "#60a5fa", background: "#060816" },
});
```

`darkColors` は `colors` と同じキー単位のフォールバックです。省略したキーは
既定のダークパレットを継承します。

## 入口ページのモード

既定テーマは 2 つのランディングページモードをサポートします。

- `default` - よりブランド寄りで、マーケティング調の入口ページ
- `subtle` - 余白を詰め、ヒーローを抑えた、docs.rs に近い静かな見せ方

```ts
defineTheme({
  extends: defaultTheme,
  entryPage: {
    mode: "subtle",
  },
});
```

## ページアウトライン

既定テーマは、ページ見出しから右手の「このページ」アウトラインを描画できます。
**既定はオフ** です。有効にするには `aside: true` を設定します。アウトラインは
TOC エントリがあるページにだけ出ます。

```ts
defineTheme({
  extends: defaultTheme,
  aside: true,
});
```

有効にすると、マークアップは `<aside class="toc">` と、記事カラム上の `main--with-toc`
のままです。オプトインになる前と同じクロムです。アウトラインが欲しい既存サイトは
`theme.aside: true` を設定する必要があります。

## ページ props とフック

テーマコンポーネントでは、フックでページデータにアクセスします。

### `usePageProps()`

現在のページのデータを返します。

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

- `title` - ページタイトル
- `description` - ページの説明
- `html` - 描画済み HTML コンテンツ
- `toc` - 目次
- `path` - ソースファイルパス
- `url` - 出力 URL
- `frontmatter` - 生の frontmatter オブジェクト
- `layout` - レイアウト名

### `useSiteConfig()`

サイト全体の設定を返します。

```tsx
function SiteHeader() {
  const site = useSiteConfig();

  return <header>{site.name}</header>;
}
```

### `useNav()`

ナビゲーショングループを返します。

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

パスが現在のページかどうかを調べます。

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

エスケープせずに raw HTML を描画します。

```tsx
<div>{raw(page.html)}</div>
```

### `each(items, render)`

配列をマップします。

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

ox-content は、ページの frontmatter に基づいて TypeScript 型を自動生成します。生成された型は出力ディレクトリに保存されます。

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

生成された型を使います。

```tsx
import type { PageProps } from "./page-props";

function Layout() {
  const page = usePageProps<PageProps["frontmatter"]>();
  // page.frontmatter is now fully typed
}
```

## レイアウト切り替え

frontmatter に基づく複数レイアウトをサポートします。

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

Markdown では:

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

それ以外は `{ icon, link, label? }` エントリの配列を渡します。
`icon` 欄はいくつかの形式を受け付けます。

| 形式                  | 例                            | 描画結果                                 |
| --------------------- | ----------------------------- | ---------------------------------------- |
| Iconify `prefix:name` | `"mdi:mastodon"`              | Iconify アイコン（任意のセット）、色対応 |
| Lucide                | `"lucide:rss"`                | Iconify 経由の Lucide アイコン           |
| 画像 URL              | `"https://example.com/x.svg"` | そのソースの `<img>`                     |
| ローカルパス          | `"/icons/x.svg"`              | サイト `base` に対して解決した `<img>`   |
| 絵文字 / テキスト     | `"📡"`                        | インラインでそのまま描画                 |

```ts
defineTheme({
  extends: defaultTheme,
  socialLinks: [
    { icon: "mdi:mastodon", link: "https://mastodon.social/@you", label: "Mastodon" },
    { icon: "lucide:rss", link: "/feed.xml", label: "RSS" },
  ],
});
```

アイコンとして渡したインライン SVG はサニタイズされ、`<script>` は取り除かれます。
そのためアイコン文字列が実行可能なマークアップを注入することはありません。

## 埋め込み HTML（スロット）

`embed` オプションは、ページレイアウトの固定位置に raw HTML を差し込みます。9 つの
位置はすべて任意です。

| 欄              | 描画先                                                     |
| --------------- | ---------------------------------------------------------- |
| `head`          | `<head>` 内（アナリティクス、`preconnect`、カスタム `<meta>`） |
| `headerBefore`  | ヘッダーバーの直前                                         |
| `headerAfter`   | ヘッダーバーの直後                                         |
| `sidebarBefore` | サイドバー先頭、ナビの前                                   |
| `sidebarAfter`  | サイドバー末尾、ナビのあと                                 |
| `contentBefore` | メインコンテンツの前（記事の上）                           |
| `contentAfter`  | メインコンテンツのあと（記事の下）                         |
| `footerBefore`  | フッターの直前                                             |
| `footer`        | 既定フッターを丸ごと置き換える                             |

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

## カスタム CSS と JavaScript

`css` は生成された `--octc-*` 変数上書きの **あと** に追記されるので、
詳細度が同じときはあなたのルールが勝ち、変数を自由に読んだり再定義できます。
`js` はすべてのページにインラインスクリプトとして注入されます。

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

一度きりの調整なら、フルテーマを定義せずに `css` を `ssg` プラグインオプションへ
直接渡せます。同じようにマージされます。

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

型はすべてエクスポートされています。

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
