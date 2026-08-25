---
title: Sitemap, robots.txt, and llms.txt
description: Opt-in crawl manifests written next to generated HTML.
---

# Sitemap, robots.txt, and llms.txt

When `siteMaps` is enabled and `ssg.siteUrl` is set, the SSG build writes
crawl manifests next to the generated HTML:

- `sitemap.xml` — every published page URL, sorted, with `<lastmod>` when Git
  history is available
- `robots.txt` — allow-all plus a Sitemap line
- `llms.txt` — site title, description, and a page list

The feature is off unless you turn it on. Existing sites stay unchanged.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      siteMaps: true,
      ssg: {
        siteUrl: "https://example.com",
      },
    }),
  ],
};
```

`false` or omitted keeps the files off. `true` enables the defaults. An object
enables the feature and overrides only the fields you set:

```ts
oxContent({
  siteMaps: {
    robots: false,
    llms: false,
  },
  ssg: {
    siteUrl: "https://example.com",
  },
});
```

| Option     | Type                          | Default |
| ---------- | ----------------------------- | ------- |
| `siteMaps` | `boolean` / `SiteMapsOptions` | `false` |
| `robots`   | `boolean`                     | `true`  |
| `llms`     | `boolean`                     | `true`  |

`sitemap.xml` is always written when the feature is on. Pages with
`draft: true` in frontmatter are omitted. When [`publishState`](./drafts.md)
is also enabled, unlisted and not-yet-scheduled pages are omitted too.

When Git history is available, each URL includes a W3C `<lastmod>` date. That
value is the source file's latest Git commit time in UTC (`YYYY-MM-DD`), not
the generated HTML mtime. Enabling `siteMaps` reuses the same Git lookup as
`ssg.lastUpdated` and does not require the visible last-updated chrome. A
shallow clone or missing history omits `<lastmod>` for that page.

If `siteMaps` is enabled without `ssg.siteUrl`, no files are written. The build
continues and emits a warning.

Titles and descriptions are escaped so they cannot break out of XML or
`llms.txt`.

## Related

- [Site Generation](./site-generation.md)
- [Built-in Features overview](../built-in-features.md)
