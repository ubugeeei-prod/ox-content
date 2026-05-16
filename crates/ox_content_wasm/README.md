# @ox-content/wasm

WebAssembly bindings for Ox Content's Rust Markdown engine.

Use this package when you want to run Ox Content in the browser, a Web Worker, or another JavaScript environment that can load `.wasm` from ESM.

If you are building for Node.js, prefer [`@ox-content/napi`](https://www.npmjs.com/package/@ox-content/napi).

## Installation

```bash
pnpm add @ox-content/wasm
```

## Usage

```ts
import init, { parseAndRender, WasmParserOptions } from "@ox-content/wasm";

await init();

const options = new WasmParserOptions();
options.gfm = true;
options.tables = true;
options.taskLists = true;

const result = parseAndRender("# Hello from WASM", options);
console.log(result.html);
```

## API

The current WebAssembly surface exposes:

- `parseAndRender(source, options?)`
- `transform(source, options?)`
- `version()`
- `WasmParserOptions`

## Notes

- The generated module is ESM-only.
- The default `init()` call loads `ox_content_wasm_bg.wasm` relative to the package entry.
- For server-side Node.js usage, use `@ox-content/napi` instead.

## Local Build

From the repository root:

```bash
vp run build:wasm
pnpm --filter @ox-content/wasm pack --dry-run
```

This generates a publish-ready package in `crates/ox_content_wasm/pkg/`.

## First Local Publish

Use `pnpm --filter` from the workspace root so you do not accidentally target the wrong package:

```bash
pnpm whoami || pnpm login
pnpm --filter @ox-content/wasm publish --access public --no-git-checks
```

Avoid `pnpm -r publish` here, because that would try to publish every publishable package in the workspace.
