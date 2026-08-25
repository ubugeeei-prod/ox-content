---
title: MDX & Components
description: Embed Vue, React, or Svelte components in Markdown using island hydration.
---

# MDX & Components

Ox Content lets you embed framework components inside Markdown and `.mdx` files.
It is worth understanding how this works, because it differs from "classic" MDX:

- **JSX elements, module-level `import` / `export`, and prose
  `{expression}` parse when MDX is enabled.** That is the default for
  `.mdx` files. With `mdx: true` / `ParserOptions.mdx`, you can enable
  the same path for every configured extension. The Rust parser turns
  PascalCase and member-name tags into `MdxJsxFlowElement` /
  `MdxJsxTextElement` nodes (self-closing or open/close, with literal,
  boolean, `{expr}`, and spread attributes), turns file-level `import` /
  `export` into `MdxjsEsm` nodes, and turns document-level `{foo}` /
  `Hello {name}` into `MdxFlowExpression` / `MdxTextExpression`.
  Fragments (`<>...</>`), JSX comments, and `{expression}` children are
  stored as AST source. Nothing is evaluated. `.md` stays CommonMark +
  GFM unless that option is on.
- **Components are resolved by a framework plugin**, not the renderer. The
  HTML renderer turns named MDX JSX into island placeholders and
  serializes props (literals as JSON, `{expression}` / spreads as source).
  For `.mdx` files (or when `mdx: true`), the React/Vue/Svelte/Solid
  plugins walk the MDX AST — or the rendered `data-ox-island` names —
  and import hydration modules for names in the global `components` map
  **or** a relative `import` resolved from that document. Nested JSX,
  expression attributes, and fragments are visible to that walk.
  Unregistered JSX without a matching import stays static HTML. Plain
  `.md` still uses a source scan of the global map so existing pages keep
  working. Expressions are stored and evaluated later.

So you get Markdown's speed for prose plus real interactive components where you
need them — without shipping a JavaScript bundle for pages that have none.

## Defaults

When `mdx` is omitted, Ox Content infers it from the source extension:

| Source              | Default                           | `mdx: true` | `mdx: false`     |
| ------------------- | --------------------------------- | ----------- | ---------------- |
| `.mdx`              | MDX on (JSX, ESM, `{expression}`) | MDX on      | CommonMark + GFM |
| `.md` / `.markdown` | CommonMark + GFM                  | MDX on      | CommonMark + GFM |

You do **not** need `mdx: true` for `.mdx` files. Set `mdx: true` /
`ParserOptions.mdx` only when you want the same syntax in `.md` files.
Set `mdx: false` to keep `.mdx` on the plain Markdown path.

## Static HTML vs islands

Without a framework plugin, the HTML renderer stays on the static path:

- **Lowercase / custom HTML tags** (`<div>`, `<note>`) stay HTML. They are
  not islands.
- **PascalCase / member-name tags** (`<NoteCard />`, `<Icons.Star />`)
  become `data-ox-island` placeholders. Props are serialized; nothing is
  hydrated until a React / Vue / Svelte / Solid plugin is present.
- **Module-level `import` / `export`** become `MdxjsEsm` nodes. They do
  **not** appear in the HTML and are **not** executed.
- **Prose `{expression}`** is stored as AST source and is **not**
  evaluated. The static HTML renderer currently emits nothing for those
  nodes — the source is not leaked as text or as JavaScript.

A runnable Vite + `@ox-content/vite-plugin` site with real `.mdx` pages is
[`examples/mdx`](https://github.com/ubugeeei-prod/ox-content/tree/main/examples/mdx).

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

## Document-local imports

On `.mdx` (or when `mdx: true`), a document can import a component relative to
itself instead of registering it in the site-wide `components` map:

```md
import GtvChart from './gtv-chart/GtvChart.tsx'

<GtvChart title="ok" />
```

The specifier is resolved from that file's directory. The binding is local to
the document: two pages may both import `Chart` from different files without
sharing one global name. Only the components that page actually uses are
imported into the generated module, as static `import`s, so changing the
component file invalidates the Markdown module through Vite HMR.

| Form                                                | Resolved as an island?             |
| --------------------------------------------------- | ---------------------------------- |
| `import Name from './file.tsx'`                     | Yes, if `<Name />` is used         |
| `import { Chart as Plot } from './file.tsx'`        | Yes, if `<Plot />` is used         |
| Bare / npm / `https:` specifier                     | No. Reported, not resolved         |
| `../` that leaves `srcDir`                          | No. Diagnostic, no import          |
| Same name in the document import and the global map | Document import wins for that file |
| `.md` without `mdx: true`                           | No. ESM is not a document import   |

The global `components` map remains the backwards-compatible fallback for pages
that do not declare a local import. Framework plugins may also pass an optional
`renderIsland(name, props, filePath)` hook to replace island inner HTML at
transform time. That hook lives on the adapter; the core renderer does not
import `react-dom/server`, `svelte/server`, or `solid-js/web`.

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

When MDX is active, the generated Vite module also exports structured
metadata collected from those AST nodes. User JavaScript is **not**
executed during transform, and `import` statements are not re-emitted as
live ESM — they are JSON data:

```ts
import { html, frontmatter, toc, imports, exports, components } from "./guide.mdx";

html;
// string — rendered HTML (islands, no live imports)

frontmatter;
// object — parsed YAML

toc;
// array — heading tree

imports;
// [
//   {
//     source: "./Alert",
//     specifiers: [{ imported: "default", local: "Alert", kind: "default" }],
//   },
//   {
//     source: "./Chart",
//     specifiers: [{ imported: "Chart", local: "Plot", kind: "named" }],
//   },
//   {
//     source: "./icons",
//     specifiers: [{ imported: "*", local: "Icons", kind: "namespace" }],
//   },
// ]

exports;
// ["title", "helper"]

components;
// ["Alert", "Badge", "Icons.Star"]
```

Each `imports` entry is one statement. Specifier `kind` is `default`,
`named`, or `namespace`. `exports` is the list of exported names
(`default` for `export default`). `components` is the unique PascalCase
and member JSX names in document order; fragments (`<>...</>`) are
skipped. When MDX is off, or a file has no MDX nodes, these three
exports are empty arrays so the module shape stays stable.

```md
import Alert from './Alert'
import { Chart as Plot } from './Chart'
import * as Icons from './icons'

export const title = 'Guide'
export function helper() {}

<Alert />

Hello <Badge /> and <Icons.Star />
```

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
island runtime. Framework plugins resolve registered names and document-local imports from
the MDX AST and hydrate those islands later. Unregistered names keep the
static HTML the renderer already emitted.

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
components a page actually uses. On `.mdx`, that list comes from the AST
intersected with the global component map and any resolved document-local
imports, so a nested or fragmented tag still hydrates when it is
registered or imported by that page.

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

- [Built-in MDX example](./examples/mdx.md)
- [React Integration](./packages/vite-plugin-ox-content-react.md)
- [Vue Integration](./packages/vite-plugin-ox-content-vue.md)
- [Svelte Integration](./packages/vite-plugin-ox-content-svelte.md)
- [Solid Integration](./packages/vite-plugin-ox-content-solid.md)
