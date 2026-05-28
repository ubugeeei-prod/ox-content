# ox_content_link_checker

Dead-link checker for Ox Content Markdown. Offline-only by design: it
resolves every link / image / image-link target against the filesystem
and against the document's own heading slugs, but it never opens a
network connection. The same binary is safe in CI without timeouts,
retries, or rate limits, and produces deterministic output across runs.

## Library

```rust
use ox_content_link_checker::{check_source, CheckOptions};
use std::path::PathBuf;

let opts = CheckOptions::for_file(PathBuf::from("/path/to/doc.md"));
for diagnostic in check_source(source, &opts) {
    eprintln!("{}:{} {}", diagnostic.line, diagnostic.column, diagnostic.message);
}
```

Knobs:

- `CheckOptions::src_dir` — base for paths that start with `/`.
  Defaults to the document's own directory; set this to the workspace
  root for the same resolution rules as Vite-served static files.
- `CheckOptions::ignore_patterns` — substring patterns. Any link whose
  raw target contains any of them is skipped.

## CLI

```bash
ox-content-link-check docs/**/*.md
ox-content-link-check --src-dir docs --format json docs/index.md
ox-content-link-check --ignore "intentionally-broken" docs/**/*.md
```

Exit code is `1` when any error-severity diagnostic was emitted (or any
file failed to read). Warning-severity diagnostics never fail the run
on their own; they show up in the output.

## What it checks

| Link form                                | Behavior                                                                                                 |
| ---------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| `[text](https://…)` / `[text](http://…)` | passed through, never asserted                                                                           |
| `[text](mailto:…)` and other URL schemes | passed through                                                                                           |
| `[text](#anchor)`                        | `#anchor` must match a heading slug in the document                                                      |
| `[text](./other.md)`                     | `./other.md` must exist relative to the document                                                         |
| `[text](/abs/leaf.md)`                   | resolved under `src_dir` (or the document's directory)                                                   |
| `[text](./other.md#section)`             | file existence is checked; the anchor is flagged with a warning until cross-file anchor resolution lands |
| `![alt](./image.png)`                    | same resolution as inline links                                                                          |

## Limitations (tracked follow-ups)

- Reference-link / shortcut-link forms (`[ok][ref]` plus `[ref]: …`) are
  not yet expanded by the parser, so the checker can't see them. A
  future parser change that emits `Link` nodes for resolved references
  will surface here automatically.
- Cross-file anchor validation is intentionally deferred — it requires
  parsing the other document and slugifying its headings, which doubles
  the I/O fan-out. The warning lets the user know the file half passed
  while keeping that work out of the per-keystroke LSP hot path.
- HTTP link checking is not implemented. A `http-check` feature flag
  with a deterministic, opt-in head-check pool is on the roadmap.
