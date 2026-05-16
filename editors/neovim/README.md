# Ox Content for Neovim

Neovim integration for Ox Content authoring and i18n workflows.

Example with `lazy.nvim`:

```lua
{
  dir = "/absolute/path/to/ox-content/editors/neovim",
  config = function()
    require("ox-content").setup({
      frontmatter_schema = "./content/frontmatter.schema.json",
    })
  end,
}
```

Commands:

- `:OxContentInsertTable`
- `:OxContentInsertCodeFence`
- `:OxContentInsertCallout`
- `:OxContentPreview`

The plugin maps `*.mdc` to `markdown`, starts `ox-content-lsp` for Markdown and JS/TS/JSON/YAML buffers, and uses the same server for insertion commands, preview rendering, and i18n key intelligence.
