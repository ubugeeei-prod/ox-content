# @ox-content/wasm

WebAssembly bindings for Ox Content's Rust Markdown engine.

Use this package when you want to run Ox Content in the browser, a Web Worker, or another JavaScript environment that can load `.wasm` from ESM.

If you are building for Node.js, prefer [`@ox-content/napi`](./napi.md).

## Installation

```bash
vp install @ox-content/wasm
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

- The package is ESM-only.
- The default `init()` call loads `ox_content_wasm_bg.wasm` relative to the package entry.
- This makes it a good fit for bundlers and browser-oriented runtimes that support `.wasm` assets.

## Local Build

From the repository root:

```bash
vp run build:wasm
cd crates/ox_content_wasm/pkg
vp exec -- npm pack --dry-run
```

The generated publish-ready package lives in `crates/ox_content_wasm/pkg/`.

## First Local Publish

If you are publishing this package from a local machine for the first time:

```bash
cd crates/ox_content_wasm/pkg
vp exec -- npm whoami || vp exec -- npm login
vp exec -- npm publish --access public
```

The package is scoped, so the initial publish should use `--access public`.

Publishing from `crates/ox_content_wasm/pkg` is the safest option in this monorepo because it targets only the generated package.

After the package exists on npm, the repository's tag-based GitHub Actions workflow can publish later versions with provenance enabled.
