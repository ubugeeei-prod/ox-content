---
title: Permalinks and Cascade
description: Opt-in frontmatter permalink / slug routing and directory-level default frontmatter.
---

# Permalinks and Cascade

Page URLs usually follow the Markdown file tree. Enable `permalinks` when a
page needs a different public path, and `cascade` when a directory should
share default frontmatter with its children.

Both features are off unless you turn them on. Existing sites stay unchanged.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      permalinks: true,
      cascade: true,
    }),
  ],
};
```

`false` or omitted keeps the feature off. `true` enables the defaults. An
object enables the feature and overrides only the fields you set.

| Option       | Type                            | Default |
| ------------ | ------------------------------- | ------- |
| `permalinks` | `boolean` / `PermalinksOptions` | `false` |
| `cascade`    | `boolean` / `CascadeOptions`    | `false` |

## Permalinks

When `permalinks` is on, frontmatter can replace the file-tree URL:

```md
---
title: Getting Started
permalink: /getting-started
---
```

`slug` replaces only the last path segment:

```md
---
title: Install
slug: install
---
```

`guide/intro.md` with `slug: install` becomes `guide/install`. A `permalink`
wins when both keys are set.

Unsafe values are rejected and the file-tree URL is kept:

- `../` or `.` path segments
- Absolute filesystem paths (`C:\`, drive letters)
- `javascript:`, `data:`, `vbscript:`, `file:`
- Protocol-relative `//` URLs

If two pages resolve to the same URL, the build records an error, keeps the
first page, and skips the later page.

Values written into HTML attributes are escaped.

## Cascade

When `cascade` is on, `_index.md` (also `_index.mdx` / `_index.markdown`)
files supply default frontmatter for pages in that directory and below.
A child keeps any key it already set. `permalink` and `slug` are never
inherited, so a section index cannot force every child onto the same URL.

```md
<!-- guide/_index.md -->

---

sidebar: Guide
---
```

```md
<!-- guide/install.md -->

---

title: Install
---
```

`guide/install.md` inherits `sidebar: Guide` and keeps its own title.

## Related

- [Site Generation](./site-generation.md)
- [Collections](./collections.md)
- [Built-in Features overview](../built-in-features.md)
