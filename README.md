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

_Environment: runner `blacksmith-32vcpu-ubuntu-2404`, Node `v26.8.1`, Bun `1.3.14`, CPU `Intel(R) Xeon(R) Processor`, 32 logical cores._

_CommonMark column: share of the 652 CommonMark 0.31.2 spec examples an engine renders correctly, measured by `benchmarks/commonmark-conformance/run.mjs`. Each engine runs in the most spec-faithful configuration it exposes, and both sides of the comparison pass through the conformance suite's HTML normalizer, so engines are ranked by behavior rather than by markup spelling._

### Parse Only (48.7 KB)

| Library                               | ops/sec | avg time |  throughput | CommonMark |
| ------------------------------------- | ------: | -------: | ----------: | ---------: |
| `ox-content (native)`                 |  11,162 |  0.09 ms | 531.17 MB/s |     100.0% |
| `@ox-content/napi`                    |   5,502 |  0.18 ms | 261.84 MB/s |      99.5% |
| `pulldown-cmark`                      |   5,038 |  0.20 ms | 239.75 MB/s |     100.0% |
| `xai-grok-markdown-core (Grok Build)` |   4,316 |  0.23 ms | 205.40 MB/s |     100.0% |
| `satteri`                             |   1,299 |  0.77 ms |  61.83 MB/s |      98.9% |
| `md4x (napi)`                         |   1,258 |  0.79 ms |  59.86 MB/s |      99.5% |
| `md4x (wasm)`                         |   1,076 |  0.93 ms |  51.18 MB/s |      99.5% |
| `md4w (md4c)`                         |   1,012 |  0.99 ms |  48.14 MB/s |      91.7% |
| `markdown-it-ts`                      |     879 |  1.14 ms |  41.81 MB/s |     100.0% |
| `@tanstack/markdown`                  |     695 |  1.44 ms |  33.09 MB/s |      47.4% |
| `marked`                              |     404 |  2.47 ms |  19.24 MB/s |      93.4% |
| `markdown-it`                         |     282 |  3.55 ms |  13.42 MB/s |     100.0% |
| `@mizchi/markdown`                    |      71 | 13.99 ms |   3.40 MB/s |      98.3% |
| `remark`                              |      32 | 31.36 ms |   1.52 MB/s |      99.8% |

### Parse + Render (48.7 KB)

| Library                      | ops/sec | avg time |  throughput | CommonMark |
| ---------------------------- | ------: | -------: | ----------: | ---------: |
| `ox-content (native)`        |   7,279 |  0.14 ms | 346.38 MB/s |     100.0% |
| `@ox-content/napi`           |   6,671 |  0.15 ms | 317.45 MB/s |      99.5% |
| `pulldown-cmark + push_html` |   4,631 |  0.22 ms | 220.36 MB/s |     100.0% |
| `ferromark`                  |   3,995 |  0.25 ms | 190.11 MB/s |      88.8% |
| `md4x (napi)`                |   3,212 |  0.31 ms | 152.85 MB/s |      99.5% |
| `Bun.markdown.html`          |   3,138 |  0.32 ms | 149.31 MB/s |     100.0% |
| `satteri`                    |   2,999 |  0.33 ms | 142.73 MB/s |      98.9% |
| `md4w (md4c)`                |   2,322 |  0.43 ms | 110.49 MB/s |      91.7% |
| `md4x (wasm)`                |   2,208 |  0.45 ms | 105.08 MB/s |      99.5% |
| `markdown-it-ts`             |     750 |  1.33 ms |  35.71 MB/s |     100.0% |
| `@tanstack/markdown`         |     455 |  2.20 ms |  21.67 MB/s |      47.4% |
| `marked`                     |     369 |  2.71 ms |  17.54 MB/s |      93.4% |
| `@mizchi/markdown`           |     317 |  3.16 ms |  15.08 MB/s |      98.3% |
| `markdown-it`                |     257 |  3.90 ms |  12.21 MB/s |     100.0% |
| `micromark`                  |      34 | 29.49 ms |   1.61 MB/s |     100.0% |
| `remark`                     |      28 | 36.35 ms |   1.31 MB/s |      99.8% |

### Parse Only (~1 MB)

| Library                               | ops/sec |   avg time |  throughput | CommonMark |
| ------------------------------------- | ------: | ---------: | ----------: | ---------: |
| `ox-content (native)`                 |     508 |    1.97 ms | 519.33 MB/s |     100.0% |
| `pulldown-cmark`                      |     234 |    4.28 ms | 238.97 MB/s |     100.0% |
| `xai-grok-markdown-core (Grok Build)` |     199 |    5.02 ms | 203.83 MB/s |     100.0% |
| `@ox-content/napi`                    |     190 |    5.28 ms | 193.89 MB/s |      99.5% |
| `md4w (md4c)`                         |      48 |   20.77 ms |  49.27 MB/s |      91.7% |
| `md4x (wasm)`                         |      42 |   23.98 ms |  42.67 MB/s |      99.5% |
| `md4x (napi)`                         |      35 |   28.69 ms |  35.66 MB/s |      99.5% |
| `satteri`                             |      31 |   32.04 ms |  31.94 MB/s |      98.9% |
| `@tanstack/markdown`                  |      25 |   39.33 ms |  26.01 MB/s |      47.4% |
| `markdown-it-ts`                      |      23 |   43.35 ms |  23.60 MB/s |     100.0% |
| `marked`                              |      11 |   91.21 ms |  11.22 MB/s |      93.4% |
| `markdown-it`                         |      10 |   96.54 ms |  10.60 MB/s |     100.0% |
| `@mizchi/markdown`                    |       2 |  550.32 ms |   1.86 MB/s |      98.3% |
| `remark`                              |       1 | 1550.85 ms |   0.66 MB/s |      99.8% |

### Parse + Render (~1 MB)

| Library                      | ops/sec |   avg time |  throughput | CommonMark |
| ---------------------------- | ------: | ---------: | ----------: | ---------: |
| `ox-content (native)`        |     331 |    3.02 ms | 338.33 MB/s |     100.0% |
| `@ox-content/napi`           |     267 |    3.74 ms | 273.63 MB/s |      99.5% |
| `pulldown-cmark + push_html` |     217 |    4.60 ms | 222.39 MB/s |     100.0% |
| `ferromark`                  |     187 |    5.35 ms | 191.39 MB/s |      88.8% |
| `md4x (napi)`                |     144 |    6.95 ms | 147.18 MB/s |      99.5% |
| `Bun.markdown.html`          |     142 |    7.04 ms | 145.33 MB/s |     100.0% |
| `satteri`                    |     131 |    7.61 ms | 134.50 MB/s |      98.9% |
| `md4w (md4c)`                |     115 |    8.69 ms | 117.72 MB/s |      91.7% |
| `md4x (wasm)`                |     100 |    9.99 ms | 102.46 MB/s |      99.5% |
| `markdown-it-ts`             |      18 |   54.06 ms |  18.93 MB/s |     100.0% |
| `@tanstack/markdown`         |      18 |   54.87 ms |  18.65 MB/s |      47.4% |
| `marked`                     |      15 |   66.92 ms |  15.29 MB/s |      93.4% |
| `markdown-it`                |       9 |  107.62 ms |   9.51 MB/s |     100.0% |
| `@mizchi/markdown`           |       9 |  109.80 ms |   9.32 MB/s |      98.3% |
| `micromark`                  |       1 |  753.30 ms |   1.36 MB/s |     100.0% |
| `remark`                     |       1 | 1747.11 ms |   0.59 MB/s |      99.8% |

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
