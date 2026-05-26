# VS Code Ox Content

VS Code support for Ox Content authoring and i18n workflows.

Features:

- Ox Content LSP client
- frontmatter schema completion and diagnostics
- i18n key completion, hover, definition, diagnostics, and inlay hints for JS/TS
- command palette actions for table/code fence/callout insertion
- Ox Content HTML preview with **HMR**: opening the preview subscribes
  the document with the LSP, and the panel reloads on every
  `oxContent/previewDidChange` notification (no client-side polling)
- `.mdc` files associated with Markdown and component tag diagnostics

## Configuration

| Setting                         | Type    | Description                                                                                                                   |
| ------------------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `oxContent.server.path`         | string  | Absolute or workspace-relative path to `ox-content-lsp`. Empty falls back to `target/debug` → `target/release` → `cargo run`. |
| `oxContent.frontmatter.schema`  | string  | Path to a frontmatter schema (Markdown + `.mdc`).                                                                             |
| `oxContent.preview.autoRefresh` | boolean | Re-render the preview as you type (default `true`).                                                                           |

The environment variable `OX_CONTENT_LSP_PATH` is honored as an override
between `oxContent.server.path` and the local-binary probe. CI and the
integration test runner use it so they can point at a freshly built
`target/release/ox-content-lsp` without writing per-workspace settings.

## Preview HMR

`oxContent.openPreview` opens a webview that subscribes to LSP push
updates instead of debouncing on `onDidChangeTextDocument`. The flow is:

1. The extension calls `oxContent.previewSubscribe` with the document
   URI; the LSP returns the initial rendered HTML.
2. Every text change to that document triggers
   `oxContent/previewDidChange` from the LSP, which the panel applies
   directly.
3. Disposing the webview (or closing the document) sends
   `oxContent.previewUnsubscribe`.

Set `oxContent.preview.autoRefresh: false` to opt out — the panel
still opens but stops at the initial render.

## Testing

The extension has two test surfaces.

**Unit tests** (pure node, no VS Code host):

```bash
pnpm --filter vscode-ox-content test:unit
# or, from the repo root:
vp run test:vscode-unit
```

These exercise the helpers extracted into `src/internal/` (path
resolution, preview HTML, command guards, snippet shape). They run on
every CI build and are part of `vp run test`.

**Integration tests** (real VS Code Electron host via `@vscode/test-cli`):

```bash
# Once, to produce the LSP the extension talks to:
vp run build:lsp

# Then run the integration suite (Linux needs xvfb-run prefix):
node scripts/run-vscode-tests.mjs
# or from the repo root:
vp run test:vscode
```

The driver script (`scripts/run-vscode-tests.mjs`) sets
`OX_CONTENT_LSP_PATH` to the absolute path of the freshly built
`target/release/ox-content-lsp` so the test workspace does not need a
`.vscode/settings.json`. Integration tests that depend on the LSP
responding self-skip when the binary is unavailable.

CI runs both jobs under `.github/workflows/ci.yml` (`vscode-test`).
