# Performance

This page tracks benchmark results, bundle-size checks, and the optimizations
that affect shipped output. Architecture and package pages should describe
boundaries and APIs; measured performance belongs here.

For allocation and span-level investigation while developing parser or renderer
changes, use [Profiling Mode](./profiling.md). Profiling answers "where is the
work happening?" Benchmarking answers "how fast is this workload?"

## What We Measure

Ox Content has two performance surfaces:

- Runtime throughput for Markdown parsing and rendering.
- Static output weight for generated sites, including gzip size and initial
  request count.

Runtime matters for CLIs, dev servers, editor integrations, and batch builds.
Output weight matters for documentation sites because generated HTML, CSS, and
JS are what users fetch on every navigation.

## Runtime Snapshot

Ox Content is positioned both as a document generator and as a high-performance
Markdown toolkit. The numbers below focus on the Markdown engine side.

Latest local runtime sweep on 2026-05-25 with Node `v24.15.0` on Apple M5 Pro.
The tables show median results from 7 local runs of the benchmark harness for
the large 48.7 KB case.

### Parse Only (48.7 KB)

| Library            | ops/sec | avg time |  throughput |
| ------------------ | ------: | -------: | ----------: |
| `@ox-content/napi` |    4207 |  0.24 ms | 200.20 MB/s |
| `md4x (napi)`      |    1231 |  0.81 ms |  58.56 MB/s |
| `md4w (md4c)`      |    1143 |  0.87 ms |  54.41 MB/s |
| `markdown-it`      |    1035 |  0.97 ms |  49.24 MB/s |
| `marked`           |     530 |  1.89 ms |  25.23 MB/s |
| `remark`           |      44 | 22.74 ms |   2.09 MB/s |

### Parse + Render (48.7 KB)

| Library             | ops/sec | avg time |  throughput |
| ------------------- | ------: | -------: | ----------: |
| `@ox-content/napi`  |    4503 |  0.22 ms | 214.26 MB/s |
| `Bun.markdown.html` |    4225 |  0.24 ms | 201.06 MB/s |
| `md4x (napi)`       |    4014 |  0.25 ms | 191.02 MB/s |
| `md4w (md4c)`       |    2653 |  0.38 ms | 126.23 MB/s |
| `markdown-it`       |     840 |  1.19 ms |  39.96 MB/s |
| `marked`            |     470 |  2.13 ms |  22.36 MB/s |
| `micromark`         |      45 | 22.35 ms |   2.13 MB/s |
| `remark`            |      36 | 28.16 ms |   1.69 MB/s |

In this latest local release-build sweep, Ox Content leads every comparison:
3.4x ahead of the next-fastest native parser (`md4x (napi)`) on parse-only and
1.07x ahead of `Bun.markdown.html` on parse+render, while remaining the native
core that drives the full documentation pipeline.

The runtime sweep covers more than the two tables above. The harness also runs
small and medium Markdown inputs, and it includes an async parse+render target
for the N-API package so that PR checks can catch overhead regressions in the
JavaScript boundary.

## Bundle Size

The bundle-size benchmark builds representative docs applications and measures
the generated production output. It reports:

- Total bytes across the generated output directory.
- Gzipped bytes for JS, CSS, HTML, and JSON assets.
- File count in the output directory.
- Estimated initial requests for `index.html` and local assets referenced from
  HTML or CSS.

Latest local bundle-size sweep on 2026-05-25 with Node `v24.15.0` on Apple M5
Pro. The table lists successful production builds from the local sweep.

| App                 |    Total |  Gzipped | Ratio   | Requests | Files |
| ------------------- | -------: | -------: | ------- | -------: | ----: |
| `ox-content (bare)` |  20.6 KB |   5.8 KB | 1.00x   |        1 |     5 |
| `ox-content`        | 109.1 KB |  25.2 KB | 4.33x   |        4 |     9 |
| `VitePress (bare)`  | 155.0 KB |  46.9 KB | 8.06x   |        6 |    14 |
| `ox-content + Vue`  | 167.6 KB |  47.5 KB | 8.16x   |        4 |     9 |
| `VitePress`         | 972.4 KB | 717.2 KB | 123.36x |       21 |    29 |

`ox-content (bare)` is the no-JS baseline. `ox-content` includes the built-in
docs shell. `ox-content + Vue` adds framework island support. VitePress rows use
the same benchmark content so the comparison is focused on generated output
shape rather than authoring content.

Bundle-size comparisons are intentionally reported beside request count. A
smaller gzip number is not always better if it creates too many blocking
requests; a larger shared chunk can be preferable when it removes repeated bytes
from every generated page and is cacheable across navigation.

## Chunk Optimization

The SSG pipeline first renders complete HTML pages, then extracts shared assets
after all pages are known. The TypeScript plugin calls the Rust-backed
`externalizeSsgAssets` implementation, which rewrites each generated page and
writes hashed assets under `assets/`.

The optimizer keeps the rules conservative:

- Identical CSS and JS content is deduplicated by content, then emitted once
  with a content hash in the filename.
- Core CSS sections such as base and footer styles are linked as a shared
  `ox-content-core-*.css` file.
- Theme CSS stays inline when it is small or when it contains relative
  `url(...)` references, so path resolution does not change.
- Search payload code is split from the main boot script into a separate
  `ox-content-search-*.js` chunk when the generated script contains the search
  placeholder.
- Generated scripts are emitted with `defer`, and public asset paths honor the
  configured site `base`.

The goal is not to split every feature into a separate file. The docs shell is
small enough that excessive chunking can increase startup work. The current
policy favors stable shared chunks for repeated bytes and avoids moving content
when doing so would create path-resolution risk or extra request overhead.

## PR Regression Gate

Pull requests run the benchmark workflow against both the base commit and the
head commit. The workflow posts one report with runtime and bundle-size sections.

Runtime rows compare the large benchmark target for `@ox-content/napi` and
`@ox-content/napi (async)`. Changes within +/-5% are treated as noise, and the
check fails when head throughput is more than 10% slower than base.

Bundle rows compare gzipped output for each successful benchmark app. The check
fails when gzipped size grows by more than 5%. Maintainers can intentionally
accept a regression with the `benchmark-regression-accepted` PR label.

## Reproduce

Run the JavaScript benchmark harness from the repository root:

```bash
node benchmarks/bundle-size/parse-benchmark.mjs
```

The benchmark includes `md4w (md4c)` and `md4x (napi)` by default and adds
`Bun.markdown.html` automatically when `bun` is available.

Run the bundle-size benchmark from the repository root:

```bash
node benchmarks/bundle-size/measure.mjs
```

For a faster local rerun after dependencies are installed, use:

```bash
node benchmarks/bundle-size/measure.mjs --skip-install
```

For Rust-side parser benchmarks, use:

```bash
cargo bench -p ox_content_parser
```

For real-world Markdown corpus benchmarks, populate the optional corpus first:

```bash
node scripts/fetch-bench-corpus.mjs
cargo bench -p ox_content_parser --bench corpus
```

For N-API transfer-format micro-benchmarks, see
[@ox-content/napi](./packages/napi.md#mdast-transfer-micro-benchmark).
