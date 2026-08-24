---
title: File Tree
description: Opt-in static directory trees from file-tree fences.
---

# File Tree

`file-tree` fences are opt-in. When enabled, a fenced block with language
`file-tree` becomes a static HTML tree. Names are escaped. The real filesystem
is never read.

| Option     | Type                          | Default |
| ---------- | ----------------------------- | ------- |
| `fileTree` | `boolean` / `FileTreeOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      fileTree: true,
    }),
  ],
};
```

## Authoring

This site enables `fileTree`, so the next block is a live tree:

```file-tree
- src/
  - index.ts **
  - lib/
    - util.ts
- …
```

- Names that end in `/` are directories.
- Other names are files.
- A trailing ` **`, or wrapping `**name**`, highlights that entry.
- `…` or `...` is a placeholder and is escaped like any other name.
- Indentation is two spaces per level. Extra blank lines are ignored.

````md
```file-tree
- src/
  - index.ts **
  - lib/
    - util.ts
- …
```
````

Passing `true` or `{}` enables the defaults. The object form accepts
`enabled: false` when you need to keep the option present while turning the
transform off.

Fences inside other fences, indented code, or inline code are left alone.

## Related

- [File Includes](./includes.md)
- [Custom Containers](./containers.md)
- [Built-in Features overview](../built-in-features.md)
