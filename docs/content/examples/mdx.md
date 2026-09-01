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

<tabs>
  <tab title="vp">
    <pre><code>vp run --filter ./examples/mdx dev
vp run --filter ./examples/mdx typecheck
vp run --filter ./examples/mdx build</code></pre>
  </tab>
  <tab title="pnpm">
    <pre><code>pnpm --filter ./examples/mdx dev
pnpm --filter ./examples/mdx typecheck
pnpm --filter ./examples/mdx build</code></pre>
  </tab>
  <tab title="bun">
    <pre><code>bun --filter ./examples/mdx dev
bun --filter ./examples/mdx typecheck
bun --filter ./examples/mdx build</code></pre>
  </tab>
  <tab title="npm">
    <pre><code>npm --workspace ./examples/mdx run dev
npm --workspace ./examples/mdx run typecheck
npm --workspace ./examples/mdx run build</code></pre>
  </tab>
  <tab title="yarn">
    <pre><code>yarn workspace ox-content-mdx-example dev
yarn workspace ox-content-mdx-example typecheck
yarn workspace ox-content-mdx-example build</code></pre>
  </tab>
</tabs>

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
