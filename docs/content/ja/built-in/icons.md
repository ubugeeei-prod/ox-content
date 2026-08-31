---
title: セルフホスト Iconify CSS
description: SSG が api.iconify.design へ実行時リクエストしない、オプトインのビルド時 Iconify CSS。
---

# セルフホスト Iconify CSS

Ox Content は Iconify 名をビルド時に解決し、実際に使ったアイコンだけを CSS
マスクとして出せます。エントリページの feature アイコンとカスタムソーシャル
リンクも同じリゾルバを使います。公開サイトは
`https://api.iconify.design` へリクエストしません。

既定はオフです。`@iconify/json` か個別の `@iconify-json/*` を入れて、
コレクションをディスクから読んでください。テストと CI はそれらのパッケージか
ローカル fixture を使い、Iconify API には触れません。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      icons: {
        mode: "css-mask",
        syntax: "unocss",
        include: ["src/**/*.{md,mdx,svelte,ts,json}"],
        safelist: ["carbon:checkbox", "ri:markdown-line"],
      },
    }),
  ],
};
```

ソースを走査したくないときは、`include` に名前を直接書けます。

```ts
oxContent({
  icons: {
    include: ["ri:markdown-line", "line-md:rss", "ph:github-logo-duotone"],
  },
});
```

| オプション | 既定         | 役割                                                      |
| ---------- | ------------ | --------------------------------------------------------- |
| `mode`     | `"css-mask"` | モノクロアイコンを `currentColor` の CSS マスクにします。 |
| `syntax`   | `"unocss"`   | `icon-[prefix--name]` クラスを出します。                  |
| `include`  | `[]`         | 走査する glob、または明示的な `prefix:name`。             |
| `safelist` | `[]`         | ソースに無くても必ず出す名前（動的クラス含む）。          |

ビルドは次の Iconify 名も集めます。

- エントリページ frontmatter の `features[].icon`
- `icon` が `prefix:name` の theme `socialLinks` 配列

コレクションやアイコン名が無いときはビルドエラーになります。インストール済み
コレクションにあっても未使用のアイコンは CSS に出しません。

生成 CSS は `__ox_icons__/icons.css` に書き、テーマの `<head>` からリンクします。
[セルフホストフォント](../theming.md#フォント) と同じアセット経路です。既存の
`icon-[prefix--name]` マークアップは、テンプレートを一気に書き換えなくても
動き続けます。
独自 Vite ホストは `virtual:ox-content/assets.css` を import するか、
`virtual:ox-content/asset-manifest` を読めます。同じ stylesheet が dev で配信され、
本番 build で書き込まれます。

`false` または省略では、エントリページの Iconify アイコンはこれまでどおり
CDN フォールバックです。
