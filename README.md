<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./assets/oxcontent-light.svg">
    <source media="(prefers-color-scheme: light)" srcset="./assets/oxcontent-dark.svg">
    <img alt="Ox Content logo" src="./assets/oxcontent-dark.svg" height="60">
  </picture>
</p>

<p align="center">
  <strong>High-performance Markdown toolkit</strong><br>
  Rust-powered Markdown engine, documentation generator, and content tooling for the JavaScript ecosystem
</p>

<p align="center">
  <a href="https://ubugeeei-prod.github.io/ox-content/">Documentation</a> •
  <a href="https://ubugeeei-prod.github.io/ox-content/getting-started">Getting Started</a> •
  <a href="https://ubugeeei-prod.github.io/ox-content/playground/">Playground</a> •
  <a href="./SECURITY.md">Security</a>
</p>

> [!NOTE]
> Ox Content is an independent personal project by [ubugeeei](https://github.com/ubugeeei). It is not an official VoidZero project, product, or endorsement.
> The current branding is an intentional homage to the VoidZero ecosystem because I care a lot about that design direction and hope I can contribute more directly in the future.
> If VoidZero or the relevant rights holders would prefer that I stop using this branding direction, I will change it.

---

## Features

- **Blazing Fast** - Arena-allocated parser with zero-copy parsing
- **mdast Compatible** - Run custom mdast plugins and existing remark/unified transforms
- **MDX-ready Files** - Process `.mdx` alongside Markdown in Vite, SSG, and framework integrations
- **GFM Support** - Tables, task lists, strikethrough, autolinks, footnotes
- **Multi-Runtime** - Node.js (NAPI), WebAssembly, Native Rust
- **Framework Agnostic** - Works with Vue, React, Svelte, and more
- **Built-in SSG** - Static site generation with theming, search, and OG images
- **Built-in Embeds** - Static GitHub repository, source code, and Open Graph link cards
- **API Docs Generation** - Generate docs from JSDoc/TypeScript (like `cargo doc`)
- **i18n** - ICU MessageFormat 2 parser, dictionary management, static checker, and LSP
- **Editor Tooling** - Markdown/MDC LSP plus VS Code, Zed, and Neovim integrations

## Quick Start

### Basic Usage (Node.js)

```bash
npm install @ox-content/napi
```

```javascript
import { parseAndRender } from "@ox-content/napi";

const { html } = parseAndRender("# Hello World", { gfm: true });
```

### Vite Plugin

```bash
npm install @ox-content/vite-plugin
```

`@ox-content/vite-plugin` already installs the native `@ox-content/napi` dependency it needs.

```typescript
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      outDir: "dist/docs",
      highlight: true,
      ssg: {
        siteName: "My Docs",
      },
    }),
  ],
});
```

### Migrate from VitePress

```bash
ox-content-migrate-vitepress .vitepress/config.ts \
  --src-dir docs \
  --out-dir dist \
  --out ox-content.config.ts
```

The same migration runner is available across JavaScript runtimes:

```bash
# Node.js, after installing @ox-content/vite-plugin
ox-content-migrate-vitepress .vitepress/config.ts --out ox-content.config.ts

# Deno
deno run -A npm:@ox-content/vite-plugin/vitepress-migrate .vitepress/config.ts \
  --out ox-content.config.ts

# Bun
bunx --bun @ox-content/vite-plugin .vitepress/config.ts --out ox-content.config.ts
```

The generated `ox-content.config.ts` contains an editable `OxContentOptions` object built from
VitePress settings such as `title`, `base`, `themeConfig.sidebar`, `themeConfig.socialLinks`,
`themeConfig.footer`, and search placeholder.
`layout: home` frontmatter is also accepted for landing pages during SSG/dev rendering.

### Browser Usage (WebAssembly)

```bash
npm install @ox-content/wasm
```

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

### Framework Integration

```bash
# Vue
npm install @ox-content/vite-plugin-vue

# React
npm install @ox-content/vite-plugin-react

# Svelte
npm install @ox-content/vite-plugin-svelte
```

### i18n Static Checker (CLI)

```bash
# Check for missing/unused translation keys
ox-content-i18n check --dict-dir content/i18n --src src

# Validate an ICU MessageFormat 2 message
ox-content-i18n validate "Hello {$name}"
```

### Dead Link Checker (CLI)

```bash
# Check every link in a tree, exit non-zero on broken targets
ox-content-link-check docs/**/*.md

# Treat `/foo.md` as workspace-rooted under docs/
ox-content-link-check --src-dir docs docs/**/*.md

# Suppress known intentionally-broken targets
ox-content-link-check --ignore "intentionally-broken" docs/**/*.md
```

Offline-only by design — `http://` and `https://` links pass through
without a network request, so the same binary is safe to run in CI
without timeouts, retries, or rate limits.

### Editor Tooling

Ox Content now ships a unified authoring and i18n language server:

```bash
cargo run -p ox_content_lsp --bin ox-content-lsp
```

You can wire it into:

- VS Code via [npm/vscode-ox-content](./npm/vscode-ox-content)
- Zed via [editors/zed](./editors/zed)
- Neovim via [editors/neovim](./editors/neovim)

Supported features include:

- fast Markdown snippet completion
- frontmatter schema completion and diagnostics, including built-in `meta` fields
- i18n key completion, hover, go-to-definition, diagnostics, inlay hints, and dictionary links for JS/TS
- table / code fence / callout insertion commands
- preview HTML generation for editor UIs (with LSP-pushed HMR)
- `.mdc` authoring support with component tag diagnostics
- asset path completion inside `[…](`, `![…](`, and HTML `src=`/`href=` attributes
- dead link diagnostics powered by `ox_content_link_checker`
- opt-in [textlint](https://textlint.github.io) integration: on-save diagnostics and quick fixes under `source: "textlint"`
- half-width/full-width spacing diagnostics, quick fixes, and opt-in save-time fixes
- MDC component name and attribute completion when a project provides a component registry

For CI or editor-independent checks, run:

```bash
cargo run -p ox_content_mdc_checker --bin ox-content-mdc-check -- docs/page.mdc
```

**[Read the full documentation →](https://ubugeeei-prod.github.io/ox-content/)**

## Performance

Ox Content is positioned both as a document generator and as a high-performance Markdown toolkit. The numbers below focus on the Markdown engine side.

<!-- benchmark:tables:start -->

_Benchmark sweep generated on 2026-07-18 (median of 3 runs). Numbers track the host machine; the relative ordering between engines is the stable signal. Regenerated by `scripts/render-benchmark-tables.mjs`._

_Environment: runner `blacksmith-32vcpu-ubuntu-2404`, Node `v24.18.0`, Bun `1.3.14`, CPU `AMD EPYC`, 32 logical cores._

### Parse Only (48.7 KB)

| Library                               | ops/sec | avg time |  throughput |
| ------------------------------------- | ------: | -------: | ----------: |
| `ox-content (native)`                 |   8,417 |  0.12 ms | 400.53 MB/s |
| `pulldown-cmark`                      |   4,952 |  0.20 ms | 235.63 MB/s |
| `xai-grok-markdown-core (Grok Build)` |   4,733 |  0.21 ms | 225.22 MB/s |
| `@ox-content/napi`                    |   4,199 |  0.24 ms | 199.80 MB/s |
| `satteri`                             |   2,174 |  0.46 ms | 103.43 MB/s |
| `md4x (napi)`                         |   1,041 |  0.96 ms |  49.55 MB/s |
| `md4w (md4c)`                         |     923 |  1.08 ms |  43.93 MB/s |
| `markdown-it`                         |     704 |  1.42 ms |  33.49 MB/s |
| `marked`                              |     448 |  2.23 ms |  21.32 MB/s |
| `@mizchi/markdown`                    |      44 | 22.73 ms |   2.09 MB/s |
| `remark`                              |      30 | 33.40 ms |   1.42 MB/s |

### Parse + Render (48.7 KB)

| Library                      | ops/sec | avg time |  throughput |
| ---------------------------- | ------: | -------: | ----------: |
| `ox-content (native)`        |   5,679 |  0.18 ms | 270.25 MB/s |
| `@ox-content/napi`           |   4,799 |  0.21 ms | 228.39 MB/s |
| `pulldown-cmark + push_html` |   4,728 |  0.21 ms | 224.97 MB/s |
| `Bun.markdown.html`          |   3,947 |  0.25 ms | 187.81 MB/s |
| `md4x (napi)`                |   3,845 |  0.26 ms | 182.96 MB/s |
| `md4w (md4c)`                |   2,148 |  0.47 ms | 102.23 MB/s |
| `satteri`                    |   1,449 |  0.69 ms |  68.95 MB/s |
| `markdown-it`                |     574 |  1.74 ms |  27.31 MB/s |
| `marked`                     |     390 |  2.56 ms |  18.58 MB/s |
| `@mizchi/markdown`           |     347 |  2.88 ms |  16.53 MB/s |
| `micromark`                  |      31 | 32.46 ms |   1.47 MB/s |
| `remark`                     |      27 | 36.68 ms |   1.30 MB/s |

### Parse Only (~1 MB)

| Library                               | ops/sec |   avg time |  throughput |
| ------------------------------------- | ------: | ---------: | ----------: |
| `ox-content (native)`                 |     379 |    2.64 ms | 387.44 MB/s |
| `pulldown-cmark`                      |     231 |    4.33 ms | 236.54 MB/s |
| `xai-grok-markdown-core (Grok Build)` |     221 |    4.53 ms | 225.74 MB/s |
| `@ox-content/napi`                    |     192 |    5.21 ms | 196.34 MB/s |
| `satteri`                             |      89 |   11.25 ms |  90.97 MB/s |
| `md4x (napi)`                         |      45 |   22.04 ms |  46.42 MB/s |
| `md4w (md4c)`                         |      43 |   23.31 ms |  43.90 MB/s |
| `markdown-it`                         |      19 |   53.79 ms |  19.02 MB/s |
| `marked`                              |      16 |   62.87 ms |  16.27 MB/s |
| `@mizchi/markdown`                    |       1 |  786.15 ms |   1.30 MB/s |
| `remark`                              |       1 | 1234.09 ms |   0.83 MB/s |

### Parse + Render (~1 MB)

| Library                      | ops/sec |   avg time |  throughput |
| ---------------------------- | ------: | ---------: | ----------: |
| `ox-content (native)`        |     251 |    3.99 ms | 256.68 MB/s |
| `pulldown-cmark + push_html` |     221 |    4.52 ms | 226.25 MB/s |
| `@ox-content/napi`           |     203 |    4.93 ms | 207.66 MB/s |
| `Bun.markdown.html`          |     186 |    5.37 ms | 190.39 MB/s |
| `md4x (napi)`                |     176 |    5.67 ms | 180.29 MB/s |
| `md4w (md4c)`                |     105 |    9.55 ms | 107.17 MB/s |
| `satteri`                    |      56 |   17.92 ms |  57.10 MB/s |
| `markdown-it`                |      15 |   65.95 ms |  15.51 MB/s |
| `marked`                     |      14 |   73.34 ms |  13.95 MB/s |
| `@mizchi/markdown`           |      11 |   93.52 ms |  10.94 MB/s |
| `micromark`                  |       1 |  860.47 ms |   1.19 MB/s |
| `remark`                     |       1 | 1488.07 ms |   0.69 MB/s |

<!-- benchmark:tables:end -->

The benchmark tables above are regenerated from a clean Blacksmith 32 vCPU CI environment by the [Benchmark docs workflow](.github/workflows/benchmark-docs.yml); run `OX_CONTENT_BENCHMARK_RUNS=7 vp run bench:docs` to refresh them locally. Ox Content leads every comparison: 1.95× ahead of the next-fastest parser (`satteri`) on parse-only and 1.65× ahead of `md4x (napi)` on parse+render at 48.7 KB, and it holds that lead at ~1 MB (2.7× / 1.6×) while sustaining ~148–168 MB/s. The incremental CST parser (`@mizchi/markdown`) and the `unified`/`remark` and `micromark` pipelines fall to ~1 op/sec at 1 MB. See `node benchmarks/bundle-size/parse-benchmark.mjs` for the full sweep across small, medium, large, and ~1 MB inputs.

Run the benchmark with:

```bash
node benchmarks/bundle-size/parse-benchmark.mjs
```

The script compares against `satteri`, `@mizchi/markdown`, `md4w (md4c)`, and `md4x (napi)` by default, and includes `Bun.markdown.html` automatically when `bun` is installed.

## Development

```bash
nix develop           # Enter the pinned dev shell
vp install             # Install JS dependencies through Vite+
vp fmt                 # Format Rust and JS/TS sources
vp check               # Check Rust and JS/TS sources
vp dev                 # Start the docs and playground dev servers
vp build               # Build Rust, npm packages, docs, and playground
```

The dev shell is pinned in `flake.nix`, the workspace task graph lives in `vite.config.ts`, and `.node-version` is kept for CI / non-Nix Node setup.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for branch, commit, PR, testing, and release-note guidance.

See the [documentation](https://ubugeeei-prod.github.io/ox-content/) for more details.

## Community Credits

Special thanks to [kazupon](https://github.com/kazupon) for substantial community contributions around JSDoc support, including the API docs generation pipeline and documentation quality.

See [Credits](./docs/content/credits.md) for the contribution summary.

## Sponsor

If you find Ox Content useful, please consider [sponsoring](https://github.com/sponsors/ubugeeei) the project.

## License

MIT License - see [LICENSE](./LICENSE)
