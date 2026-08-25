---
title: Generated Section Index Pages
description: Opt-in static index pages for directories that have children but no index.md.
---

# Generated Section Index Pages

When `ssg.sectionIndex` is enabled, the SSG build walks collected pages and
the file tree. For each directory that has child pages and no existing index
route (`index.md`, `index.mdx`, or an already generated directory index), it
writes a themed landing page that lists those children as cards or a list.

The listing honors frontmatter `title` (falling back to the filename). Theme
chrome — navigation, search, sidebar — comes from the same `generateHtmlPage`
path as ordinary pages. Existing `index.md` / `index.mdx` files are never
overwritten.

The feature is off unless you turn it on. Existing sites stay unchanged.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        sectionIndex: true,
      },
    }),
  ],
};
```

`false` or omitted writes nothing. `true` enables card listings. An object
enables the feature and overrides only the fields you set:

```ts
oxContent({
  ssg: {
    sectionIndex: {
      style: "list",
    },
  },
});
```

| Option             | Type                              | Default   |
| ------------------ | --------------------------------- | --------- |
| `ssg.sectionIndex` | `boolean` / `SectionIndexOptions` | `false`   |
| `style`            | `"list"` / `"cards"`              | `"cards"` |

## What gets generated

Given this tree and no `guide/index.md`:

```
content/
  index.md
  guide/
    a.md
    b.md
```

the build writes `dist/guide/index.html`. Child titles come from frontmatter
when present:

```md
---
title: Install
---

# Install
```

If `title` is omitted, the filename is formatted (`getting-started.md` →
"Getting Started"). Nested directories that themselves have pages appear as
a single child that links to that directory's index (authored or generated).

Root `index.md` is left alone. A directory whose only children are drafts or
unlisted pages does not get a generated index.

## Existing indexes are kept

If `guide/index.md` or `guide/index.mdx` already exists, that page is the
section landing page. The generator skips the directory even when the option
is on. The same is true when a permalink already claims the directory URL.

Do not rely on this feature to wrap or replace an authored index. Write the
Markdown you want, or omit `index.md` and let the listing be generated.

## Drafts, unlisted pages, and hostile input

Children with `draft: true` or `unlisted: true` are omitted from the listing,
even when `publishState` is off. Scheduled / expired pages follow
`publishState` when that option is enabled.

Titles, descriptions, and hrefs are escaped. `javascript:`, `data:`,
`vbscript:`, `file:`, and protocol-relative `//` hrefs are dropped from the
markup. Attribute breakouts such as `"/guide" onclick="alert(1)"` are escaped
so they cannot leave the `href` attribute.

Cards use the `.ox-section-index` class. List style uses
`.ox-section-index--list`. Bare mode still runs the same escape and href
rules when the listing HTML is passed through `generateHtmlPage`.

## Related

- [Site Generation](./site-generation.md)
- [Draft / unlisted / scheduled](./drafts.md)
- [Permalinks and Cascade](./permalinks.md)
- [Built-in Features overview](../built-in-features.md)
