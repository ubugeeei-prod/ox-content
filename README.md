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
- **Built-in MDX** - `.mdx` files parse JSX, ESM, and `{expression}` by default; `.md` stays GFM. Static HTML plus optional component islands
- **GFM Support** - Tables, task lists, strikethrough, autolinks, footnotes
- **Multi-Runtime** - Node.js (NAPI), WebAssembly, Native Rust
- **Framework Agnostic** - Works with Vue, React, Svelte, and more
- **Built-in SSG** - Static site generation with theming, search, and OG images
- **Built-in Embeds** - Static GitHub repository, source code, and Open Graph link cards
- **Code Play** - Opt-in `@ox-content/code-play` plugin for on-demand sample execution
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

# Solid (vite-plugin-solid compiles the generated JSX, so it is required)
npm install @ox-content/vite-plugin-solid solid-js vite-plugin-solid
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

Speed is only half of a fair comparison: Markdown engines differ in how much of CommonMark they implement, and some deliberately trade spec coverage for throughput. Every row therefore carries a measured CommonMark conformance rate next to its speed, so a faster engine that skips spec behavior is visible as such rather than simply ranking higher. See [CommonMark conformance](#commonmark-conformance) for how Ox Content itself scores.

<!-- benchmark:tables:start -->

_Benchmark sweep generated on 2026-08-30 (median of 7 runs). Numbers track the host machine; the relative ordering between engines is the stable signal. Regenerated by `scripts/render-benchmark-tables.mjs`._

_Environment: runner `blacksmith-32vcpu-ubuntu-2404`, Node `v26.8.1`, Bun `1.3.14`, CPU `AMD EPYC`, 32 logical cores._

_CommonMark column: share of the 652 CommonMark 0.31.2 spec examples an engine renders correctly, measured by `benchmarks/commonmark-conformance/run.mjs`. Each engine runs in the most spec-faithful configuration it exposes, and both sides of the comparison pass through the conformance suite's HTML normalizer, so engines are ranked by behavior rather than by markup spelling._

### Parse Only (48.7 KB)

| Library                               | ops/sec | avg time |  throughput | CommonMark |
| ------------------------------------- | ------: | -------: | ----------: | ---------: |
| `ox-content (native)`                 |  11,411 |  0.09 ms | 542.99 MB/s |     100.0% |
| `@ox-content/napi`                    |   5,880 |  0.17 ms | 279.79 MB/s |      99.5% |
| `pulldown-cmark`                      |   5,066 |  0.20 ms | 241.06 MB/s |     100.0% |
| `xai-grok-markdown-core (Grok Build)` |   4,563 |  0.22 ms | 217.13 MB/s |     100.0% |
| `md4x (napi)`                         |   1,400 |  0.71 ms |  66.64 MB/s |      99.5% |
| `satteri`                             |   1,378 |  0.73 ms |  65.59 MB/s |      98.9% |
| `md4x (wasm)`                         |   1,066 |  0.94 ms |  50.72 MB/s |      99.5% |
| `md4w (md4c)`                         |   1,060 |  0.94 ms |  50.46 MB/s |      91.7% |
| `markdown-it-ts`                      |     868 |  1.15 ms |  41.30 MB/s |     100.0% |
| `@tanstack/markdown`                  |     742 |  1.35 ms |  35.32 MB/s |      47.4% |
| `@mizchi/markdown (native)`           |     708 |  1.41 ms |  33.71 MB/s |     100.0% |
| `@mizchi/markdown (js)`               |     555 |  1.80 ms |  26.41 MB/s |     100.0% |
| `@mizchi/markdown (wasm)`             |     551 |  1.81 ms |  26.24 MB/s |     100.0% |
| `marked`                              |     387 |  2.58 ms |  18.44 MB/s |      93.4% |
| `markdown-it`                         |     314 |  3.18 ms |  14.97 MB/s |     100.0% |
| `remark`                              |      31 | 32.73 ms |   1.45 MB/s |      99.8% |

### Parse + Render (48.7 KB)

| Library                      | ops/sec | avg time |  throughput | CommonMark |
| ---------------------------- | ------: | -------: | ----------: | ---------: |
| `ox-content (native)`        |   7,445 |  0.13 ms | 354.28 MB/s |     100.0% |
| `@ox-content/napi`           |   7,375 |  0.14 ms | 350.97 MB/s |      99.5% |
| `pulldown-cmark + push_html` |   4,742 |  0.21 ms | 225.66 MB/s |     100.0% |
| `ferromark`                  |   4,389 |  0.23 ms | 208.87 MB/s |      88.8% |
| `Bun.markdown.html`          |   3,886 |  0.26 ms | 184.93 MB/s |     100.0% |
| `md4x (napi)`                |   3,729 |  0.27 ms | 177.47 MB/s |      99.5% |
| `satteri`                    |   3,332 |  0.30 ms | 158.56 MB/s |      98.9% |
| `md4w (md4c)`                |   2,370 |  0.42 ms | 112.78 MB/s |      91.7% |
| `md4x (wasm)`                |   2,211 |  0.45 ms | 105.22 MB/s |      99.5% |
| `markdown-it-ts`             |     703 |  1.42 ms |  33.46 MB/s |     100.0% |
| `@mizchi/markdown (native)`  |     664 |  1.51 ms |  31.60 MB/s |     100.0% |
| `@mizchi/markdown (wasm)`    |     520 |  1.92 ms |  24.73 MB/s |     100.0% |
| `@tanstack/markdown`         |     493 |  2.03 ms |  23.44 MB/s |      47.4% |
| `@mizchi/markdown (js)`      |     482 |  2.08 ms |  22.92 MB/s |     100.0% |
| `marked`                     |     366 |  2.73 ms |  17.42 MB/s |      93.4% |
| `markdown-it`                |     285 |  3.51 ms |  13.55 MB/s |     100.0% |
| `micromark`                  |      30 | 32.82 ms |   1.45 MB/s |     100.0% |
| `remark`                     |      25 | 39.26 ms |   1.21 MB/s |      99.8% |

### Parse Only (~1 MB)

| Library                               | ops/sec |   avg time |  throughput | CommonMark |
| ------------------------------------- | ------: | ---------: | ----------: | ---------: |
| `ox-content (native)`                 |     528 |    1.89 ms | 539.95 MB/s |     100.0% |
| `@ox-content/napi`                    |     303 |    3.30 ms | 309.82 MB/s |      99.5% |
| `pulldown-cmark`                      |     233 |    4.28 ms | 238.82 MB/s |     100.0% |
| `xai-grok-markdown-core (Grok Build)` |     213 |    4.70 ms | 217.84 MB/s |     100.0% |
| `md4w (md4c)`                         |      48 |   20.81 ms |  49.16 MB/s |      91.7% |
| `md4x (napi)`                         |      46 |   21.59 ms |  47.39 MB/s |      99.5% |
| `md4x (wasm)`                         |      44 |   22.98 ms |  44.52 MB/s |      99.5% |
| `satteri`                             |      26 |   37.99 ms |  26.94 MB/s |      98.9% |
| `@tanstack/markdown`                  |      23 |   43.71 ms |  23.41 MB/s |      47.4% |
| `@mizchi/markdown (native)`           |      22 |   44.47 ms |  23.01 MB/s |     100.0% |
| `markdown-it-ts`                      |      22 |   45.92 ms |  22.28 MB/s |     100.0% |
| `marked`                              |      14 |   70.42 ms |  14.53 MB/s |      93.4% |
| `@mizchi/markdown (wasm)`             |      11 |   87.27 ms |  11.72 MB/s |     100.0% |
| `markdown-it`                         |      10 |   97.51 ms |  10.49 MB/s |     100.0% |
| `@mizchi/markdown (js)`               |      10 |   98.26 ms |  10.41 MB/s |     100.0% |
| `remark`                              |       1 | 1169.06 ms |   0.88 MB/s |      99.8% |

### Parse + Render (~1 MB)

| Library                      | ops/sec |   avg time |  throughput | CommonMark |
| ---------------------------- | ------: | ---------: | ----------: | ---------: |
| `ox-content (native)`        |     329 |    3.04 ms | 336.45 MB/s |     100.0% |
| `@ox-content/napi`           |     291 |    3.44 ms | 297.82 MB/s |      99.5% |
| `pulldown-cmark + push_html` |     222 |    4.50 ms | 227.46 MB/s |     100.0% |
| `ferromark`                  |     207 |    4.82 ms | 212.13 MB/s |      88.8% |
| `Bun.markdown.html`          |     189 |    5.30 ms | 192.92 MB/s |     100.0% |
| `md4x (napi)`                |     168 |    5.96 ms | 171.66 MB/s |      99.5% |
| `satteri`                    |     151 |    6.64 ms | 154.10 MB/s |      98.9% |
| `md4w (md4c)`                |     120 |    8.32 ms | 122.98 MB/s |      91.7% |
| `md4x (wasm)`                |     106 |    9.46 ms | 108.11 MB/s |      99.5% |
| `@mizchi/markdown (native)`  |      19 |   51.49 ms |  19.87 MB/s |     100.0% |
| `markdown-it-ts`             |      17 |   58.20 ms |  17.58 MB/s |     100.0% |
| `@tanstack/markdown`         |      16 |   63.37 ms |  16.15 MB/s |      47.4% |
| `marked`                     |      13 |   79.68 ms |  12.84 MB/s |      93.4% |
| `@mizchi/markdown (wasm)`    |      12 |   81.01 ms |  12.63 MB/s |     100.0% |
| `@mizchi/markdown (js)`      |      11 |   90.76 ms |  11.27 MB/s |     100.0% |
| `markdown-it`                |       9 |  107.83 ms |   9.49 MB/s |     100.0% |
| `micromark`                  |       1 |  822.92 ms |   1.24 MB/s |     100.0% |
| `remark`                     |       1 | 1975.03 ms |   0.52 MB/s |      99.8% |

<!-- benchmark:tables:end -->

The benchmark tables above are regenerated from a clean Blacksmith 32 vCPU CI environment by the [Benchmark docs workflow](.github/workflows/benchmark-docs.yml); run `OX_CONTENT_BENCHMARK_RUNS=7 vp run bench:docs` to refresh them locally. Against the TypeScript renderers, the JavaScript-facing `@ox-content/napi` row is 4.0–5.1× faster to parse and 7.1–12.1× faster to parse+render at 48.7 KB; at ~1 MB the leads grow to 7.7–8.7× and 11.4–12.0×. `@tanstack/markdown` uses `parseMarkdown` and `renderHtml`, while `markdown-it-ts` uses `parse` and `render` on a reused instance. See `node benchmarks/bundle-size/parse-benchmark.mjs` for the full sweep across small, medium, large, and ~1 MB inputs.

Run the benchmark with:

```bash
node benchmarks/bundle-size/parse-benchmark.mjs
```

The script compares against `@tanstack/markdown`, `markdown-it-ts`, `satteri`, `@mizchi/markdown` (JS, Wasm, and native), `md4w (md4c)`, and `md4x` (NAPI and Wasm) by default, and includes `Bun.markdown.html` automatically when `bun` is installed.

## CommonMark Conformance

Ox Content targets full CommonMark conformance. The engine is checked against the vendored [CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/) specification on every CI run, not only when the benchmark tables are refreshed:

- **Core profile: 652 / 652 examples.** `cargo test -p ox_content_renderer --test spec_commonmark` fails both when a passing example regresses and when a recorded failure starts passing, so the baseline cannot quietly drift.
- **GFM profile: 649 / 652 examples.** The three differences are spec examples 608, 611, and 612, where the GFM autolink extension deliberately linkifies bare URLs and emails that plain CommonMark leaves as text. They are listed in `crates/ox_content_renderer/tests/spec_fixtures/commonmark-known-failures.txt`.
- **GFM extensions: every example** in the GitHub Flavored Markdown 0.29-gfm spec sections for tables, task lists, strikethrough, autolinks, and disallowed raw HTML, driven by `spec_gfm.rs`.

Where the two Ox Content rows in the tables differ: `ox-content (native)` is the core profile and scores 100%, while `@ox-content/napi` scores 99.5% because its defaults enable the bare-URL autolinking builtin, which linkifies examples 602, 608, and 611. Pass `autolinkUrls: false` to turn it off. The `@mizchi/markdown` JS, Wasm, and native rows likewise disable their default autolink and tagfilter extensions for the CommonMark column; the speed rows keep runtime defaults.

Extensions beyond CommonMark — GFM tables, task lists, strikethrough, footnotes, and the built-in embeds — are opt-out rather than opt-in, so a document that uses none of them renders exactly as the specification requires. [Markdown Baseline](https://ubugeeei-prod.github.io/ox-content/built-in/markdown/) lists each toggle.

One deliberate deviation is available opt-in. CommonMark's flanking rules leave `**` immediately inside CJK punctuation (`A**強調。**B`) as literal text, which bites CJK prose constantly because punctuation is set directly against the preceding word. Enabling [`cjkEmphasis`](https://ubugeeei-prod.github.io/ox-content/examples/cjk-emphasis/) makes those runs pair; halfwidth ASCII punctuation is untouched, so Latin documents parse identically. It is off by default so the shipped default stays spec-conformant.

## Development

```bash
nix develop           # Enter the pinned dev shell
vp install             # Install JS dependencies through Vite+
vp fmt                 # Format Rust and JS/TS sources
vp check               # Check Rust and JS/TS sources
vp run dev             # Start the docs and playground dev servers
vp run build           # Build Rust, npm packages, docs, and playground
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
