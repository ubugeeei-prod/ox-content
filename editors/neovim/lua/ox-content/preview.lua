local client = require("ox-content.client")
local config = require("ox-content.config")
local util = require("ox-content.util")

local M = {}

local function open_path(path)
  if vim.ui and vim.ui.open then
    vim.ui.open(path)
    return
  end

  local command
  if vim.fn.has("mac") == 1 then
    command = { "open", path }
  elseif util.is_windows() then
    command = { "cmd", "/c", "start", "", path }
  else
    command = { "xdg-open", path }
  end

  vim.fn.jobstart(command, { detach = true })
end

function M.open()
  local bufnr = vim.api.nvim_get_current_buf()
  client.request_command(bufnr, config.commands.preview_html, {
    client.current_buffer_uri(bufnr),
  }, function(result)
    if type(result) ~= "table" or type(result.html) ~= "string" then
      util.notify("Preview payload was empty", vim.log.levels.ERROR)
      return
    end

    local file = vim.fn.tempname() .. ".html"
    vim.fn.writefile(vim.split(result.html, "\n", { plain = true }), file)
    open_path(file)
  end)
end

return M
