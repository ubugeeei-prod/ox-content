# Getting Started

Ox Content can be adopted at four different layers.

If you are building a documentation site, start with the Vite plugin.
If you want the Rust core from Node.js, use the N-API package next.
If you need browser-side or sandboxed execution, use the WebAssembly package next.
If you want the parser and renderer directly in Rust, use the Rust crates last.

Contributor setup and source builds live on a separate page: [Development Setup](./development-setup.md).

## Choose Your Entry Point

| You want to...                                            | Start here                                  |
| --------------------------------------------------------- | ------------------------------------------- |
| Build a docs site or content pipeline                     | [Vite Plugin](#1-vite-plugin-first)         |
| Call the parser and renderer from Node.js                 | [N-API](#2-nodejs-api-via-n-api)            |
| Run Ox Content in the browser or another WebAssembly host | [WebAssembly Package](./packages/wasm.md)   |
| Embed Ox Content directly in a Rust project               | [Rust Crates](#4-rust-crates)               |
| Work on Ox Content itself                                 | [Development Setup](./development-setup.md) |

## Requirements

| Path                | Requirement                                                            |
| ------------------- | ---------------------------------------------------------------------- |
| Vite Plugin         | Node.js `24+` and a Vite or Vite+ project                              |
| N-API               | Node.js `24+`                                                          |
| WebAssembly Package | A JS toolchain that can install npm packages and load `.wasm` from ESM |
| Rust Crates         | Rust `1.83+`                                                           |

## 1. Vite Plugin First

This is the default entry point for most users.

The Vite plugin gives you the full Ox Content pipeline: Markdown transforms, static site generation, theming, search, OG images, and generated API docs.
It already brings in the native runtime it needs, so you do not need to install `@ox-content/napi` separately for the Vite-based path.

### Install

```bash
vp install @ox-content/vite-plugin
```

### Minimal Setup

```ts
// vite.config.ts
import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "content",
      outDir: "dist/docs",
      highlight: true,
      ogImage: true,
      docs: {
        enabled: true,
        src: ["./src"],
        out: "content/api",
      },
    }),
  ],
});
```

Create a Markdown entry page:

```md
<!-- content/index.md -->

# Hello Ox Content

This site is generated from Markdown.
```

Then run the docs app:

```bash
vp dev
```

### Framework Integrations

If you want component islands inside Markdown, add one of the first-party integrations:

```bash
# Vue
vp install @ox-content/vite-plugin-vue vue @vitejs/plugin-vue

# React
vp install @ox-content/vite-plugin-react react react-dom @vitejs/plugin-react

# Svelte
vp install @ox-content/vite-plugin-svelte svelte @sveltejs/vite-plugin-svelte
```

Read more:

- [@ox-content/vite-plugin](./packages/vite-plugin-ox-content.md)
- [Vue integration](./packages/vite-plugin-ox-content-vue.md)
- [React integration](./packages/vite-plugin-ox-content-react.md)
- [Svelte integration](./packages/vite-plugin-ox-content-svelte.md)
- [Theming](./theming.md)
- [Source docs example](./examples/gen-source-docs.md)

## 2. Node.js API via N-API

If you want Ox Content as a fast Markdown engine inside a Node.js tool, script, or custom docs workflow, use `@ox-content/napi`.

### Install

```bash
vp install @ox-content/napi
```

### Parse and Render

```ts
import { parseAndRender } from "@ox-content/napi";

const markdown = `
# Welcome

- Fast parser
- Rust core
- HTML output
`;

const result = parseAndRender(markdown, {
  gfm: true,
  tables: true,
  taskLists: true,
});

console.log(result.html);
```

### Parse to AST

```ts
import { parseMarkdown } from "@ox-content/napi";

const ast = parseMarkdown("# Hello\n\nThis is **bold**.", {
  gfm: true,
});

console.log(JSON.stringify(ast, null, 2));
```

Read more:

- [@ox-content/napi](./packages/napi.md)
- [@ox-content/wasm](./packages/wasm.md)
- [API Reference](./api/index.md)

## 3. WebAssembly via @ox-content/wasm

If you need Ox Content in the browser, in a Web Worker, or in another WebAssembly host, use `@ox-content/wasm`.

### Install

```bash
vp install @ox-content/wasm
```

### Use from JavaScript

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

The default `init()` call loads `ox_content_wasm_bg.wasm` relative to the package entry, so this path works well in bundlers and environments that support `.wasm` assets from ESM.

### Build and Publish Locally from This Repo

If you are maintaining Ox Content itself, the repository can generate a publish-ready npm package for you:

```bash
vp run build:wasm
cd crates/ox_content_wasm/pkg
vp exec -- npm pack --dry-run
```

For the first local publish of this scoped package, authenticate with the registry if needed and publish it as public:

```bash
cd crates/ox_content_wasm/pkg
vp exec -- npm whoami || vp exec -- npm login
vp exec -- npm publish --access public
```

Publishing from `crates/ox_content_wasm/pkg` is safer than publishing from the workspace root, because it only targets the generated package.

Avoid publishing from the workspace root here unless you intentionally want a broader workspace release flow.

If your npm account enforces 2FA for publishing, npm will prompt for the one-time code during `npm publish`.

The current WASM surface exposes `parseAndRender`, `transform`, `version`, and `WasmParserOptions` from [`crates/ox_content_wasm/src/lib.rs`](https://github.com/ubugeeei-prod/ox-content/blob/main/crates/ox_content_wasm/src/lib.rs).

## 4. Rust Crates

If you want the lowest-level building blocks directly, use the Rust crates.

### Add Dependencies

```toml
[dependencies]
ox_content_allocator = "2.26.0"
ox_content_ast = "2.26.0"
ox_content_parser = "2.26.0"
ox_content_renderer = "2.26.0"
```

### Parse and Render in Rust

```rust
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

fn main() {
    let allocator = Allocator::new();
    let markdown = "# Hello from Rust\n\n- Fast\n- Reusable\n- Markdown";

    let parser = Parser::with_options(&allocator, markdown, ParserOptions::gfm());
    let document = parser.parse().expect("failed to parse markdown");

    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&document);

    println!("{}", html);
}
```

If you need deeper internals, the crate-level APIs are documented in the Rust workspace and explained further in [Architecture](./architecture.md).

## Need to Build Ox Content Itself?

If you are cloning the repository, working on the docs theme, building the N-API bindings locally, or running the full test suite, use [Development Setup](./development-setup.md) instead of this page.
