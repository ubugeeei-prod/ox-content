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
        "textlintCommand": "pnpm exec textlint"
      }
    }
  }
}
```

Once `.mdc` is associated with `Markdown`, you get Zed's native Markdown preview/highlighting together with Ox Content frontmatter completion and diagnostics, plus i18n key intelligence in JS/TS.

`textlintEnabled` is optional and off by default. When enabled, the
language server runs textlint on save and publishes diagnostics with
`source: "textlint"`; omit `textlintCommand` to use `npx textlint`.
