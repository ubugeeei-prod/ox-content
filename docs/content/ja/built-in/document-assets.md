---
title: Document assets
description: 独自 HTML document 向けに、script、style、link、manifest、self-hosted asset descriptor を描画する。
---

# Document assets

`renderDocumentAssets()` は typed descriptor を独自 document shell 向けの安全な
`<head>` tag に変換します。ホストは build 後の `index.html` を parse したり、
script / style / link serializer を持ったりする必要がありません。

```ts
import { renderDocumentAssets, renderHead } from "@ox-content/vite-plugin";

const head = renderHead({
  title: "Guide",
  description: "Custom HTML host",
});

const assets = renderDocumentAssets({
  base: "/docs/",
  head,
  manifest: viteManifest,
  selfHostedAssets: oxAssetManifest,
  sharedStyles: ["/src/shared.css"],
  pageStyles: ["src/pages/guide.css"],
  islandStyles: solidIslandStylesheets,
  inlineStyles: [{ content: criticalCss, nonce: cspNonce }],
  clientEntries: ["src/main.ts"],
  scripts: [{ content: bootstrapScript, nonce: cspNonce }],
  crossorigin: true,
});

return `<!doctype html><html><head>${assets.headHtml}</head><body>...</body></html>`;
```

戻り値は structured descriptor と描画済み string の両方を持ちます。
`links`、`styles`、`scripts`、`tags`、`headHtml` です。

## 順序と重複排除

出力順は安定しています。metadata head markup、generic link、self-hosted preload、
self-hosted stylesheet、shared style、page style、island style、inline style、
manifest 由来の client CSS、client script、extra script の順です。重複した style、
script、key 付き link は最初の descriptor が残ります。

`manifest` を渡すと、`clientEntries` は Vite build manifest から解決されます。
imported chunk は entry より先に辿られ、transitive CSS は module script より前に
出ます。query string、fragment、`base`、`type="module"`、`crossorigin` も保ちます。
dev では `manifest` を省略すると、同じ entry が Vite の source module URL として
描画されます。

## escape と CSP

属性値は quote、ampersand、tag delimiter を escape します。inline style は mixed-case
の `</style` 終端を、CSS text を落とさず escape します。inline script の
`</script` も別に escape します。

`nonce: "..."` を渡すと inline style と script の両方に nonce を付けます。
`nonce: { style, script }` なら policy を分けられます。各 descriptor が個別に
`nonce` を指定した場合はそれが優先されます。

## 関連

- [ページ head](./page-head.md)
- [独自ホスト lifecycle](./custom-host.md)
- [SSG 出力プリミティブ](./ssg-output.md)
- 追跡: [#1284](https://github.com/ubugeeei-prod/ox-content/issues/1284)
