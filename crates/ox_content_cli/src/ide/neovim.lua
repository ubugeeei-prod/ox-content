-- Ox Content project setup (Neovim 0.11+). Source with :luafile .ox-content/neovim.lua
local ok, plugin = pcall(require, "ox-content")
if ok then
  plugin.setup({ cmd = { "vpx", "oxct", "lsp", "--project" } })
else
  vim.filetype.add({ extension = { mdc = "markdown" } })
  vim.lsp.config("ox-content-lsp", {
    cmd = { "vpx", "oxct", "lsp", "--project" },
    filetypes = { "markdown", "mdx" },
    root_markers = { "vite.config.ts", "vite.config.mjs", "package.json", ".git" },
  })
  vim.lsp.enable("ox-content-lsp")
end
