---
title: File Includes
description: Opt-in Markdown file includes for shared snippets.
---

# File Includes

Markdown file includes are opt-in. When enabled, an HTML comment directive
inlines another Markdown file before the host document is parsed.

| Option     | Type                         | Default |
| ---------- | ---------------------------- | ------- |
| `includes` | `boolean` / `IncludeOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      includes: true,
    }),
  ],
};
```

## Directive

```md
<!-- @include: ./shared/warning.md -->
```

The path may be quoted (`"./shared/warning.md"` or `'./shared/warning.md'`).
Whitespace around the path is trimmed.

This site enables `includes`, so the next paragraph is a live include of
`_fragments/include-warning.md`:

<!-- @include: ./_fragments/include-warning.md -->

## Path resolution

- Relative paths resolve from the current file.
- `@/` and a leading `/` resolve from `rootDir` (the Vite project root when
  `rootDir` is omitted).
- After canonicalize, any path outside `rootDir` is rejected. The directive
  stays in the source and a transform error is reported.
- A missing or unreadable target is also a transform error; the directive is
  left in place.

```ts
oxContent({
  includes: {
    rootDir: process.cwd(),
  },
});
```

## Nested includes

Included files may include other files. Cycles (`A` includes `B` includes `A`)
and nesting deeper than 16 levels are transform errors. Those directives stay
literal.

## What is not expanded

The directive is not expanded inside fenced code, indented code, or inline
code. HTML comments that are not exactly `<!-- @include: PATH -->` stay as-is,
including unclosed comments.

Included Markdown is then parsed as part of the host document, so headings and
lists in the fragment become real headings and lists.

## Related

- [Code Blocks](./code-blocks.md)
- [Built-in Features overview](../built-in-features.md)
