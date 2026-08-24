---
title: RSS, Atom, and JSON feeds
description: Opt-in machine-readable feeds written next to generated HTML.
date: 2026-08-24
---

# RSS, Atom, and JSON feeds

When `feeds` is enabled and `ssg.siteUrl` is set, the SSG build writes
machine-readable feeds next to the generated HTML:

- `feed.xml` — RSS 2.0
- `atom.xml` — Atom
- `feed.json` — JSON Feed 1.1

The feature is off unless you turn it on. Existing sites stay unchanged.

Items come from generated pages (the default `content` collection). They are
sorted by a frontmatter date field, newest first. Pages without a parseable
date are listed last. The default field name is `date`.

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

`false` or omitted keeps the files off. `true` enables the defaults. An object
enables the feature and overrides only the fields you set:

```ts
oxContent({
  feeds: {
    rss: true,
    atom: true,
    json: false,
    limit: 10,
    collection: "content",
    dateField: "date",
  },
  ssg: {
    siteUrl: "https://example.com",
  },
});
```

| Option       | Type                       | Default     |
| ------------ | -------------------------- | ----------- |
| `feeds`      | `boolean` / `FeedsOptions` | `false`     |
| `rss`        | `boolean`                  | `true`      |
| `atom`       | `boolean`                  | `true`      |
| `json`       | `boolean`                  | `true`      |
| `limit`      | `number`                   | `20`        |
| `collection` | `string`                   | `"content"` |
| `dateField`  | `string`                   | `"date"`    |

If `feeds` is enabled without `ssg.siteUrl`, no files are written. The build
continues and emits a warning.

Titles and descriptions are escaped so they cannot break out of XML or JSON
Feed strings. Hostile markup such as `<script>` does not appear raw.

This documentation site enables `feeds: true` with `ssg.siteUrl` set.

## Related

- [Collections](./collections.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [Built-in Features overview](../built-in-features.md)
