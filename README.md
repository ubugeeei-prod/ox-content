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

_Benchmark sweep generated on 2026-07-30 (median of 7 runs). Numbers track the host machine; the relative ordering between engines is the stable signal. Regenerated by `scripts/render-benchmark-tables.mjs`._

_Environment: runner `blacksmith-32vcpu-ubuntu-2404`, Node `v24.18.1`, Bun `1.3.14`, CPU `Intel(R) Xeon(R) Processor`, 32 logical cores._

_CommonMark column: share of the 652 CommonMark 0.31.2 spec examples an engine renders correctly, measured by `benchmarks/commonmark-conformance/run.mjs`. Each engine runs in the most spec-faithful configuration it exposes, and both sides of the comparison pass through the conformance suite's HTML normalizer, so engines are ranked by behavior rather than by markup spelling._

### Parse Only (48.7 KB)

| Library                               | ops/sec | avg time |  throughput | CommonMark |
| ------------------------------------- | ------: | -------: | ----------: | ---------: |
| `ox-content (native)`                 |   7,494 |  0.13 ms | 356.63 MB/s |     100.0% |
| `pulldown-cmark`                      |   4,867 |  0.21 ms | 231.62 MB/s |     100.0% |
| `xai-grok-markdown-core (Grok Build)` |   4,167 |  0.24 ms | 198.28 MB/s |     100.0% |
| `@ox-content/napi`                    |   3,950 |  0.25 ms | 187.97 MB/s |      99.5% |
| `satteri`                             |   2,372 |  0.42 ms | 112.86 MB/s |      98.9% |
| `md4x (napi)`                         |     921 |  1.09 ms |  43.81 MB/s |      92.5% |
| `md4w (md4c)`                         |     893 |  1.12 ms |  42.50 MB/s |      91.7% |
| `markdown-it-ts`                      |     811 |  1.23 ms |  38.60 MB/s |     100.0% |
| `markdown-it`                         |     700 |  1.43 ms |  33.32 MB/s |     100.0% |
| `@tanstack/markdown`                  |     687 |  1.46 ms |  32.68 MB/s |      45.9% |
| `marked`                              |     444 |  2.25 ms |  21.12 MB/s |      93.4% |
| `@mizchi/markdown`                    |      48 | 20.75 ms |   2.29 MB/s |      45.9% |
| `remark`                              |      32 | 31.64 ms |   1.50 MB/s |      99.8% |

### Parse + Render (48.7 KB)

| Library                      | ops/sec | avg time |  throughput | CommonMark |
| ---------------------------- | ------: | -------: | ----------: | ---------: |
| `ox-content (native)`        |   5,195 |  0.19 ms | 247.19 MB/s |     100.0% |
| `pulldown-cmark + push_html` |   4,566 |  0.22 ms | 217.28 MB/s |     100.0% |
| `@ox-content/napi`           |   4,532 |  0.22 ms | 215.66 MB/s |      99.5% |
| `md4x (napi)`                |   3,437 |  0.29 ms | 163.56 MB/s |      92.5% |
| `Bun.markdown.html`          |   3,107 |  0.32 ms | 147.87 MB/s |     100.0% |
| `md4w (md4c)`                |   2,129 |  0.47 ms | 101.30 MB/s |      91.7% |
| `satteri`                    |   1,323 |  0.76 ms |  62.93 MB/s |      98.9% |
| `markdown-it-ts`             |     676 |  1.48 ms |  32.16 MB/s |     100.0% |
| `markdown-it`                |     563 |  1.77 ms |  26.81 MB/s |     100.0% |
| `@tanstack/markdown`         |     426 |  2.35 ms |  20.28 MB/s |      45.9% |
| `marked`                     |     405 |  2.47 ms |  19.26 MB/s |      93.4% |
| `@mizchi/markdown`           |     287 |  3.48 ms |  13.67 MB/s |      45.9% |
| `micromark`                  |      32 | 30.89 ms |   1.54 MB/s |     100.0% |
| `remark`                     |      27 | 37.31 ms |   1.28 MB/s |      99.8% |

### Parse Only (~1 MB)

| Library                               | ops/sec |   avg time |  throughput | CommonMark |
| ------------------------------------- | ------: | ---------: | ----------: | ---------: |
| `ox-content (native)`                 |     313 |    3.20 ms | 320.21 MB/s |     100.0% |
| `pulldown-cmark`                      |     219 |    4.56 ms | 224.20 MB/s |     100.0% |
| `xai-grok-markdown-core (Grok Build)` |     188 |    5.33 ms | 192.08 MB/s |     100.0% |
| `@ox-content/napi`                    |     175 |    5.71 ms | 179.05 MB/s |      99.5% |
| `satteri`                             |      81 |   12.34 ms |  82.94 MB/s |      98.9% |
| `md4w (md4c)`                         |      42 |   24.09 ms |  42.46 MB/s |      91.7% |
| `md4x (napi)`                         |      41 |   24.24 ms |  42.21 MB/s |      92.5% |
| `@tanstack/markdown`                  |      24 |   42.05 ms |  24.33 MB/s |      45.9% |
| `markdown-it-ts`                      |      21 |   47.31 ms |  21.62 MB/s |     100.0% |
| `markdown-it`                         |      20 |   49.38 ms |  20.72 MB/s |     100.0% |
| `marked`                              |      17 |   59.01 ms |  17.34 MB/s |      93.4% |
| `@mizchi/markdown`                    |       1 |  681.41 ms |   1.50 MB/s |      45.9% |
| `remark`                              |       1 | 1571.13 ms |   0.65 MB/s |      99.8% |

### Parse + Render (~1 MB)

| Library                      | ops/sec |   avg time |  throughput | CommonMark |
| ---------------------------- | ------: | ---------: | ----------: | ---------: |
| `ox-content (native)`        |     218 |    4.58 ms | 223.26 MB/s |     100.0% |
| `pulldown-cmark + push_html` |     201 |    4.97 ms | 205.71 MB/s |     100.0% |
| `@ox-content/napi`           |     193 |    5.19 ms | 197.05 MB/s |      99.5% |
| `md4x (napi)`                |     156 |    6.40 ms | 159.99 MB/s |      92.5% |
| `Bun.markdown.html`          |     140 |    7.16 ms | 142.98 MB/s |     100.0% |
| `md4w (md4c)`                |     108 |    9.27 ms | 110.41 MB/s |      91.7% |
| `satteri`                    |      73 |   13.77 ms |  74.29 MB/s |      98.9% |
| `markdown-it-ts`             |      18 |   54.08 ms |  18.92 MB/s |     100.0% |
| `@tanstack/markdown`         |      17 |   57.30 ms |  17.86 MB/s |      45.9% |
| `markdown-it`                |      17 |   58.81 ms |  17.40 MB/s |     100.0% |
| `marked`                     |      16 |   62.50 ms |  16.37 MB/s |      93.4% |
| `@mizchi/markdown`           |      12 |   81.45 ms |  12.56 MB/s |      45.9% |
| `micromark`                  |       1 |  765.78 ms |   1.34 MB/s |     100.0% |
| `remark`                     |       1 | 1739.84 ms |   0.59 MB/s |      99.8% |

<!-- benchmark:tables:end -->

The benchmark tables above are regenerated from a clean Blacksmith 32 vCPU CI environment by the [Benchmark docs workflow](.github/workflows/benchmark-docs.yml); run `OX_CONTENT_BENCHMARK_RUNS=7 vp run bench:docs` to refresh them locally. Against the TypeScript renderers, the JavaScript-facing `@ox-content/napi` row is 4.9–5.7× faster to parse and 6.7–10.6× faster to parse+render at 48.7 KB; at ~1 MB the leads grow to 7.3–8.3× and 10.7–11.4×. `@tanstack/markdown` uses `parseMarkdown` and `renderHtml`, while `markdown-it-ts` uses `parse` and `render` on a reused instance. See `node benchmarks/bundle-size/parse-benchmark.mjs` for the full sweep across small, medium, large, and ~1 MB inputs.

Run the benchmark with:

```bash
node benchmarks/bundle-size/parse-benchmark.mjs
```

The script compares against `@tanstack/markdown`, `markdown-it-ts`, `satteri`, `@mizchi/markdown`, `md4w (md4c)`, and `md4x (napi)` by default, and includes `Bun.markdown.html` automatically when `bun` is installed.

## CommonMark Conformance

Ox Content targets full CommonMark conformance. The engine is checked against the vendored [CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/) specification on every CI run, not only when the benchmark tables are refreshed:

- **Core profile: 652 / 652 examples.** `cargo test -p ox_content_renderer --test spec_commonmark` fails both when a passing example regresses and when a recorded failure starts passing, so the baseline cannot quietly drift.
- **GFM profile: 649 / 652 examples.** The three differences are spec examples 608, 611, and 612, where the GFM autolink extension deliberately linkifies bare URLs and emails that plain CommonMark leaves as text. They are listed in `crates/ox_content_renderer/tests/spec_fixtures/commonmark-known-failures.txt`.
- **GFM extensions: every example** in the GitHub Flavored Markdown 0.29-gfm spec sections for tables, task lists, strikethrough, autolinks, and disallowed raw HTML, driven by `spec_gfm.rs`.

Where the two Ox Content rows in the tables differ: `ox-content (native)` is the core profile and scores 100%, while `@ox-content/napi` scores 99.5% because its defaults enable the bare-URL autolinking builtin, which linkifies examples 602, 608, and 611. Pass `autolinkUrls: false` to turn it off.

Extensions beyond CommonMark — GFM tables, task lists, strikethrough, footnotes, and the built-in embeds — are opt-out rather than opt-in, so a document that uses none of them renders exactly as the specification requires. [Markdown Baseline](https://ubugeeei-prod.github.io/ox-content/built-in/markdown/) lists each toggle.

Ox Content does **not** extend CommonMark's emphasis rules for CJK text: `**` immediately inside CJK punctuation (`A**強調。**B`) is left as literal text, matching CommonMark and every other spec-conformant engine.

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
