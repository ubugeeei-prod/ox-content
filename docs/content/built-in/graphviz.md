---
title: Graphviz DOT Diagrams
description: Render dot and graphviz fences to sanitized static SVG at build time.
---

# Graphviz DOT Diagrams

Graphviz rendering is opt-in:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      graphviz: true,
    }),
  ],
};
```

When enabled, ` ```dot ` and ` ```graphviz ` fences are rendered to inline SVG
during the build. The output is static HTML and SVG, so pages ship no diagram
runtime or client JavaScript.

## Example

````md
```dot
digraph pipeline {
  rankdir=LR
  Markdown -> Parser -> Renderer -> HTML
}
```
````

Graphviz output is wrapped in stable markup:

```html
<figure class="ox-graphviz" role="img" aria-label="Graphviz diagram">
  <svg><!-- sanitized Graphviz output --></svg>
</figure>
```

Generated SVG is constrained before it is embedded: script-like content,
event-handler attributes, and non-fragment links are removed. SVG IDs and
references are prefixed per diagram occurrence so repeated diagrams cannot
collide.

## Renderer Command

By default Ox Content runs `dot -Tsvg` and feeds the DOT source on stdin. You
can point at another compatible command or add fixed arguments:

```ts
oxContent({
  graphviz: {
    command: "dot",
    args: ["-Gbgcolor=transparent"],
  },
});
```

Missing renderers fail the build by default. In CI images where Graphviz is not
available yet, use `missingRenderer: "warn"` to keep the original code block:

```ts
oxContent({
  graphviz: {
    missingRenderer: "warn",
  },
});
```

Invalid DOT sources also fail by default. Use `renderErrors: "warn"` only when
you explicitly want a best-effort docs build that preserves the original fence.

## Options

| Option            | Default   | Description                                      |
| ----------------- | --------- | ------------------------------------------------ |
| `command`         | `"dot"`   | Graphviz-compatible command to execute.          |
| `args`            | `[]`      | Extra arguments passed before `-Tsvg`.           |
| `missingRenderer` | `error`   | `error` or `warn` when the command is missing.   |
| `renderErrors`    | `error`   | `error` or `warn` when Graphviz rejects a graph. |
| `timeout`         | `10000`   | Per-diagram timeout in milliseconds.             |
| `cache`           | `true`    | Cache rendered raw SVGs for the current process. |
| `cacheTTL`        | `3600000` | Cache TTL in milliseconds.                       |

## Related

- [Mermaid Diagrams](./mermaid.md) — another static diagram renderer.
- [Component styles](./component-styles.md) — import `styles/graphviz.css` for custom hosts.
- [Built-in Features overview](../built-in-features.md)
