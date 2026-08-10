# Benchmarks

Two flavors of performance measurement live in this tree, plus a correctness
sweep that the published tables depend on and a one-off compiler experiment:

- **JS comparison sweep** in `bundle-size/` — runs Ox Content alongside
  `@tanstack/markdown`, `markdown-it-ts`, `markdown-it`, `marked`, `md4w`,
  `md4x`, and friends to compare ops/sec and throughput at small / medium /
  large input sizes. Trigger via `node benchmarks/bundle-size/parse-benchmark.mjs` (see
  [`bundle-size/README` flow in the top-level README](../README.md#performance)).
- **Rust criterion suites** under `crates/ox_content_parser/benches/` —
  in-process measurements that avoid NAPI overhead. Trigger via
  `cargo bench -p ox_content_parser`.
- **CommonMark conformance sweep** in `commonmark-conformance/` — scores every
  engine in the speed tables against the vendored CommonMark 0.31.2 spec, so a
  faster engine that skips spec behavior is visible as such. Trigger via
  `node benchmarks/commonmark-conformance/run.mjs`.
- **Polonius borrow-checker comparison** in `polonius-borrowck/` — measures what
  the next-generation borrow checker costs this workspace to compile, and lists
  the borrow-checker workarounds it would let us delete once it stabilizes. This
  one measures the compiler, not Ox Content, so it feeds no published table.
  Trigger via
  `node benchmarks/polonius-borrowck/run.mjs --toolchain nightly-2026-08-09`.

## CommonMark conformance sweep

`commonmark-conformance/run.mjs` renders all 652 spec examples with every engine
the speed tables list and writes `results.json`, which
`scripts/render-benchmark-tables.mjs` reads to fill the `CommonMark` column in
`README.md` and `docs/content/performance.md`.

Two decisions make the comparison fair, and both are load-bearing:

- **Configuration.** Each engine runs in the most spec-faithful mode it exposes,
  not in its benchmark defaults: `markdown-it` and `markdown-it-ts` use their
  `commonmark` presets, `micromark` and `remark-html` are told to pass raw HTML
  through, and `md4w` runs with `parseFlags: 0`. Otherwise the column would
  measure a default preset rather than the engine. Engines with no such mode
  (`marked`, `md4x`, `@tanstack/markdown`, `@mizchi/markdown`) are scored as
  they ship, which each entry notes.
- **Comparison.** Both sides pass through `normalize_html`, the normalizer the
  in-repo conformance suite uses, reached through the native binary's
  `--normalize` filter so JS and native engines are judged identically. A
  byte-exact comparison would rank engines by markup spelling instead: entity
  spelling, attribute order, `<br />` vs `<br>`, and the slug `id` ox-content
  adds to headings all differ without changing how a document renders.
  ox-content scores 82.5% byte-exact and 100% normalized for exactly that
  reason, and `pulldown-cmark` — an independent Rust implementation, unrelated
  to ox-content — scores 100% under the normalizer too, which is the check that
  it is not tuned to one engine.

The sweep needs `cargo` because the normalizer lives in the native binary; `bun`
is optional and adds the `Bun.markdown.html` row.

Regenerate with:

```bash
node benchmarks/commonmark-conformance/run.mjs --json benchmarks/commonmark-conformance/results.json
```

## Native Rust competitor rows

The JS sweep also injects rows from a standalone cargo crate in
`native-competitors/` (built and run as a subprocess when `cargo` is on the
PATH, skipped otherwise):

- **`ox-content (native)`** (parse and render) — the engine itself, called
  directly by path dependency: a full arena parse producing the AST, and
  parse + HTML render with the same defaults as the `@ox-content/napi`
  `parseAndRender` row. No napi boundary, no mdast serialization — the gap
  between this row and the `@ox-content/napi` rows _is_ the JS hand-off
  cost. Because the crate is built from the benchmarked checkout, base and
  head runs each measure their own core. Note the comparison asymmetry:
  this row builds a full AST while the pulldown rows only drain a
  streaming event iterator, and (like the napi rows) it parses with
  default options, so GFM tables stay off while the Grok option set
  enables them.
- **`xai-grok-markdown-core (Grok Build)`** (parse) — drains
  `offset_events()`, the exact parse path of the markdown stack xAI
  open-sourced with [Grok Build](https://github.com/xai-org/grok-build)
  (Apache-2.0, pinned to rev `98c3b2438aa922fbbe6178a5c0a4c48f85edc8ce`): a
  lean wrapper around pulldown-cmark with Grok's option set and single-tilde
  strikethrough demoted.
- **`pulldown-cmark`** (parse) — plain pulldown-cmark event draining under the
  same option set, isolating the wrapper's overhead.
- **`pulldown-cmark + push_html`** (render) — parse plus
  `pulldown_cmark::html::push_html` into a fresh output string.

Caveat: xai-grok-markdown's own output target is terminal rendering, not
HTML, so its HTML-comparable surface is the parse side (`offset_events`);
pulldown-cmark's `push_html` stands in for the render side of that stack.
The runner mirrors the JS harness protocol byte-for-byte (same sample
document, sizes, warmup, iteration counts, and `--runs` median selection),
which its unit tests pin against `parse-benchmark-bun.mjs`.

## OSS Markdown corpus (real-world inputs)

The Rust `corpus` benchmark target measures parse and parse+render against
real-world Markdown trees taken from MIT / Apache-2.0 licensed OSS projects.
Because each corpus is multiple megabytes, the actual files are not
checked in — populate them on demand with:

```bash
node scripts/fetch-bench-corpus.mjs
```

This sparse-checkouts the docs subtree of each upstream repo into
`benchmarks/corpus/<project>/`. Today the script tracks:

| project               | source                                                                                                             | license           |
| --------------------- | ------------------------------------------------------------------------------------------------------------------ | ----------------- |
| `vue-docs`            | [vuejs/docs](https://github.com/vuejs/docs)                                                                        | MIT               |
| `vite-docs`           | [vitejs/vite](https://github.com/vitejs/vite) (`docs/`)                                                            | MIT               |
| `rust-book`           | [rust-lang/book](https://github.com/rust-lang/book)                                                                | MIT OR Apache-2.0 |
| `typescript-handbook` | [microsoft/TypeScript-Website](https://github.com/microsoft/TypeScript-Website) (`packages/documentation/copy/en`) | MIT               |

After fetching, run:

```bash
cargo bench -p ox_content_parser --bench corpus
```

The benchmark gracefully no-ops when the corpus directory is empty so a
fresh checkout can run `cargo bench` without first downloading anything.

Each upstream LICENSE file is included in the sparse checkout so any
benchmark output remains attributable.

## Published benchmark docs

The committed performance tables and charts are generated by
`.github/workflows/benchmark-docs.yml` on `blacksmith-32vcpu-ubuntu-2404`. Trigger
it manually when the published snapshot needs a refresh:

```bash
gh workflow run benchmark-docs.yml --ref main -f runs=7
```

The workflow opens a PR with updated Markdown tables and SVG charts. The shared
local command is:

```bash
OX_CONTENT_BENCHMARK_RUNS=7 vp run bench:docs
```

Pull requests use the same Blacksmith runner class for base/head runtime and
bundle comparisons. The comment includes the regression gate, a head-commit
competitive snapshot, and the captured runner/runtime metadata so result drift
can be traced back to the environment.
