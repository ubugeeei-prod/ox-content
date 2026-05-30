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

Latest local runtime sweep on 2026-05-30 with Node `v24.16.0` on an Apple M2
Max. The tables show median results from 7 local runs of the benchmark harness,
covering the 48.7 KB "large" case and a ~1 MB "huge" case. Absolute ops/sec
track the host (earlier sweeps on faster hardware report higher numbers), but
the relative ordering between engines is stable.

### Parse Only (48.7 KB)

| Library            | ops/sec | avg time |  throughput |
| ------------------ | ------: | -------: | ----------: |
| `@ox-content/napi` |    3272 |  0.31 ms | 155.72 MB/s |
| `satteri`          |    1682 |  0.59 ms |  80.06 MB/s |
| `md4w (md4c)`      |     670 |  1.49 ms |  31.89 MB/s |
| `md4x (napi)`      |     623 |  1.60 ms |  29.66 MB/s |
| `markdown-it`      |     596 |  1.68 ms |  28.37 MB/s |
| `marked`           |     386 |  2.59 ms |  18.38 MB/s |
| `@mizchi/markdown` |      41 | 24.19 ms |   1.97 MB/s |
| `remark`           |      27 | 36.56 ms |   1.30 MB/s |

### Parse + Render (48.7 KB)

| Library            | ops/sec | avg time |  throughput |
| ------------------ | ------: | -------: | ----------: |
| `@ox-content/napi` |    3738 |  0.27 ms | 177.90 MB/s |
| `md4x (napi)`      |    2270 |  0.44 ms | 108.03 MB/s |
| `md4w (md4c)`      |    1737 |  0.58 ms |  82.67 MB/s |
| `satteri`          |    1043 |  0.96 ms |  49.63 MB/s |
| `markdown-it`      |     481 |  2.08 ms |  22.88 MB/s |
| `marked`           |     327 |  3.06 ms |  15.57 MB/s |
| `@mizchi/markdown` |     237 |  4.21 ms |  11.29 MB/s |
| `micromark`        |      29 | 34.76 ms |   1.37 MB/s |
| `remark`           |      24 | 41.92 ms |   1.14 MB/s |

On the 48.7 KB document Ox Content leads every comparison: 1.95x ahead of the
next-fastest parser (`satteri`) on parse-only and 1.65x ahead of `md4x (napi)`
on parse+render, while remaining the native core that drives the full
documentation pipeline.

### Parse Only (~1 MB)

| Library            | ops/sec |   avg time |  throughput |
| ------------------ | ------: | ---------: | ----------: |
| `@ox-content/napi` |     144 |    6.92 ms | 147.83 MB/s |
| `satteri`          |      54 |   18.66 ms |  54.83 MB/s |
| `md4w (md4c)`      |      32 |   31.04 ms |  32.97 MB/s |
| `md4x (napi)`      |      27 |   36.37 ms |  28.13 MB/s |
| `markdown-it`      |      21 |   48.49 ms |  21.10 MB/s |
| `marked`           |      16 |   63.16 ms |  16.20 MB/s |
| `@mizchi/markdown` |       1 |  836.85 ms |   1.22 MB/s |
| `remark`           |       1 | 1839.10 ms |   0.56 MB/s |

### Parse + Render (~1 MB)

| Library            | ops/sec |   avg time |  throughput |
| ------------------ | ------: | ---------: | ----------: |
| `@ox-content/napi` |     164 |    6.08 ms | 168.24 MB/s |
| `md4x (napi)`      |     103 |    9.71 ms | 105.41 MB/s |
| `md4w (md4c)`      |      79 |   12.59 ms |  81.24 MB/s |
| `satteri`          |      37 |   26.70 ms |  38.32 MB/s |
| `markdown-it`      |      15 |   67.75 ms |  15.10 MB/s |
| `marked`           |      14 |   73.27 ms |  13.96 MB/s |
| `@mizchi/markdown` |      10 |   99.77 ms |  10.25 MB/s |
| `micromark`        |       1 |  817.55 ms |   1.25 MB/s |
| `remark`           |       1 | 1981.80 ms |   0.52 MB/s |

The ~1 MB case stresses the engines at single-file-handbook scale. Ox Content
holds its lead (2.7x ahead of `satteri` on parse-only, 1.6x ahead of
`md4x (napi)` on parse+render) and sustains ~148–168 MB/s throughput, while the
incremental CST parser (`@mizchi/markdown`, tuned for real-time editing rather
than bulk parsing) and the `unified`/`remark` and `micromark` pipelines fall to
~1 op/sec — two to three orders of magnitude behind.

The runtime sweep covers more than the tables above. The harness also runs small
and medium Markdown inputs, an async parse+render target for the N-API package
(so PR checks can catch JavaScript-boundary overhead regressions), and an
optional `Bun.markdown` comparison when the harness is run under Bun.

## Bundle Size

The bundle-size benchmark builds representative docs applications and measures
the generated production output. It reports:

- Total bytes across the generated output directory.
- Gzipped bytes for JS, CSS, HTML, and JSON assets.
- File count in the output directory.
- Estimated initial requests for `index.html` and local assets referenced from
  HTML or CSS.

Latest local bundle-size sweep on 2026-05-28 with Node `v24.16.0` on Apple M5
Pro. The table lists successful production builds from the local sweep.

| App                 |    Total |  Gzipped | Ratio   | Requests | Files |
| ------------------- | -------: | -------: | ------- | -------: | ----: |
| `ox-content (bare)` |  20.6 KB |   5.8 KB | 1.00x   |        1 |     5 |
| `ox-content`        | 111.1 KB |  25.6 KB | 4.41x   |        4 |     9 |
| `VitePress (bare)`  | 155.0 KB |  46.9 KB | 8.05x   |        6 |    14 |
| `ox-content + Vue`  | 169.6 KB |  47.9 KB | 8.23x   |        4 |     9 |
| `VitePress`         | 972.4 KB | 717.2 KB | 123.18x |       21 |    29 |

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
