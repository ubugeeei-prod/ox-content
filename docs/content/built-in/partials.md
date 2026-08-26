---
title: Markdown Partials
description: Opt-in parameterized Markdown partials for reusable snippets.
---

# Markdown Partials

Parameterized partials are opt-in and off by default. When enabled, an HTML
comment directive inlines another Markdown file and substitutes named
`{{ values }}` before the host document is parsed. Existing
`<!-- @include: -->` includes are unchanged.

| Option     | Type                          | Default |
| ---------- | ----------------------------- | ------- |
| `partials` | `boolean` / `PartialsOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      partials: true,
    }),
  ],
};
```

`false` or omitted leaves the directive literal. `true` or an object enables
the transform. Expansion is build-time only.

## Directive

```md
<!-- @partial: ./_partials/install.md package="ox-content" manager="pnpm" -->
```

This site enables `partials`, so the next sentence is a live partial:

<!-- @partial: ./_partials/install.md package="ox-content" manager="pnpm" -->

The path may be quoted. Bare names such as `install.md` resolve under
`_partials` (configurable with `root`). `{{ name }}` substitutions are
HTML-escaped, so `<script>` cannot be injected as raw markup.

## Missing parameters

A missing `{{ name }}` stays literal. It is never replaced with an empty
string. Set `missing: "error"` to report a transform diagnostic while still
leaving the placeholder in the source.

```ts
oxContent({
  partials: {
    root: "_partials",
    missing: "literal",
  },
});
```

| Field     | Type                    | Default       |
| --------- | ----------------------- | ------------- |
| `enabled` | `boolean`               | `true`        |
| `rootDir` | `string`                | project root  |
| `root`    | `string`                | `"_partials"` |
| `missing` | `"literal"` / `"error"` | `"literal"`   |

## Path safety

- Relative `./` and `../` paths resolve from the current file.
- `@/` and a leading `/` resolve from `rootDir`.
- After canonicalize, any path outside `rootDir` is rejected. The directive
  stays in the source and a transform error is reported.
- Cycles and nesting deeper than 16 levels are transform errors. Diagnostics
  include the host file and line.

## What is not expanded

The directive is not expanded inside fenced code, indented code, or inline
code. HTML comments that are not `@partial:` stay as-is, including
`<!-- @include: PATH -->`.

## Related

- [File Includes](./includes.md)
- [Built-in Features overview](../built-in-features.md)
