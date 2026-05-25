# Performance

This page tracks benchmark results and reproduction commands for Ox Content.
Architecture and package pages should describe boundaries and APIs; measured
performance belongs here.

For allocation and span-level investigation while developing parser or renderer
changes, use [Profiling Mode](./profiling.md). Profiling answers "where is the
work happening?" Benchmarking answers "how fast is this workload?"

## Latest Benchmark Snapshot

Ox Content is positioned both as a document generator and as a high-performance
Markdown toolkit. The numbers below focus on the Markdown engine side.

Latest local benchmark sweep on 2026-05-25 with Node `v24.15.0` on Apple M5 Pro.
The tables below show median results from 7 local runs of the benchmark harness
for the large 48.7 KB case.

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

## Reproduce

Run the JavaScript benchmark harness from the repository root:

```bash
node benchmarks/bundle-size/parse-benchmark.mjs
```

The benchmark includes `md4w (md4c)` and `md4x (napi)` by default and adds
`Bun.markdown.html` automatically when `bun` is available.

For Rust-side parser benchmarks, use:

```bash
cargo bench -p ox_content_parser
```

For N-API transfer-format micro-benchmarks, see
[@ox-content/napi](./packages/napi.md#mdast-transfer-micro-benchmark).
