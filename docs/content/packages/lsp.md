---
title: Checker and Language Server
description: Shared diagnostics between ox-content-lsp and the CLI checkers, including capabilities and known gaps.
---

# Checker and Language Server

`ox-content-lsp` and the CLI checkers share the same diagnostic cases for
Markdown, MDC, and local links. Editors receive those findings through
`textDocument/publishDiagnostics`. CI can run the same codes, ranges, and
messages from the checker binaries.

## Shared diagnostic cases

| Source               | CLI                            | Codes                                                                                   |
| -------------------- | ------------------------------ | --------------------------------------------------------------------------------------- |
| `ox-content-mdc`     | `ox-content-mdc-check`         | `mdc-unquoted-prop`, `mdc-mismatched-tag`, `mdc-orphan-close`, `mdc-unclosed-tag`       |
| `ox-content-link`    | `ox-content-link-check`        | `link-missing-file`, `link-missing-anchor`, `link-cross-file-anchor`, `link-unresolved` |
| `ox-content`         | none (frontmatter is LSP-only) | `frontmatter-unknown`, `frontmatter-type`, `frontmatter-enum`, `frontmatter-required`   |
| `ox-content-spacing` | none (spacing is LSP-only)     | `space-between-half-and-full-width`, `require-space-between-half-and-full-width`        |
| `textlint`           | configured `textlint` command  | rule ids from the sidecar                                                               |

MDC CLI checks skip YAML frontmatter so tag diagnostics line up with the
language server. Link checks run on the full document in both surfaces.

```bash
ox-content-mdc-check --format json docs/page.mdc
ox-content-link-check --format json docs/page.md
cargo run -p ox_content_lsp --bin ox-content-lsp
```

## Language server behavior

- Incremental `textDocument/didChange` applies only the edited range, then
  recomputes the slices that changed. A body-only edit reuses cached
  frontmatter diagnostics.
- Each publish carries the document version. A later edit cancels the
  in-flight job; stale or duplicate results are not published.
- textlint stays opt-in and runs on save. Markdown/MDC/link diagnostics
  still refresh on every change.

## Known gaps

- External HTTP links are not fetched. The link checker is offline-only.
- Source-side cross-file anchors (`./other.md#section`) emit
  `link-cross-file-anchor` warnings. The generated-site pass
  (`--site-dir`) validates them after the build.
- Reference-style links (`[ok][ref]`) are not expanded by the parser yet.
- Frontmatter schema and half/full-width spacing have no CLI counterpart.
- Code-block lint, `tsgo` typecheck, and docs-as-tests stay in the Vite
  transform. They are not LSP diagnostics.
- MDX parse errors are reported for `.mdx` files; expression typechecking
  is not.
- i18n diagnostics publish only on JavaScript/TypeScript source files, not
  mixed into Markdown gutters.

See also the [editor extension roadmap](../editor-extension-roadmap.md) and
[architecture](../architecture.md).
