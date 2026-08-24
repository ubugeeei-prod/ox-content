---
title: テーマ
description: Theme API でドキュメントサイトの見た目を変える。
---

# テーマ

Ox Content の Theme API で見た目を変えられます。簡単な色は CSS 変数、完全な制御は JSX テーマです。

いちから作らなくても、[テーマプリセット](./theme-presets.md) の公式カタログに 27 スキンと 45 配色があります。`@ox-content/theme-*` と `@ox-content/theme-color-*` を `ssg.theme` で合成します。

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

ヘッダーナビの `text` は `{ en, ja }` のロケールマップにできます。[ヘッダー chrome](./built-in/header-chrome.md) を見てください。
