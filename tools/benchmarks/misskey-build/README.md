# Misskey Hub build investigation

Source report: <https://github.com/odanado/misskey-hub-next/pull/1>.
Consumer head: `af92e493ce46d27c27600eda8924de05593e4c94`.
The existing instrumentation archives the unchanged islands implementation at
`6cbe059c5d3b358557f6493b606f196c2ca06e8a`. Ox base:
`a4a74fa3994298e719fe201ace32f234b5d9d5a2`.

Measured on an Apple M2 Max, Node 22.22.3, Vite 7.3.1, Vue 3.5.25.
Comparisons retain all 1,991 URLs and the original Vue/MDC/Shiki work. Reduced
themes and missing page bodies are not performance candidates here.

## Findings

- The unmodified published Ox 3.4.3 build completed in 30.09 seconds including
  PNPM/Node startup. Its verifier passed all 1,991 pages and local CSS/JS links.
- The existing instrumented runner measured 28.948 / 27.623 / 27.100 seconds.
  Its boundary starts before importing Vite and ends after the awaited build;
  startup and output verification are excluded. The first sample overlapped
  package preparation, so it is retained as a warmup, not independent evidence.
- One instrumented run spent 7.95 seconds preparing content/compatibility data,
  8.09 seconds rendering pages, and 3.73 seconds loading the host before data
  preparation. Sidebar Vue SSR alone occupied 3.83 seconds within page rendering.
  These figures describe this consumer's compatibility implementation.
- Replaying the 1,434 exact prepared Markdown inputs through `host.markdown.render`
  measured **227.74 ms before / 208.11 ms after**, a **8.6%** reduction in this
  isolated pipeline. Each side has two fresh processes, one full warmup and nine
  samples per process, ordered before/after/after/before. All output fields other
  than unused generated module code have the same SHA-256 on both sides.
- Full builds remain around 27 seconds once package imports are warm. The small
  total difference overlaps run-to-run variation. Do not infer a material whole
  site speedup from the isolated pipeline result or the slower first imports.

The changes disable file watching in the internal build-only Vite loader and
avoid another paragraph normalization when embed transforms left HTML unchanged.
The normal development server still uses its watcher. Across both full-build
series, all HTML and generated assets are byte-identical to each series' baseline;
all 750 original public files are also byte-identical. This verifies preservation
of the existing candidate output, not visual parity with the original Nuxt site.
The consumer's compatibility config already specifies `server.watch: null`, so
the watcher change is a general build-loader improvement; no watcher speedup is
claimed for this consumer. The resolved-config hook also handles consumer server
objects correctly: Vite skips nested null overrides during config merging.

## Reproduction

Install the consumer's pinned lockfile in an isolated worktree; existing local
dependencies may have been linked to unpublished Ox worktrees. Run `nuxt prepare`
outside the timer. Use the existing `islands-comparison/prepare.py` and runner to
record stage timings and output checks. Redirect their results to a new local
directory to preserve historical committed measurements.

For the focused replay, prepare a local JSON containing `root`, `options`, and
`documents: [{ documentPath, source, convertMdLinks }]`. Extract the frontmatter
with the pinned native binding and call the consumer's `prepareNativeMarkdown`
with `mdcIslands: true, allowUnclosedContainers: true`. Keep this corpus out of Git.
Match the consumer options: `embeds: false`, `highlight: false`, `containers: true`,
`docs: false`, `search: false`, and disable OG images, last-updated and reader chrome.

```sh
node tools/benchmarks/custom-host-markdown.mjs corpus.json /path/to/built/plugin 9
```

Use identical Node, dependencies and native binding builds for both plugin
snapshots. This run uses published native 3.5.0 on both sides and the consumer's
Vite 7.3.1. Run fresh processes sequentially with no competing builds. Hash outputs
and verify public assets outside timers. The replay excludes the consumer's
`renderHtml` callback, MDC/Shiki compatibility work, Vue SSR and Vite client build;
it measures the portion Ox owns. The JSON beside this file records all samples.

## Follow-up: deferred module generation

Custom Host Markdown callers usually consume HTML and metadata. Defer escaping
that HTML and constructing its JavaScript module until `transform.code` is read.
Snapshot mutable metadata before returning, so callback edits do not change what
that module would have exported. The ordinary Vite Markdown transform remains
an eager module producer.

With the same 1,434-input corpus, two fresh processes per side and nine samples
after one warmup, the HTML-only pipeline measured **208.90 → 193.59 ms (7.3%)**.
The mode that requests module code measured **209.64 → 212.77 ms**; sample ranges
overlap (204–233 ms), so this is an HTML-consumption optimization, not a claim
that every caller becomes faster. HTML/structured metadata match on both sides;
the requested generated modules are also byte-identical. Complete samples are
in `2026-10-31-deferred.json`.

The replay now avoids accessing `code` during outside-timer output checks. To
include module generation inside the timer and hash its output as well, run:

```sh
node tools/benchmarks/custom-host-markdown.mjs corpus.json /path/to/plugin 9 --module-code
```
