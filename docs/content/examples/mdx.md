---
title: Built-in MDX
description: Default-on MDX for .mdx files, with a runnable Vite SSG example.
---

# Built-in MDX

A copy-paste Vite app lives at
[`examples/mdx`](https://github.com/ubugeeei-prod/ox-content/tree/main/examples/mdx)
in the repository.

This example does **not** set `mdx: true`. `.mdx` files parse JSX, ESM, and
`{expression}` by default. A sibling `.md` file stays CommonMark + GFM.

## Run it

```bash
corepack pnpm --filter ./examples/mdx dev
corepack pnpm --filter ./examples/mdx typecheck
corepack pnpm --filter ./examples/mdx build
```

## What the pages show

| File                      | Shows                                                                                                 |
| ------------------------- | ----------------------------------------------------------------------------------------------------- |
| `src/content/index.mdx`   | Lowercase tags as static HTML, PascalCase `data-ox-island`, ESM omitted, `{expression}` not evaluated |
| `src/content/islands.mdx` | Static HTML vs island placeholders without a framework plugin                                         |
| `src/content/plain.md`    | Sibling `.md` that does **not** parse JSX                                                             |

There is no React / Vue / Svelte / Solid plugin. Named components stay
`data-ox-island` placeholders. `import` / `export` do not appear in the HTML
and are not executed.

See [MDX & Components](../mdx.md) for the language reference.
