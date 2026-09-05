---
title: 独自ホスト lifecycle
description: 独自 HTML ホスト向けに、Vite loading、development routing、cache invalidation、出力書き出しを Ox Content が持つ。
---

# 独自ホスト lifecycle

サイトが layout と公開ポリシーを持ちつつ、Vite lifecycle は Ox Content に任せたい
場合は `oxContentCustomHost()` を使います。

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContentCustomHost } from "@ox-content/vite-plugin";

export default defineConfig({
  appType: "custom",
  plugins: [
    oxContentCustomHost({
      host: "./src/site-host.ts",
      oxContent: {
        srcDir: "content",
        redirects: { provider: "netlify" },
        ssg: {
          markdownSource: true,
          siteUrl: "https://example.com",
          siteName: "Example",
        },
      },
      themeTokens: {
        theme: colorScheme,
        include: (name) => name.startsWith("syntax-"),
      },
    }),
  ],
});
```

この factory は `oxContent({ ssg: { enabled: false } })` と独自ホスト plugin を
一緒に登録します。すでに `oxContent()` を自分で入れているホスト向けには、
低レベルの `createOxContentCustomHostPlugin()` もあります。

## ホスト module

ホスト module は route を export します。route は通常の `Response`、または
`html`、`text`、`contentType`、metadata、dependencies を持つ plain object を
返せます。

```ts
// src/site-host.ts
export default {
  routes: [
    {
      path: "/",
      inputPath: "content/index.md",
      source: "# Home\n",
      aliases: ["/old-home"],
      dependencies: ["src/data.ts"],
      async render(ctx) {
        const data = await ctx.loadModule("/src/data.ts");
        const assets = ctx.assets.document({
          head: "<title>" + data.title + "</title>",
          sharedStyles: [ctx.assets.themeTokens?.href].filter(Boolean),
          clientEntries: ["src/main.ts"],
          crossorigin: true,
        });

        return {
          html: `<!doctype html><html><head>${assets.headHtml}</head><body>${data.html}</body></html>`,
          title: data.title,
        };
      },
    },
  ],
};
```

development では Ox Content が Vite 経由で host を SSR load し、route を dispatch
します。status と content type を保ち、HTML だけに `transformIndexHtml()` を適用し、
route も custom 404 もなければ fallthrough します。route response は promise として
cache され、宣言した dependency の変更で該当 response だけ invalidation されます。
reload は debounce され、失敗した render は次回 retry され、古い in-flight render が
新しい cache entry を消すこともありません。

production では Vite が client asset と `.vite/manifest.json` を出したあと、
`closeBundle` から一度だけ走ります。host と site module を SSR load するためだけに
一時的な middleware-mode Vite server を開き、raw server ではなく `ctx.loadModule` を
渡し、`finally` で必ず閉じます。

## 協調する出力

ホストが描画した HTML route は、既定 SSG と同じ公開 writer に接続されます。

- `writeResourceFiles()` による resource fingerprint と HTML URL 書き換え。
- `writeSelfHostedAssets()` による font と Iconify CSS。
- route `source` からの `writeMarkdownCompanions()`。
- route `aliases` / `redirect` からの `writeRedirectOutputs()`。
- 選択された route metadata からの `writeFeedFiles()` と `writeSiteMapFiles()`。

route の出力 path が重複すると、どの route 同士が衝突したかを示して build を
失敗させます。公開対象の選択はホストが持ち、Ox Content はホストが返した route
だけを書きます。

## テーマトークン stylesheet

`themeTokens` は小さな stylesheet を書き出して dev でも配信します。既定 href は
`/__ox_theme_tokens__/theme-tokens.css` です。`ctx.assets.themeTokens.href` を
`ctx.assets.document()` に渡せば、local Vite transform なしで syntax token CSS を
使えます。

## 関連

- [Document assets](./document-assets.md)
- [SSG 出力プリミティブ](./ssg-output.md)
- [テーマ](../theming.md)
- 追跡: [#1281](https://github.com/ubugeeei-prod/ox-content/issues/1281)
