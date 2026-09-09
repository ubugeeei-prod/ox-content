# Downstream extension benchmarks

The native Markdown matrix lives in `native-extensions.md`. This companion covers the post-render pipeline: every native media provider, tabs, package-manager tabs, YouTube, syntax highlighting, cross references, citations (including local bibliography reads), code lint, and docs-test extraction. It tests 1, 32 and 256 occurrences plus enabled-but-absent controls: 144 rows. A registry assertion prevents silently omitting new providers. Each fixture must exercise its transform and retain stable output.

From the repository root, with workspace dependencies and the matching NAPI binding built:

```sh
node tools/benchmarks/build-pipeline-extensions.mjs
cd npm/vite-plugin-ox-content
OX_MEDIA_FIXTURES=../../tools/benchmarks/extension-media.json \
OX_BIBLIOGRAPHY=../../tools/benchmarks/extension-bibliography.json \
node ../../tools/benchmarks/pipeline-extensions.mjs .pipeline-extensions.mjs > results.jsonl
```

`OX_EXTENSION_FILTER` restricts workload names. Keep the generated bundle local. Use the same Node, dependency versions and NAPI build profile for comparisons. Each row records input bytes, nine samples after two warmups, median milliseconds and an output SHA-256. Five calls form each sample; output verification happens outside the timed interval. Save both revisions and runner metadata alongside results. Run A/B/B/A without competing builds, and retain unchanged controls. Timing is observational, not a wall-clock unit-test threshold.

Media measurements are native HTML transformations with supplied metadata; they perform no HTTP requests. Live GitHub/OpenGraph/Reddit/provider latency, cold Mermaid/Graphviz renderer execution, TypeScript compiler startup and docs-test execution are separate costs. Existing provider request caches and renderer caches must be measured explicitly as cold or warm. Do not treat an unavailable optional renderer as a successful fast render. `speakerdeck-oembed.mjs` separately measures request deduplication against a controlled local HTTP server.

KaTeX, BudouX and static-diagram protection have their own focused drivers (`katex-formulas.mjs`, `budoux-protected.mjs`, `static-diagrams.mjs`). End-to-end build speed cannot be inferred by multiplying these isolated speedups.
