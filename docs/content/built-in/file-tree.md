---
title: File Tree
description: Opt-in static directory trees from file-tree fences.
---

# File Tree

`file-tree` fences are opt-in. When enabled, a fenced block with language
`file-tree` becomes a static HTML tree. Names are escaped. The real filesystem
is never read. Directories that have children open and close with
`<details>`/`<summary>`. Folder and file icons are on by default and can be
replaced from site config.

| Option                 | Type                              | Default |
| ---------------------- | --------------------------------- | ------- |
| `fileTree`             | `boolean` / `FileTreeOptions`     | `false` |
| `fileTree.defaultOpen` | `boolean`                         | `true`  |
| `fileTree.icons`       | `boolean` / `FileTreeIconOptions` | `true`  |

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

This site enables `fileTree`, so the next block is a live tree. Click a
directory to close or open it.

```file-tree
- src/
  - index.ts **
  - lib/
    - util.ts
- empty/
- …
```

- Names that end in `/` are directories.
- Other names are files.
- A trailing ` **`, or wrapping `**name**`, highlights that entry.
- `…` or `...` is a placeholder and is escaped like any other name.
- Indentation is two spaces per level. Extra blank lines are ignored.
- Directories with children start open. Empty directories are a single row.

````md
```file-tree
- src/
  - index.ts **
  - lib/
    - util.ts
- empty/
- …
```
````

Passing `true` or `{}` enables the defaults. The object form accepts
`enabled: false` when you need to keep the option present while turning the
transform off.

Fences inside other fences, indented code, or inline code are left alone.

## Opening and closing

Directories that have children render as `<details>`. No client JavaScript is
required. Set `defaultOpen: false` to start them closed.

```ts
oxContent({
  fileTree: {
    defaultOpen: false,
  },
});
```

## Icons

Icons are on whenever the transform is on. Pass `icons: false` to keep the
tree without glyphs.

```ts
oxContent({
  fileTree: {
    icons: false,
  },
});
```

Replace the defaults with trusted SVG markup or CSS class tokens from site
config. Fence names are never treated as HTML.

```ts
oxContent({
  fileTree: {
    icons: {
      folder: `<svg viewBox="0 0 16 16"><path fill="currentColor" d="M2 3h5l1 2h6v8H2z"/></svg>`,
      folderOpen: `<svg viewBox="0 0 16 16"><path fill="currentColor" d="M2 4h5l1 2H3l2 7h9L12 6H8z"/></svg>`,
      file: "codicon-file",
      files: {
        ts: `<svg viewBox="0 0 16 16"><path fill="currentColor" d="M3 2h7l3 3v9H3z"/></svg>`,
      },
    },
  },
});
```

`files` is keyed by extension. `.ts` and `ts` both match `index.ts`.

## Related

- [File Includes](./includes.md)
- [Custom Containers](./containers.md)
- [Built-in Features overview](../built-in-features.md)
