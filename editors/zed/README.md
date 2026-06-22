# Ox Content for Zed

This extension wires Zed Markdown, JavaScript, TypeScript, JSON, and YAML buffers to `ox-content-lsp`.

Recommended Zed settings:

```json
{
  "file_types": {
    "Markdown": ["md", "markdown", "mdc"]
  },
  "lsp": {
    "ox-content-lsp": {
      "binary": {
        "path": "/absolute/path/to/ox-content-lsp"
      },
      "initialization_options": {
        "frontmatterSchema": "./content/frontmatter.schema.json",
        "textlintEnabled": true,
        "textlintCommand": "pnpm exec textlint",
        "spaceBetweenHalfAndFullWidth": "forbid",
        "spacingAutoFixOnSave": false
      }
    }
  }
}
```

Once `.mdc` is associated with `Markdown`, you get Zed's native Markdown
preview/highlighting together with Ox Content frontmatter completion and
diagnostics, including built-in `meta` fields when no custom schema is
configured. JS/TS files get i18n key completion, hover previews, inlay hints,
diagnostics, go-to-definition, and document links into dictionary files.

`textlintEnabled` is optional and off by default. When enabled, the
language server runs textlint on save and publishes diagnostics with
`source: "textlint"`; omit `textlintCommand` to use `npx textlint`.
If textlint reports a JSON fix, it is exposed as a quick fix.

`spaceBetweenHalfAndFullWidth` accepts `forbid` (default), `require`, or
`off`. Use `spacingAutoFixOnSave: true` only when you want the LSP to return
spacing edits from `willSaveWaitUntil`; alternatively, use Zed's standard
format-on-save support with the LSP formatting provider.

Publishing follows Zed's registry flow: release this repository, then update
the `ox-content` submodule entry in `zed-industries/extensions` with
`path = "editors/zed"` and the same version as `extension.toml`.
