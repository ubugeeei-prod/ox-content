---
title: MDX & Components
description: Embed Vue, React, or Svelte components in Markdown using island hydration.
---

# MDX & Components

Ox Content lets you embed framework components inside Markdown and `.mdx` files.
It is worth understanding how this works, because it differs from "classic" MDX:

- **JSX elements, module-level `import` / `export`, and prose
  `{expression}` parse when MDX is enabled.** With `mdx: true` /
  `ParserOptions.mdx`, the Rust parser turns PascalCase and member-name
  tags into `MdxJsxFlowElement` / `MdxJsxTextElement` nodes (self-closing
  or open/close, with literal, boolean, `{expr}`, and spread attributes),
  turns file-level `import` / `export` into `MdxjsEsm` nodes, and turns
  document-level `{foo}` / `Hello {name}` into `MdxFlowExpression` /
  `MdxTextExpression`. Fragments (`<>...</>`), JSX comments, and
  `{expression}` children are stored as AST source. Nothing is evaluated.
  `.md` stays CommonMark + GFM unless that option is on.
- **Components are resolved by a framework plugin**, not the renderer. The
  HTML renderer turns named MDX JSX into island placeholders and
  serializes props (literals as JSON, `{expression}` / spreads as source).
  The React/Vue/Svelte plugins still discover PascalCase tags for hydration
  and will evaluate expressions later.

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

Vue, Svelte, and Solid work the same way via `@ox-content/vite-plugin-vue`
(`oxContentVue`), `@ox-content/vite-plugin-svelte` (`oxContentSvelte`), and
`@ox-content/vite-plugin-solid` (`oxContentSolid`). When `components` is a glob,
the component name is the PascalCased file name.

The Solid integration additionally has to run before `vite-plugin-solid`, which
must be given the Markdown extensions — see
[its reference page](./packages/vite-plugin-ox-content-solid.md#plugin-order-and-extensions).

## Authoring components in Markdown

Write components as PascalCase tags in your Markdown — self-closing or with
children:

```md
# My Page

Regular **Markdown** prose. Hello {name}.

{count + 1}

<Counter initial={5} />

<Callout type="tip">

# Title

Hello **world**.

- nested
  - list

<Badge />

</Callout>

<>
<Icons.Star />
{label}
</>

<Card {...cardProps} />

{/_ Hidden from the rendered page _/}
```

Only tags that start with an uppercase letter are treated as JSX / components,
so ordinary HTML (`<div>`, `<span>`, …) stays raw HTML. Member names
(`Foo.Bar`), fragments (`<>...</>`), spreads (`{...props}`), JSX comments
(`{/* note */}`), `{expression}` children, and document-level
`{expression}` are parsed when MDX is on; expression source is stored and
not run. Tags inside fenced code blocks and inline code are **not**
components or expressions.

Module-level `import` and `export` at the start of a file (and after other
ESM) become `MdxjsEsm` nodes. Multi-line statements are collected with a
naive brace / paren / string / comment scan — not a JavaScript parser —
so regex literals and `${}` inside templates may confuse statement
boundaries. `import` / `export` inside fences or inline code is not ESM.
Hostile strings such as `import x from "<script>"` store source and do not
panic. The HTML renderer currently emits nothing for `MdxjsEsm` or
`{expression}` nodes; framework plugins will resolve imports and evaluate
expressions later.

Document-level `{expression}` uses a naive brace / string / comment scan —
not a JavaScript parser — so regex literals may confuse boundaries.
Unclosed `{` stays ordinary text. Fences and inline code never become
expressions. Hostile source such as `{ "<script>" }` is stored and is not
emitted as HTML.

When MDX is on, Markdown between a component's tags is parsed as Markdown
and rendered as HTML **inside** the island wrapper (`<h1>`, `<strong>`,
lists, fences). Fenced and inline code stay code — a `<Alert />` inside a
fence is not an island. Nested PascalCase tags become nested islands.
Unclosed tags do not swallow the rest of the file. Hostile raw HTML in
children such as `<script>alert(1)</script>` is neutralized (the leading
`<` is escaped) so it cannot execute. Fragments (`<>...</>`) render their
markdown children with no island wrapper.

When MDX is on, a named JSX component becomes an island placeholder in the
HTML (`data-ox-island="Name"`). Its attributes are serialized onto that
island — they are not run:

- quoted strings, boolean attributes, and JSON-literal `{42}` / `{true}` /
  `{"a":1}` values become JSON-safe props
- any other `{expression}` is stored as a **source string**
- `{...spread}` attributes become a **spread-source list**

The payload is JSON that unicode-escapes `<`, `>`, and `&`, then sits in
`data-ox-props` (HTML-escaped) and in a `<script type="application/json">`
that the browser does not execute. Hostile source such as
`{"</script><script>"}` or `{alert(1)}` cannot break out of the payload
and is not evaluated. Pages with no components emit no `<script>` and no
island runtime. Framework plugins still resolve and hydrate components
later.

### Props

Props use JSX-like syntax. The following forms are recognised:

| Syntax             | Serialized as                     |
| ------------------ | --------------------------------- |
| `prop="text"`      | string                            |
| `prop={42}`        | number / JSON value               |
| `prop={true}`      | boolean                           |
| `prop={ {"a":1} }` | object (JSON)                     |
| `prop`             | boolean `true`                    |
| `prop={count + 1}` | expression source (not evaluated) |
| `{...props}`       | spread source (not evaluated)     |

Literal props, expression sources, and spreads are serialized together on
the island element. Hydration still happens later; this slice only stores
the payload.

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

That resolves `@ox-content/vite-plugin/jsx-runtime`, and
`@ox-content/vite-plugin/jsx-dev-runtime` when `jsx` is `react-jsxdev`. Both
render to HTML strings; there is no React and no dev-only behavior to opt into.

See [Theming](./theming.md) for using it to build a custom layout.

## See also

- [React Integration](./packages/vite-plugin-ox-content-react.md)
- [Vue Integration](./packages/vite-plugin-ox-content-vue.md)
- [Svelte Integration](./packages/vite-plugin-ox-content-svelte.md)
- [Solid Integration](./packages/vite-plugin-ox-content-solid.md)
