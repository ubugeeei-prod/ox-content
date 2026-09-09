# Native Markdown extension matrix

Run `cargo run --release -p ox_content_transform --example extensions` from the repository root. Pass a case-name substring after `--` to select a subset, such as `-- abbreviations`. Set `OX_EXTENSION_VERIFY=1` to validate the fixtures without timing.

The runner emits one JSON object per case and mode. All 49 cases have a syntax-present input and a fixed enabled-but-absent prose control. Every present fixture checks an extension-specific output marker, and all outputs must have no diagnostics. The same fixture validation is included in `cargo test -p ox_content_transform`.

Each row records input bytes, iterations, nine raw samples after ten warmup iterations, rendered HTML, frontmatter, table of contents, and component metadata. Times are milliseconds per operation:

- `preprocess_ms`: preprocessing with resolved feature options.
- `transform_ms`: full native transform with a reused transformer.
- `fresh_ms`: option resolution plus full native transform, as for one-shot callers.

Most present inputs repeat a small representative fixture 32 times. Frontmatter uses one document header. File includes, partials, code imports, and external data tables use the tracked assets next to the example; repeated reads measure a warm local filesystem. They do not measure network fetches.

The matrix covers GFM options, MDX, footnotes, superscript/subscript, punctuation, heading attributes/permalinks, source spans, code annotations, wiki links, emoji, math placeholders, attributes, badges, NotByAI, keyboard keys, definition lists, magic links, figures, sanitization, four abbreviation workloads, containers, cards, steps, code groups, galleries, timelines, conditionals, file trees, inline CSV/JSON, combined block options, includes, partials, code imports, external CSV, edit links, frontmatter, semantic footnotes, and renderer autolinks.

KaTeX rendering, syntax highlighters, diagram processes, citations, BudouX, media/provider enrichment, and framework generation run after or outside this native pipeline and need their own measurements. Native math and code-group rows end at their placeholders; they do not include KaTeX or the tab widget renderer.

For comparisons, use identical example/asset files and `Cargo.lock` in isolated base/head checkouts. Build both release executables before measuring, record the exact source SHAs, Rust/Node versions, CPU and OS, and run the saved binaries in ABBA order without concurrent builds or tests. Compare output fields exactly before interpreting medians. Keep raw samples and unchanged controls; these microbenchmarks do not establish whole-site build speedups.
