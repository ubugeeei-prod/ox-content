local config = require("ox-content.config")
local util = require("ox-content.util")

local M = {}

local function resolve_root(bufnr)
  local name = vim.api.nvim_buf_get_name(bufnr)
  if name == "" then
    return vim.fn.getcwd()
  end

  local markers = { ".ox-content.json", "ox-content.json", "package.json", ".git" }
  local root = vim.fs.root(name, markers)
  return root or vim.fs.dirname(name) or vim.fn.getcwd()
end

local function server_binary_name()
  if util.is_windows() then
    return "ox-content-lsp.exe"
  end
  return "ox-content-lsp"
end

local function resolve_cmd(bufnr)
  local user_cmd = config.get().cmd
  if type(user_cmd) == "table" and #user_cmd > 0 then
    return vim.deepcopy(user_cmd)
  end

  if type(user_cmd) == "string" and user_cmd ~= "" then
    return { user_cmd }
  end

  local executable = vim.fn.exepath(server_binary_name())
  if executable ~= "" then
    return { executable }
  end

  local root = resolve_root(bufnr)
  local binary_name = server_binary_name()
  local local_binaries = {
    vim.fs.joinpath(root, "target", "debug", binary_name),
    vim.fs.joinpath(root, "target", "release", binary_name),
  }

  for _, candidate in ipairs(local_binaries) do
    if vim.uv.fs_stat(candidate) then
      return { candidate }
    end
  end

  return {
    "cargo",
    "run",
    "-p",
    "ox_content_lsp",
    "--bin",
    "ox-content-lsp",
  }
end

local function resolve_init_options(bufnr)
  local schema = config.get().frontmatter_schema
  if type(schema) ~= "string" or schema == "" then
    return {}
  end

  if vim.startswith(schema, "/") or schema:match("^%a:[/\\]") then
    return { frontmatterSchema = schema }
  end

  return { frontmatterSchema = vim.fs.joinpath(resolve_root(bufnr), schema) }
end

local function position_from_cursor()
  local row, col = unpack(vim.api.nvim_win_get_cursor(0))
  local line = vim.api.nvim_get_current_line()
  return { line = row - 1, character = vim.str_utfindex(line, col) }
end

function M.current_buffer_uri(bufnr)
  return vim.uri_from_bufnr(bufnr)
end

function M.get(bufnr)
  local clients = vim.lsp.get_clients({ bufnr = bufnr, name = config.server_name })
  return clients[1]
end

function M.start(bufnr)
  if M.get(bufnr) then
    return
  end

  local client_id = vim.lsp.start({
    name = config.server_name,
    cmd = resolve_cmd(bufnr),
    root_dir = resolve_root(bufnr),
    init_options = resolve_init_options(bufnr),
  }, { bufnr = bufnr })

  if client_id and vim.lsp.completion and vim.lsp.completion.enable then
    pcall(vim.lsp.completion.enable, true, client_id, bufnr, { autotrigger = true })
  end
end

function M.request_command(bufnr, command, arguments, callback)
  local client = M.get(bufnr)
  if not client then
    M.start(bufnr)
    client = M.get(bufnr)
  end

  if not client then
    util.notify("Ox Content language server is not running", vim.log.levels.ERROR)
    return
  end

  client:request("workspace/executeCommand", {
    command = command,
    arguments = arguments,
  }, function(err, result)
    if err then
      util.notify(err.message or ("Command failed: " .. command), vim.log.levels.ERROR)
      return
    end

    if callback then
      callback(result)
    end
  end, bufnr)
end

function M.insert(command)
  local bufnr = vim.api.nvim_get_current_buf()
  M.request_command(bufnr, command, {
    {
      uri = M.current_buffer_uri(bufnr),
      position = position_from_cursor(),
    },
  })
end

return M
