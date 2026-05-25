# Benchmarks

Two flavors of performance measurement live in this tree:

- **JS comparison sweep** in `bundle-size/` — runs Ox Content alongside
  `markdown-it`, `marked`, `md4w`, `md4x`, and friends to compare ops/sec
  and throughput at small / medium / large input sizes. Trigger via
  `node benchmarks/bundle-size/parse-benchmark.mjs` (see
  [`bundle-size/README` flow in the top-level README](../README.md#performance)).
- **Rust criterion suites** under `crates/ox_content_parser/benches/` —
  in-process measurements that avoid NAPI overhead. Trigger via
  `cargo bench -p ox_content_parser`.

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

| project              | source                                                                        | license            |
| -------------------- | ----------------------------------------------------------------------------- | ------------------ |
| `vue-docs`           | [vuejs/docs](https://github.com/vuejs/docs)                                   | MIT                |
| `vite-docs`          | [vitejs/vite](https://github.com/vitejs/vite) (`docs/`)                       | MIT                |
| `rust-book`          | [rust-lang/book](https://github.com/rust-lang/book)                           | MIT OR Apache-2.0  |
| `typescript-handbook`| [microsoft/TypeScript-Website](https://github.com/microsoft/TypeScript-Website) (`packages/documentation/copy/en`) | MIT |

After fetching, run:

```bash
cargo bench -p ox_content_parser --bench corpus
```

The benchmark gracefully no-ops when the corpus directory is empty so a
fresh checkout can run `cargo bench` without first downloading anything.

Each upstream LICENSE file is included in the sparse checkout so any
benchmark output remains attributable.
