<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="./assets/oxcontent-light.svg">
    <source media="(prefers-color-scheme: light)" srcset="./assets/oxcontent-dark.svg">
    <img alt="Ox Content logo" src="./assets/oxcontent-dark.svg" height="60">
  </picture>
</p>

<p align="center">
  <strong>cargo doc for JavaScript</strong><br>
  Rust-powered document generator and high-performance Markdown toolkit
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
- frontmatter schema completion and diagnostics
- i18n key completion, hover, go-to-definition, diagnostics, and inlay hints for JS/TS
- table / code fence / callout insertion commands
- preview HTML generation for editor UIs (with LSP-pushed HMR)
- `.mdc` authoring support with component tag diagnostics
- asset path completion inside `[…](`, `![…](`, and HTML `src=`/`href=` attributes
- dead link diagnostics powered by `ox_content_link_checker`
- opt-in [textlint](https://textlint.github.io) integration: on-save diagnostics under `source: "textlint"`
- MDC component name and attribute completion when a project provides a component registry

For CI or editor-independent checks, run:

```bash
cargo run -p ox_content_mdc_checker --bin ox-content-mdc-check -- docs/page.mdc
```

**[Read the full documentation →](https://ubugeeei-prod.github.io/ox-content/)**

## Performance

Ox Content is positioned both as a document generator and as a high-performance Markdown toolkit. The numbers below focus on the Markdown engine side.

<!-- benchmark:tables:start -->

_Benchmark sweep generated on 2026-05-30 (median of 1 runs). Numbers track the host machine; the relative ordering between engines is the stable signal. Regenerated by `scripts/render-benchmark-tables.mjs`._

### Parse Only (48.7 KB)

| Library            | ops/sec | avg time |  throughput |
| ------------------ | ------: | -------: | ----------: |
| `@ox-content/napi` |   2,578 |  0.39 ms | 122.67 MB/s |
| `satteri`          |   1,137 |  0.88 ms |  54.09 MB/s |
| `md4w (md4c)`      |     519 |  1.93 ms |  24.71 MB/s |
| `md4x (napi)`      |     513 |  1.95 ms |  24.40 MB/s |
| `markdown-it`      |     455 |  2.20 ms |  21.64 MB/s |
| `marked`           |     265 |  3.78 ms |  12.60 MB/s |
| `@mizchi/markdown` |      25 | 39.76 ms |   1.20 MB/s |
| `remark`           |      18 | 54.60 ms |   0.87 MB/s |

### Parse + Render (48.7 KB)

| Library             | ops/sec | avg time |  throughput |
| ------------------- | ------: | -------: | ----------: |
| `@ox-content/napi`  |   2,647 |  0.38 ms | 125.94 MB/s |
| `md4x (napi)`       |   1,461 |  0.68 ms |  69.53 MB/s |
| `md4w (md4c)`       |   1,140 |  0.88 ms |  54.23 MB/s |
| `Bun.markdown.html` |   1,123 |  0.89 ms |  53.46 MB/s |
| `satteri`           |     768 |  1.30 ms |  36.52 MB/s |
| `markdown-it`       |     320 |  3.12 ms |  15.25 MB/s |
| `marked`            |     199 |  5.02 ms |   9.48 MB/s |
| `@mizchi/markdown`  |     161 |  6.20 ms |   7.68 MB/s |
| `micromark`         |      22 | 45.49 ms |   1.05 MB/s |
| `remark`            |      17 | 59.75 ms |   0.80 MB/s |

### Parse Only (~1 MB)

| Library            | ops/sec |   avg time |  throughput |
| ------------------ | ------: | ---------: | ----------: |
| `@ox-content/napi` |     106 |    9.41 ms | 108.77 MB/s |
| `md4w (md4c)`      |      21 |   47.43 ms |  21.57 MB/s |
| `md4x (napi)`      |      18 |   56.83 ms |  18.00 MB/s |
| `satteri`          |      15 |   64.86 ms |  15.77 MB/s |
| `markdown-it`      |      13 |   75.78 ms |  13.50 MB/s |
| `marked`           |      10 |   99.02 ms |  10.33 MB/s |
| `@mizchi/markdown` |       0 | 3249.35 ms |   0.31 MB/s |
| `remark`           |       0 | 6797.62 ms |   0.15 MB/s |

### Parse + Render (~1 MB)

| Library             | ops/sec |   avg time | throughput |
| ------------------- | ------: | ---------: | ---------: |
| `@ox-content/napi`  |      88 |   11.40 ms | 89.79 MB/s |
| `Bun.markdown.html` |      75 |   13.29 ms | 76.97 MB/s |
| `md4x (napi)`       |      33 |   29.89 ms | 34.24 MB/s |
| `md4w (md4c)`       |      32 |   31.09 ms | 32.91 MB/s |
| `satteri`           |      22 |   45.26 ms | 22.61 MB/s |
| `markdown-it`       |       7 |  139.24 ms |  7.35 MB/s |
| `marked`            |       5 |  196.50 ms |  5.21 MB/s |
| `@mizchi/markdown`  |       3 |  290.00 ms |  3.53 MB/s |
| `micromark`         |       0 | 2255.02 ms |  0.45 MB/s |
| `remark`            |       0 | 9488.08 ms |  0.11 MB/s |

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

## Sponsor

If you find Ox Content useful, please consider [sponsoring](https://github.com/sponsors/ubugeeei) the project.

## License

MIT License - see [LICENSE](./LICENSE)
