# Code Play example

Minimal Vite + Ox Content site that opts into `@ox-content/code-play` for
JavaScript, TypeScript, Rust, Go, and Python.

From the repository root:

```bash
corepack pnpm --filter ./examples/code-play dev
```

Then open the printed local URL. Click **Run** on a `play` fence. Rust and Go
use the Vite dev proxy for their official playground requests during `dev`.
Python renders as Code Play, but **Run** reports `unsupported` until you point
it at a Piston-compatible executor:

```bash
OX_CODE_PLAY_PYTHON_ENDPOINT=https://your-piston.example/api/v2/piston corepack pnpm --filter ./examples/code-play dev
```

```bash
corepack pnpm --filter ./examples/code-play build
corepack pnpm --filter ./examples/code-play preview
```

After `build`, `dist/plain.html` should not contain `ox-code-play.js`; only
routes with `play` fences load the runtime.

The plugin never executes samples during Markdown transform or SSG. Readers
trigger **Run** in the browser. TypeScript **Typecheck** needs the Vite dev
proxy or a reachable `endpoints.typecheck`. Python source is sent to the
configured `OX_CODE_PLAY_PYTHON_ENDPOINT`; set only an executor you trust.
