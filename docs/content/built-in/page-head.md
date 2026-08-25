---
title: Page head
description: Build-time Unhead-compatible page-head API shared by themed, bare, custom, and ssg: false hosts.
---

# Page head

Ox Content resolves `<title>`, `meta`, `link`, and JSON-LD at **build time**.
There is no Unhead runtime, no client head manager, and no extra browser JS.

Themed pages, bare pages, `ssg.render` hosts, and `ssg: false` apps share the
same typed resolver. Call it yourself when you own the document:

```ts
import { renderHead } from "@ox-content/vite-plugin";

const { html, diagnostics } = renderHead({
  site: { name: "Docs", url: "https://example.com" },
  title: "Guide",
  description: "How it works",
  canonical: "https://example.com/guide/",
});
```

`html` is already escaped. Inject it into `<head>` — for example with `raw()`
in a custom `ssg.render` layout. `@ox-content/vite-plugin-svelte` re-exports
the same `renderHead`.

| Host                        | What emits head tags                                                                        |
| --------------------------- | ------------------------------------------------------------------------------------------- |
| Default theme               | Built-in resolver. Same OG/Twitter tags as before.                                          |
| `bare: true`                | Same resolver. Social tags only when description, `siteUrl`, site name, or OG image is set. |
| `ssg.render` / `ssg: false` | Nothing is injected. Call `renderHead` and write the tags yourself.                         |

Existing sites keep the same tags unless they opt into
[SEO](./seo.md) (`ssg.siteUrl`, `robots`, locale alternates) or
[JSON-LD](./json-ld.md).

## Descriptors

`renderHead` accepts Unhead-shaped input. Later descriptors with the same
identity win and keep the first position.

| Field    | Identity                                       |
| -------- | ---------------------------------------------- |
| `title`  | one title                                      |
| `metas`  | `key`, else `name`, `property`, or `httpEquiv` |
| `links`  | `key`, else `rel`, or `alternate` + `hreflang` |
| `jsonLd` | `key`                                          |

```ts
renderHead({
  title: "Guide",
  titleTemplate: "%s · %siteName",
  site: { name: "Docs" },
  metas: [{ name: "theme-color", content: "#111" }],
  links: [{ rel: "icon", href: "https://example.com/favicon.ico" }],
  jsonLd: [{ key: "blog", json: JSON.stringify({ "@type": "BlogPosting" }) }],
});
```

Unknown keys such as `twitter.imggg` are a TypeScript error. Put extra tags in
`metas` or `links`.

## Safety

Attribute values are HTML-escaped. JSON-LD script bodies escape `<`, `>`, `&`,
and U+2028 / U+2029 as `\uXXXX`.

`javascript:`, `data:`, `vbscript:`, and protocol-relative `//` URLs are
dropped. Custom hosts validate by default. Built-in themed/bare paths keep
emitting the OG image and canonical strings they already computed.

See [SEO](./seo.md) for `ssg.headValidation`.

## Related

- [SEO](./seo.md)
- [JSON-LD](./json-ld.md)
- [Site Generation](./site-generation.md)
- Tracking: [#819](https://github.com/ubugeeei-prod/ox-content/issues/819)
