---
title: RSS, Atom, and JSON feeds
description: Opt-in collection feeds written next to generated HTML.
---

# RSS, Atom, and JSON feeds

When `feeds` is enabled and `ssg.siteUrl` is set, the SSG build writes
machine-readable feeds from a named collection:

- `feed.xml` — RSS 2.0
- `atom.xml` — Atom 1.0
- `feed.json` — JSON Feed 1.1

The feature is off unless you turn it on. Existing sites stay unchanged.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      feeds: true,
      ssg: {
        siteUrl: "https://example.com",
      },
    }),
  ],
};
```

`false` or omitted keeps the files off. `true` enables the defaults: all three
formats, the `content` collection (or the first configured collection), and a
20-item limit. An object enables the feature and overrides only the fields you
set:

```ts
oxContent({
  feeds: {
    formats: ["rss", "json"],
    collection: "blog",
    limit: 10,
    path: "/feeds",
  },
  ssg: {
    siteUrl: "https://example.com",
  },
});
```

| Option       | Type                            | Default                              |
| ------------ | ------------------------------- | ------------------------------------ |
| `feeds`      | `boolean` / `FeedsOptions`      | `false`                              |
| `formats`    | `("rss" \| "atom" \| "json")[]` | `["rss", "atom", "json"]`            |
| `collection` | `string`                        | `content`, else the first collection |
| `limit`      | `number`                        | `20`                                 |
| `path`       | `string`                        | `/` (site root)                      |

`path` is the site-relative directory for the generated files. `/feeds` writes
`feeds/feed.xml`, `feeds/atom.xml`, and `feeds/feed.json`.

Items are sorted newest first. The sort key is frontmatter `date`, then
`lastUpdated` when `date` is missing. Entries with `draft: true` are omitted.

If `feeds` is enabled without `ssg.siteUrl`, no files are written. The build
continues and emits a warning.

Titles and descriptions are escaped so they cannot break out of XML or JSON.

## Related

- [Collections](./collections.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [Site Generation](./site-generation.md)
- [Built-in Features overview](../built-in-features.md)
