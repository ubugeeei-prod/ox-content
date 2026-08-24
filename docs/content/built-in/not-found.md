---
title: Custom 404 Page
description: Opt-in themed 404 page written next to generated HTML.
---

# Custom 404 Page

When `notFound` is enabled, the SSG build writes a themed 404 page from
`404.md` (or a configured `source`) under `srcDir`. The page uses the default
theme, so header navigation and search stay available.

The feature is off unless you turn it on. Existing sites stay unchanged, and
Ox Content does not write a 404 page on its own.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      notFound: true,
    }),
  ],
};
```

`false` or omitted keeps the page off. `true` enables the defaults. An object
enables the feature and overrides only the fields you set:

```ts
oxContent({
  notFound: {
    source: "404.md",
  },
});
```

| Option     | Type                          | Default  |
| ---------- | ----------------------------- | -------- |
| `notFound` | `boolean` / `NotFoundOptions` | `false`  |
| `source`   | `string`                      | `404.md` |

The output path matches the rest of the site. Ox Content uses directory URLs,
so the file is `404/index.html`.

The 404 page is marked `noindex` (draft-like). It is omitted from the search
index and from `sitemap.xml` / `llms.txt` when those run. Frontmatter titles
are HTML-escaped.

If `notFound` is enabled and the Markdown source is missing, no page is
written. The build continues and emits a warning.

## Authoring

```md
---
title: Page not found
description: This URL is not a published page.
noindex: true
---

# Page not found

This URL is not a published page. Use search in the header to find another
topic.
```

The default theme already exposes search in the header. A short note on the
404 page is enough to point readers there.

This documentation site enables `notFound: true` and ships
[`404.md`](../404.md).

## Related

- [Search](./search.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [Site Generation](./site-generation.md)
- [Built-in Features overview](../built-in-features.md)
