---
title: Conditional Blocks
description: Opt-in static ::: if / ::: else blocks for environment-specific Markdown.
---

# Conditional Blocks

Conditional blocks are opt-in. When disabled, the `:::` form stays literal.
When enabled, Ox Content evaluates conditions from page frontmatter and
`conditionalBlocks.values` before Markdown is parsed.

| Option              | Type                                  | Default |
| ------------------- | ------------------------------------- | ------- |
| `conditionalBlocks` | `boolean` / `ConditionalBlockOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      conditionalBlocks: {
        values: {
          runtime: "node",
          channels: ["stable", "alpha"],
        },
      },
    }),
  ],
};
```

## Authoring

Use `::: if`, `::: else if`, `::: elif`, and `::: else`. Non-selected branches
are removed before HTML, TOC, module code, and search extraction are built.

```md
::: if runtime == "node"
Node-only setup.
::: else if runtime in ["deno", "bun"]
Alternative runtime setup.
::: else
Browser setup.
:::
```

This documentation build enables `conditionalBlocks` with `runtime: "node"`,
so only the selected branch below is rendered:

::: if runtime == "node"
Node-only setup.
::: else
Browser setup.
:::

## Expressions

Expressions are deliberately small and static. They support:

| Syntax                     | Example                                     |
| -------------------------- | ------------------------------------------- |
| Booleans, numbers, strings | `release == "stable"`                       |
| `null`                     | `frontmatter.variant != null`               |
| Arrays                     | `runtime in ["node", "deno"]`               |
| Equality                   | `audience == "library"` / `tier != "draft"` |
| Boolean operators          | `runtime == "node" and channel == "stable"` |
| Parentheses                | `(runtime == "node") or experimental`       |
| Page frontmatter           | `frontmatter.runtime == "browser"`          |
| Shared build-time config   | `config.runtime == "node"`                  |

Bare identifiers first read page frontmatter, then fall back to
`conditionalBlocks.values`. Use `frontmatter.name` or `config.name` when both
places have the same key. Bare values used as conditions must be booleans; Ox
Content does not add JavaScript-style truthiness.

```md
---
runtime: browser
---

::: if runtime == "browser"
The page frontmatter branch wins.
::: else if config.runtime == "node"
The shared config branch is skipped.
:::
```

No user JavaScript is executed. Values come from already-parsed JSON-like data.
Markers inside fenced, inline, or indented code stay literal. Unclosed
conditional blocks also stay literal and emit a transform warning.

## Search

Search indexes use the same preprocessing options as page transforms, so hidden
branches do not add headings or body text to the static index. Pass
`conditionalBlocks` to a custom search build when you call the exported search
helpers directly.

## Related

- [Custom Containers](./containers.md)
- [Search](./search.md)
- [Built-in Features overview](../built-in-features.md)
