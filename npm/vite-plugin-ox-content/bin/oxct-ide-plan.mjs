import { join } from "node:path";
import { homedir } from "node:os";
import { planJsonEdit, readOptional, parseConfig } from "./oxct-config-files.mjs";

export const ideCommands = {
  vscode: "code",
  cursor: "cursor",
  windsurf: "windsurf",
  vscodium: "codium",
};
export const ideNames = [...Object.keys(ideCommands), "zed", "neovim"];

export async function planIdeSetup(
  options,
  root,
  configHome = process.env.XDG_CONFIG_HOME || join(homedir(), ".config"),
) {
  const plans = [];
  if (options.ides.some((ide) => ide in ideCommands) && options.config) {
    const settings = join(root, ".vscode/settings.json");
    plans.push(
      await planJsonEdit(settings, [
        [["files.associations", "*.mdc"], "markdown"],
        [["oxContent.frontmatter.projectValidation"], true],
      ]),
    );
    const extensions = join(root, ".vscode/extensions.json");
    const old = parseConfig(await readOptional(extensions));
    if (
      old.recommendations !== undefined &&
      (!Array.isArray(old.recommendations) ||
        old.recommendations.some((value) => typeof value !== "string"))
    )
      throw new Error(`Invalid recommendations in ${extensions}`);
    plans.push(
      await planJsonEdit(extensions, [
        [
          ["recommendations"],
          [...new Set([...(old.recommendations ?? []), "ubugeeei.vscode-ox-content"])],
        ],
      ]),
    );
  }
  if (options.ides.includes("zed")) {
    if (options.config) {
      const file = join(root, ".zed/settings.json");
      const existing = parseConfig(await readOptional(file)).file_types?.Markdown ?? [];
      if (!Array.isArray(existing) || existing.some((value) => typeof value !== "string"))
        throw new Error(`Invalid Markdown file types in ${file}`);
      plans.push(
        await planJsonEdit(file, [
          [["file_types", "Markdown"], [...new Set([...existing, "md", "markdown", "mdc", "mdx"])]],
          [["lsp", "ox-content-lsp", "binary", "path"], "vpx"],
          [
            ["lsp", "ox-content-lsp", "binary", "arguments"],
            ["oxct", "lsp", "--project"],
          ],
        ]),
      );
    }
    if (options.extensions) {
      const settings =
        process.platform === "win32"
          ? join(process.env.APPDATA ?? configHome, "Zed/settings.json")
          : join(configHome, "zed/settings.json");
      plans.push(await planJsonEdit(settings, [[["auto_install_extensions", "ox-content"], true]]));
    }
  }
  if (options.ides.includes("neovim") && options.config) {
    const file = join(root, ".ox-content/neovim.lua");
    const original = await readOptional(file);
    const content = `-- Ox Content project setup (Neovim 0.11+). Source with :luafile .ox-content/neovim.lua
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
`;
    if (original !== undefined && original !== content)
      throw new Error(`Preserving existing ${file}. Move it before generating a replacement.`);
    plans.push({ file, original, content });
  }
  return plans;
}
