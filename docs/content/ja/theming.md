---
title: テーマ
description: Theme API でドキュメントサイトの見た目を変える。
---

# テーマ

Ox Content の Theme API で見た目を変えられます。簡単な色は CSS 変数、完全な制御は JSX テーマです。

いちから作らなくても、[テーマプリセット](./theme-presets.md) の公式カタログに 27 スキンと 45 配色があります。`@ox-content/theme-*` と `@ox-content/theme-color-*` を `ssg.theme` で合成します。

## 安定した MPA ナビゲーション

組み込みテーマは、保存されたライト、ダーク、またはシステムの配色を初回描画前に復元します。cross-document View Transitions に対応するブラウザーでは、同一オリジンのページ遷移中も現在の画面を維持します。SPA にはならず、リンクは通常どおり別の HTML 文書へ遷移します。未対応ブラウザーではネイティブの遷移にフォールバックします。

`prefers-reduced-motion: reduce` の場合、トランジションは自動的に無効になります。テーマ単位で無効化する場合は `viewTransitions: false` を指定します。

```ts
defineTheme({
  viewTransitions: false,
});
```

外部リンク、ダウンロード、ページ内リンクの動作は変わりません。

## サイドバーラベルの多言語化

サイドバーのすべての `text` には、文字列またはロケールマップを指定できます。
トップレベルのグループ、リンク付き親項目、ネスト項目で同じ形式を使います。

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

ラベルは、ページの完全なロケール、言語サブタグ、設定された既定ロケール、
既定ロケールの言語サブタグ、最初の空でない値の順で解決され、HTML
エスケープされます。兄弟ページが無いリンクは、壊れた URL にせず元の href を保ちます。
折りたたみ状態の保存キーはラベルではなくツリー位置を使うため、言語を切り替えても維持されます。

## 最短

```ts
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

## JSX テーマ

JSX / TSX テーマは、既定でクライアント JS なしの静的 HTML に描画されます。

```tsx
import { usePageProps, useSiteConfig, useNav, raw, each } from "@ox-content/vite-plugin";

export function Layout({ children }) {
  const page = usePageProps();
  const site = useSiteConfig();
  const nav = useNav();
  return (
    <html lang={page.lang}>
      <body>
        <header>{site.siteName}</header>
        <nav>
          {each(nav, (item) => (
            <a href={item.href}>{item.text}</a>
          ))}
        </nav>
        <main>{raw(children)}</main>
      </body>
    </html>
  );
}
```

トークンは `--octc-*` です。スキンは色を直書きせず、配色パッケージが light / dark を担います。契約の詳細は [テーマプリセット](./theme-presets.md) と [英語の Theming](/theming.md) を見てください。

ヘッダーナビの `text` も `{ en, ja }` のロケールマップにできます。[ヘッダー chrome](./built-in/header-chrome.md) を見てください。
