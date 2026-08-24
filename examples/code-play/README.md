# Code Play example

Minimal Vite + Ox Content site that opts into `@ox-content/code-play` for
JavaScript and TypeScript only.

From the repository root:

```bash
corepack pnpm --filter ./examples/code-play dev
```

Then open the printed local URL. Click **Run** on a `play` fence.

```bash
corepack pnpm --filter ./examples/code-play build
corepack pnpm --filter ./examples/code-play preview
```

The plugin never executes samples during Markdown transform or SSG. Readers
trigger execute / type-check in the browser (or Node, for the headless API).
