---
title: Documentation versioning
description: Opt-in version prefixes, frozen snapshots, and a header version dropdown.
---

# Documentation versioning

When `versions` is enabled, the SSG can keep a live docs tree next to frozen
snapshots and render a header version dropdown.

The feature is off unless you turn it on. Existing sites stay unchanged.
Versioning **duplicates content on disk**. Historical snapshot directories
are read during the build and are never rewritten.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      versions: {
        current: "3.0.0-alpha",
        entries: [
          {
            id: "3.0.0-alpha",
            label: "3.0.0-alpha",
            prefix: "",
            banner: "unreleased",
          },
          {
            id: "2.90.0",
            label: "2.90.0",
            prefix: "2.90",
            dir: "versions/2.90",
          },
        ],
      },
    }),
  ],
};
```

`false` or omitted keeps prefixes, banners, and the dropdown off. `true`
enables a single current entry labeled `Latest`. An object enables the
feature and overrides only the fields you set.

| Option     | Type                          | Default                     |
| ---------- | ----------------------------- | --------------------------- |
| `versions` | `boolean` / `VersionsOptions` | `false`                     |
| `current`  | `string`                      | first entry, or `"current"` |
| `switcher` | `boolean`                     | `true`                      |
| `badge`    | `boolean`                     | `true`                      |
| `entries`  | `VersionEntry[]`              | one current `Latest` entry  |

Each entry may set:

| Field    | Purpose                                                                  |
| -------- | ------------------------------------------------------------------------ |
| `id`     | Stable key referenced by `current`                                       |
| `label`  | Dropdown text (HTML-escaped)                                             |
| `prefix` | URL segment such as `2.90` or `next`. Empty string is the site root      |
| `dir`    | Snapshot directory relative to the Vite root. Omit for the live `srcDir` |
| `banner` | `"unreleased"`, `"unmaintained"`, or omitted                             |

Search on a prefixed tree fetches `{prefix}/search-index.json` instead of the
root index. Sitemaps stay scoped to the live tree unless a snapshot pass
writes its own files. `javascript:`, `data:`, `vbscript:`, `//`, and `..`
prefixes or snapshot paths are dropped.

Inside a frozen snapshot, safe internal sidebar and header links stay under
that snapshot's prefix. This includes generated and manual sidebars, nested
items, permalinks, frontmatter aliases, configured redirects, breadcrumb
roots, and previous/next links. Locale resolution runs first, so a link from
`/2.90/ja/` keeps both the `2.90` version and `ja` locale when the translated
sibling exists.

If a sidebar destination does not exist in the snapshot, ox-content links to
that version's root (for example, `/2.90/`) instead of silently returning to
the live docs. External URLs, `mailto:`, hash-only links, unsafe schemes, and
protocol-relative URLs are never version-prefixed. The live tree keeps its
existing unprefixed navigation.

Recreate a snapshot from a git tag with:

```bash
node scripts/snapshot-docs-version.mjs --tag v2.90.0 --prefix 2.90
```

## Related

- [Locale Switcher](./locale-switcher.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [Search](./search.md)
- [Site Generation](./site-generation.md)
