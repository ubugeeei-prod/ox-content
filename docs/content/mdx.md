---
title: MDX & Components
description: Embed Vue, React, or Svelte components in Markdown using island hydration.
---

# MDX & Components

Ox Content lets you embed framework components inside Markdown and `.mdx` files.
It is worth understanding how this works, because it differs from "classic" MDX:

- **`.md` and `.mdx` are parsed identically** — both go through the same Rust
  Markdown parser (CommonMark + GFM). The parser does **not** parse JSX. The
  `.mdx` extension is simply recognised as a content file.
- **Components are resolved by a framework plugin**, not the parser. The
  React/Vue/Svelte plugins scan the content for PascalCase component tags,
  replace them with **island** placeholders, and hydrate them on the client.

So you get Markdown's speed for prose plus real interactive components where you
need them — without shipping a JavaScript bundle for pages that have none.

## Setup

Add the plugin for your framework alongside its official Vite plugin and point
it at your components:

```ts
// vite.config.ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { oxContentReact } from "@ox-content/vite-plugin-react";

export default defineConfig({
  plugins: [
    react(),
    oxContentReact({
      srcDir: "docs",
      // Auto-discover components by glob…
      components: "./src/components/*.tsx",
      // …or map names explicitly:
      // components: { Counter: "./src/components/Counter.tsx" },
    }),
  ],
});
```

Vue and Svelte work the same way via `@ox-content/vite-plugin-vue`
(`oxContentVue`) and `@ox-content/vite-plugin-svelte` (`oxContentSvelte`). When
`components` is a glob, the component name is the PascalCased file name.

## Authoring components in Markdown

Write components as PascalCase tags in your Markdown — self-closing or with
children:

```md
# My Page

Regular **Markdown** prose.

<Counter initial={5} />

<Callout type="tip">
  This child content is passed to the component.
</Callout>
```

Only tags that start with an uppercase letter are treated as components, so your
ordinary HTML (`<div>`, `<span>`, …) is left untouched. Tags inside fenced code
blocks are **not** transformed, so you can document component usage without it
being executed.

### Props

Props use JSX-like syntax. The following forms are recognised:

| Syntax             | Parsed as           |
| ------------------ | ------------------- |
| `prop="text"`      | string              |
| `prop={42}`        | number / JSON value |
| `prop={true}`      | boolean             |
| `prop={ {"a":1} }` | object (JSON)       |
| `prop`             | boolean `true`      |

Props are serialized to a `data-ox-props` attribute on the island element and
handed to your component at hydration time.

## How islands hydrate

Each component becomes an island wrapper in the generated HTML — a block-level
component renders as a `<div data-ox-island="Name" …>` and an inline one as a
`<span data-ox-island="Name" …>`. The matching framework runtime mounts the real
component into that element on the client.

Hydration timing is controlled by a load strategy (see
[`@ox-content/islands`](./packages/vite-plugin-ox-content.md)):

| Strategy  | Hydrates…                                                   |
| --------- | ----------------------------------------------------------- |
| `eager`   | immediately on load (default)                               |
| `idle`    | during `requestIdleCallback` (≈200 ms fallback)             |
| `visible` | when the element scrolls into view (`IntersectionObserver`) |
| `media`   | when a media query matches (`matchMedia`)                   |

Because the server output is plain HTML, pages render and are readable before
(or entirely without) hydration; the island JavaScript is only loaded for the
components a page actually uses.

## Static JSX in themes

Separately from component islands, Ox Content ships a small **static JSX
runtime** (`jsx`, `jsxs`, `Fragment`, `renderToString`, `raw`, `when`, `each`)
used to author themes and layouts that render to HTML strings with no
client-side JavaScript. Configure it in `tsconfig.json`:

```json
{
  "compilerOptions": {
    "jsx": "react-jsx",
    "jsxImportSource": "@ox-content/vite-plugin"
  }
}
```

See [Theming](./theming.md) for using it to build a custom layout.

## See also

- [React Integration](./packages/vite-plugin-ox-content-react.md)
- [Vue Integration](./packages/vite-plugin-ox-content-vue.md)
- [Svelte Integration](./packages/vite-plugin-ox-content-svelte.md)
