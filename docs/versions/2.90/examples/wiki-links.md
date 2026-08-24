---
title: Wiki Links
description: Resolve Obsidian-style wiki links into site links.
---

# Wiki Links

Wiki links are opt-in and expand before Markdown parsing, so the rest of the
pipeline still uses the native Rust parser.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      wikiLinks: {
        baseUrl: "/docs",
      },
    }),
  ],
};
```

```md
See [[getting-started|Getting started]] and [[api/transform#options]].
```

`[[target|label]]` becomes a normal Markdown link. Fragment targets are
slugified and the configured `baseUrl` is applied to site-relative links.
