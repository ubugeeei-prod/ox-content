---
title: Markdown source companions
description: Opt-in original Markdown files written beside generated HTML.
---

# Markdown source companions

When `ssg.markdownSource` is enabled, the SSG build writes the original
Markdown beside each published HTML page. The same URL is served in
`vite dev`. The feature is off unless you turn it on.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        markdownSource: true,
      },
    }),
  ],
};
```

`false` or omitted writes nothing extra. `true` enables the defaults. An
object enables the feature and can turn the alternate link off, or opt in
to the default theme's Copy as Markdown control:

```ts
oxContent({
  ssg: {
    markdownSource: {
      alternate: false,
      copy: true,
    },
  },
});
```

| Option           | Type                                | Default |
| ---------------- | ----------------------------------- | ------- |
| `markdownSource` | `boolean` / `MarkdownSourceOptions` | `false` |
| `alternate`      | `boolean`                           | `true`  |
| `copy`           | `boolean`                           | `false` |

## URL mapping

The companion follows the **published** page URL, not the source file tree.
The HTML output extension does not change the companion, which is always
`.md`.

| Published HTML                            | Companion             |
| ----------------------------------------- | --------------------- |
| `/blog/slug/index.html`                   | `/blog/slug.md`       |
| `/index.html`                             | `/index.md`           |
| `/guide/index.htm` (custom extension)     | `/guide.md`           |
| `/docs/guide/index.html` (`base`)         | `/docs/guide.md`      |
| `/getting-started/index.html` (permalink) | `/getting-started.md` |
| `/ja/guide/index.html` (locale)           | `/ja/guide.md`        |

Path escape (`..`) is rejected. Two pages that resolve to the same companion
keep the first and skip the later page.

## Frontmatter

The companion is a **byte-for-byte copy** of the source file, including YAML
frontmatter. The plugin does not strip or rewrite it. Reconstructing Markdown
from HTML would drop authoring syntax; this path copies the bytes already
read for the page transform and does not re-parse Markdown.

## Drafts and exclusions

Draft (`draft: true`) and unlisted (`unlisted: true`) source is never
written or served, even when unlisted HTML is still produced. When
[`publishState`](./drafts.md) is on, scheduled and expired pages follow that
filter too.

Generated pages without an authoring source (blog indexes, taxonomies,
section indexes, 404) do not get a companion.

## Alternate link and themes

When `alternate` is on (the default), generated HTML includes:

```html
<link rel="alternate" type="text/markdown" href="/guide.md" />
```

Custom renderers and themes read the same URL from `usePageProps()`:

```tsx
const page = usePageProps();
return page.markdownSource ? <a href={page.markdownSource}>Source</a> : null;
```

When `copy` is on, the default theme adds a **Copy as Markdown** button and a
**View Markdown** link near the page title (next to “Edit this page” when that
is also enabled). Copy fetches the companion `.md` URL and writes those
original source bytes — including frontmatter — to the clipboard. The View
Markdown link is a static `<a href="…md">` and works without JavaScript.
`copy` stays off unless you set it, even when companions are enabled.

This project's docs site turns the control on with `markdownSource: { copy: true }`.

## Related

- [SSG output primitives](./ssg-output.md)
- [Site Generation](./site-generation.md)
- [Draft / unlisted / scheduled](./drafts.md)
- [Permalinks and Cascade](./permalinks.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [Built-in Features overview](../built-in-features.md)
