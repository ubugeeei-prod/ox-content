local M = {}

local defaults = {
  cmd = nil,
  frontmatter_schema = nil,
  auto_start = true,
}

local state = vim.deepcopy(defaults)

M.server_name = "ox-content-lsp"
M.commands = {
  insert_table = "oxContent.insertTable",
  insert_code_fence = "oxContent.insertCodeFence",
  insert_callout = "oxContent.insertCallout",
  preview_html = "oxContent.previewHtml",
}

function M.setup(opts)
  state = vim.tbl_deep_extend("force", {}, defaults, opts or {})
end

function M.get()
  return state
end

return M
