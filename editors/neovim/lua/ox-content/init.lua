local client = require("ox-content.client")
local config = require("ox-content.config")
local preview = require("ox-content.preview")

local M = {}
local augroup = nil
local commands_defined = false

local function define_commands()
  if commands_defined then
    return
  end

  vim.api.nvim_create_user_command("OxContentInsertTable", function()
    client.insert(config.commands.insert_table)
  end, { force = true })

  vim.api.nvim_create_user_command("OxContentInsertCodeFence", function()
    client.insert(config.commands.insert_code_fence)
  end, { force = true })

  vim.api.nvim_create_user_command("OxContentInsertCallout", function()
    client.insert(config.commands.insert_callout)
  end, { force = true })

  vim.api.nvim_create_user_command("OxContentPreview", function()
    preview.open()
  end, { force = true })

  commands_defined = true
end

function M.setup(opts)
  config.setup(opts)

  vim.filetype.add({
    extension = {
      mdc = "markdown",
    },
  })

  if augroup then
    vim.api.nvim_del_augroup_by_id(augroup)
  end

  augroup = vim.api.nvim_create_augroup("OxContentMarkdown", { clear = true })

  if config.get().auto_start then
    vim.api.nvim_create_autocmd("FileType", {
      group = augroup,
      pattern = {
        "markdown",
        "javascript",
        "javascriptreact",
        "typescript",
        "typescriptreact",
        "json",
        "yaml",
      },
      callback = function(args)
        client.start(args.buf)
      end,
    })
  end

  define_commands()
end

function M.start(bufnr)
  client.start(bufnr or vim.api.nvim_get_current_buf())
end

function M.preview()
  preview.open()
end

return M
