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
vpx oxct migrate vitepress .vitepress/config.ts \
  --src-dir docs \
  --out-dir dist \
  --out ox-content.config.ts
```

The same migration runner is available through the unified `oxct` CLI:

```bash
vpx oxct migrate vitepress .vitepress/config.ts --out ox-content.config.ts
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

# Solid (@solidjs/vite-plugin compiles generated Markdown JSX)
npm install @ox-content/vite-plugin-solid solid-js@next @solidjs/web@next @solidjs/vite-plugin
```

### i18n Static Checker (CLI)

```bash
# Check for missing/unused translation keys after installing @ox-content/vite-plugin
vpx oxct i18n check --dict-dir content/i18n --src src

# Validate an ICU MessageFormat 2 message
vpx oxct i18n validate "Hello {$name}"
```

### Dead Link Checker (CLI)

```bash
# Check every link in a tree, exit non-zero on broken targets
vpx oxct link-check docs/**/*.md

# Treat `/foo.md` as workspace-rooted under docs/
vpx oxct link-check --src-dir docs docs/**/*.md

# Suppress known intentionally-broken targets
vpx oxct link-check --ignore "intentionally-broken" docs/**/*.md
```

Offline-only by design — `http://` and `https://` links pass through
without a network request, so the same binary is safe to run in CI
without timeouts, retries, or rate limits.

### Editor Tooling

Ox Content now ships a unified authoring and i18n language server:

```bash
vpx oxct lsp
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
vpx oxct mdc-check docs/page.mdc
```

To preview a generated Open Graph image as SVG:

```bash
vpx oxct og-preview --title "My Docs" --description "Fast content tooling" --out og.svg
```

**[Read the full documentation →](https://ubugeeei-prod.github.io/ox-content/)**

## Performance

Ox Content is positioned both as a document generator and as a high-performance Markdown toolkit. The numbers below focus on the Markdown engine side.

Speed is only half of a fair comparison: Markdown engines differ in how much of CommonMark they implement, and some deliberately trade spec coverage for throughput. Every row therefore carries a measured CommonMark conformance rate next to its speed, so a faster engine that skips spec behavior is visible as such rather than simply ranking higher. See [CommonMark conformance](#commonmark-conformance) for how Ox Content itself scores.

Ferromark v2.1.2 is a [source fork of Ox Content v3.2.3](https://github.com/sebastian-software/ferromark/blob/v2.1.2/UPSTREAM.md) with a separate Git history. Its benchmark row measures a related implementation, not an independent parser design.

<!-- benchmark:tables:start -->

_Benchmark sweep generated on 2026-09-25 (median of 7 runs). Numbers track the host machine; the relative ordering between engines is the stable signal. Regenerated by `tools/scripts/render-benchmark-tables.mjs`._

_Environment: runner `blacksmith-32vcpu-ubuntu-2404`, Node `v26.8.1`, Bun `1.3.14`, CPU `AMD EPYC`, 32 logical cores._

_CommonMark column: share of the 652 CommonMark 0.31.2 spec examples an engine renders correctly, measured by `tools/benchmarks/commonmark-conformance/run.mjs`. Each engine runs in the most spec-faithful configuration it exposes, and both sides of the comparison pass through the conformance suite's HTML normalizer, so engines are ranked by behavior rather than by markup spelling._

### Parse Only (48.7 KB)

| Library                               | ops/sec | avg time |  throughput | CommonMark |
| ------------------------------------- | ------: | -------: | ----------: | ---------: |
| `ferromark`                           |  11,226 |  0.09 ms | 534.19 MB/s |     100.0% |
| `ox-content (native)`                 |  10,435 |  0.10 ms | 496.54 MB/s |     100.0% |
| `@ox-content/napi`                    |   5,712 |  0.18 ms | 271.82 MB/s |      99.5% |
| `pulldown-cmark`                      |   5,243 |  0.19 ms | 249.49 MB/s |     100.0% |
| `xai-grok-markdown-core (Grok Build)` |   4,979 |  0.20 ms | 236.93 MB/s |     100.0% |
| `md4x (napi)`                         |   1,443 |  0.69 ms |  68.67 MB/s |      99.5% |
| `satteri`                             |   1,427 |  0.70 ms |  67.90 MB/s |      98.9% |
| `md4w (md4c)`                         |   1,071 |  0.93 ms |  50.95 MB/s |      91.7% |
| `md4x (wasm)`                         |   1,050 |  0.95 ms |  49.98 MB/s |      99.5% |
| `markdown-it-ts`                      |     911 |  1.10 ms |  43.37 MB/s |     100.0% |
| `@mizchi/markdown (native)`           |     787 |  1.27 ms |  37.43 MB/s |     100.0% |
| `@tanstack/markdown`                  |     598 |  1.67 ms |  28.48 MB/s |      53.5% |
| `@mizchi/markdown (js)`               |     536 |  1.87 ms |  25.49 MB/s |     100.0% |
| `@mizchi/markdown (wasm)`             |     509 |  1.96 ms |  24.23 MB/s |     100.0% |
| `marked`                              |     401 |  2.49 ms |  19.08 MB/s |      95.9% |
| `markdown-it`                         |     325 |  3.08 ms |  15.45 MB/s |     100.0% |
| `remark`                              |      30 | 32.88 ms |   1.45 MB/s |      99.8% |

### Parse + Render (48.7 KB)

| Library                      | ops/sec | avg time |  throughput | CommonMark |
| ---------------------------- | ------: | -------: | ----------: | ---------: |
| `ferromark`                  |   7,288 |  0.14 ms | 346.83 MB/s |     100.0% |
| `ox-content (native)`        |   7,260 |  0.14 ms | 345.47 MB/s |     100.0% |
| `@ox-content/napi`           |   6,428 |  0.16 ms | 305.90 MB/s |      99.5% |
| `pulldown-cmark + push_html` |   4,987 |  0.20 ms | 237.30 MB/s |     100.0% |
| `Bun.markdown.html`          |   3,987 |  0.25 ms | 189.73 MB/s |     100.0% |
| `md4x (napi)`                |   3,623 |  0.28 ms | 172.38 MB/s |      99.5% |
| `satteri`                    |   3,397 |  0.29 ms | 161.64 MB/s |      98.9% |
| `md4w (md4c)`                |   2,361 |  0.42 ms | 112.37 MB/s |      91.7% |
| `md4x (wasm)`                |   2,234 |  0.45 ms | 106.33 MB/s |      99.5% |
| `markdown-it-ts`             |     778 |  1.29 ms |  37.03 MB/s |     100.0% |
| `@mizchi/markdown (native)`  |     601 |  1.66 ms |  28.61 MB/s |     100.0% |
| `@mizchi/markdown (wasm)`    |     524 |  1.91 ms |  24.92 MB/s |     100.0% |
| `@mizchi/markdown (js)`      |     496 |  2.01 ms |  23.63 MB/s |     100.0% |
| `@tanstack/markdown`         |     453 |  2.21 ms |  21.56 MB/s |      53.5% |
| `marked`                     |     359 |  2.78 ms |  17.09 MB/s |      95.9% |
| `markdown-it`                |     293 |  3.42 ms |  13.92 MB/s |     100.0% |
| `micromark`                  |      31 | 32.10 ms |   1.48 MB/s |     100.0% |
| `remark`                     |      26 | 38.55 ms |   1.23 MB/s |      99.8% |

### Parse Only (~1 MB)

| Library                               | ops/sec |   avg time |  throughput | CommonMark |
| ------------------------------------- | ------: | ---------: | ----------: | ---------: |
| `ferromark`                           |     520 |    1.92 ms | 531.99 MB/s |     100.0% |
| `ox-content (native)`                 |     487 |    2.05 ms | 498.46 MB/s |     100.0% |
| `@ox-content/napi`                    |     294 |    3.40 ms | 300.92 MB/s |      99.5% |
| `pulldown-cmark`                      |     239 |    4.18 ms | 244.57 MB/s |     100.0% |
| `xai-grok-markdown-core (Grok Build)` |     226 |    4.43 ms | 230.84 MB/s |     100.0% |
| `md4x (napi)`                         |      53 |   18.86 ms |  54.26 MB/s |      99.5% |
| `md4w (md4c)`                         |      50 |   20.03 ms |  51.09 MB/s |      91.7% |
| `md4x (wasm)`                         |      42 |   23.76 ms |  43.07 MB/s |      99.5% |
| `satteri`                             |      26 |   38.29 ms |  26.72 MB/s |      98.9% |
| `@mizchi/markdown (native)`           |      24 |   42.20 ms |  24.25 MB/s |     100.0% |
| `markdown-it-ts`                      |      22 |   45.08 ms |  22.70 MB/s |     100.0% |
| `@tanstack/markdown`                  |      22 |   46.36 ms |  22.07 MB/s |      53.5% |
| `marked`                              |      15 |   65.78 ms |  15.55 MB/s |      95.9% |
| `@mizchi/markdown (wasm)`             |      11 |   89.60 ms |  11.42 MB/s |     100.0% |
| `markdown-it`                         |      11 |   89.82 ms |  11.39 MB/s |     100.0% |
| `@mizchi/markdown (js)`               |      10 |   95.49 ms |  10.71 MB/s |     100.0% |
| `remark`                              |       1 | 1134.23 ms |   0.90 MB/s |      99.8% |

### Parse + Render (~1 MB)

| Library                      | ops/sec |   avg time |  throughput | CommonMark |
| ---------------------------- | ------: | ---------: | ----------: | ---------: |
| `ferromark`                  |     325 |    3.08 ms | 332.39 MB/s |     100.0% |
| `ox-content (native)`        |     319 |    3.14 ms | 325.90 MB/s |     100.0% |
| `@ox-content/napi`           |     253 |    3.95 ms | 259.05 MB/s |      99.5% |
| `pulldown-cmark + push_html` |     227 |    4.40 ms | 232.55 MB/s |     100.0% |
| `Bun.markdown.html`          |     188 |    5.31 ms | 192.76 MB/s |     100.0% |
| `md4x (napi)`                |     162 |    6.17 ms | 165.92 MB/s |      99.5% |
| `satteri`                    |     155 |    6.47 ms | 158.25 MB/s |      98.9% |
| `md4w (md4c)`                |     116 |    8.64 ms | 118.39 MB/s |      91.7% |
| `md4x (wasm)`                |     105 |    9.51 ms | 107.64 MB/s |      99.5% |
| `@mizchi/markdown (native)`  |      20 |   49.52 ms |  20.66 MB/s |     100.0% |
| `markdown-it-ts`             |      18 |   55.24 ms |  18.52 MB/s |     100.0% |
| `@tanstack/markdown`         |      16 |   62.42 ms |  16.39 MB/s |      53.5% |
| `marked`                     |      13 |   78.07 ms |  13.11 MB/s |      95.9% |
| `@mizchi/markdown (wasm)`    |      13 |   79.72 ms |  12.83 MB/s |     100.0% |
| `@mizchi/markdown (js)`      |      11 |   89.38 ms |  11.45 MB/s |     100.0% |
| `markdown-it`                |      10 |  102.62 ms |   9.97 MB/s |     100.0% |
| `micromark`                  |       1 |  777.83 ms |   1.32 MB/s |     100.0% |
| `remark`                     |       1 | 1439.29 ms |   0.71 MB/s |      99.8% |

<!-- benchmark:tables:end -->

The benchmark tables above are regenerated from a clean Blacksmith 32 vCPU CI environment by the [Benchmark docs workflow](.github/workflows/benchmark-docs.yml); run `OX_CONTENT_BENCHMARK_RUNS=7 vp run bench:docs` to refresh them locally. Against the TypeScript renderers, the JavaScript-facing `@ox-content/napi` row is 4.0–5.1× faster to parse and 7.1–12.1× faster to parse+render at 48.7 KB; at ~1 MB the leads grow to 7.7–8.7× and 11.4–12.0×. `@tanstack/markdown` uses `parseMarkdown` and `renderHtml`, while `markdown-it-ts` uses `parse` and `render` on a reused instance. See `node tools/benchmarks/bundle-size/parse-benchmark.mjs` for the full sweep across small, medium, large, and ~1 MB inputs.

Run the benchmark with:

```bash
node tools/benchmarks/bundle-size/parse-benchmark.mjs
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

The dev shell is pinned in `flake.nix`, the workspace task graph lives in `vite.config.ts`, and `package.json` declares the Node.js runtime through `devEngines.runtime`.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for branch, commit, PR, testing, and release-note guidance.

See the [documentation](https://ubugeeei-prod.github.io/ox-content/) for more details.

## Community Credits

Special thanks to [kazupon](https://github.com/kazupon) for substantial community contributions around JSDoc support, including the API docs generation pipeline and documentation quality.

See [Credits](./docs/content/credits.md) for the contribution summary.

## Sponsor

If you find Ox Content useful, please consider [sponsoring](https://github.com/sponsors/ubugeeei) the project.

## License

MIT License - see [LICENSE](./LICENSE)
