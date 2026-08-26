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

After `build`, `dist/plain.html` should not contain `ox-code-play.js`; only
routes with `play` fences load the runtime.

The plugin never executes samples during Markdown transform or SSG. Readers
trigger **Run** in the browser. TypeScript **Typecheck** needs the Vite dev
proxy or a reachable `endpoints.typecheck`.
