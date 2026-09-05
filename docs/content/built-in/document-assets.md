---
title: Document assets
description: Render typed script, style, link, manifest, and self-hosted asset descriptors for custom HTML documents.
---

# Document assets

`renderDocumentAssets()` turns typed descriptors into safe `<head>` tags for a
custom document shell. Hosts do not have to parse a built `index.html` or carry
their own script/style/link serializer.

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

The result contains both structured descriptors and the rendered string:
`links`, `styles`, `scripts`, `tags`, and `headHtml`.

## Ordering and dedupe

Tags are emitted in a stable order: metadata head markup, generic links,
self-hosted preloads, self-hosted stylesheets, shared styles, page styles,
island styles, inline styles, manifest-derived client CSS, client scripts, then
extra scripts. Duplicate styles, scripts, and keyed links keep the first
descriptor.

When `manifest` is supplied, `clientEntries` are resolved through Vite's build
manifest. Imported chunks are walked before the entry, transitive CSS is emitted
before the module script, and query strings, fragments, `base`, `type="module"`,
and `crossorigin` are preserved. In dev, omit `manifest` and the same entry
renders as a Vite-served source module URL.

## Escaping and CSP

Attribute values escape quotes, ampersands, and tag delimiters. Inline styles
escape mixed-case `</style` end tags without dropping the CSS text. Inline
scripts escape `</script` separately.

Pass `nonce: "..."` to apply one nonce to inline styles and scripts, or
`nonce: { style, script }` to split the policy. Descriptors can still set their
own `nonce`.

## Related

- [Page head](./page-head.md)
- [Custom host lifecycle](./custom-host.md)
- [SSG output primitives](./ssg-output.md)
- Tracking: [#1284](https://github.com/ubugeeei-prod/ox-content/issues/1284)
