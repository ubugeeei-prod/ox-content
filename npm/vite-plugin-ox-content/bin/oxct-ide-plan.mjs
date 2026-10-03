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
    plans.push(await planJsonEdit(settings, [[["files.associations", "*.mdc"], "markdown"]]));
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
    if (options.config)
      plans.push(
        await planJsonEdit(join(root, ".zed/settings.json"), [
          [
            ["file_types", "Markdown"],
            ["md", "markdown", "mdc", "mdx"],
          ],
        ]),
      );
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
    const content = `-- Ox Content project setup (Neovim 0.11+). Source with :luafile .ox-content/neovim.lua\nvim.lsp.config("ox_content", {\n  cmd = { "vpx", "oxct", "lsp" },\n  filetypes = { "markdown", "mdx" },\n  root_markers = { "vite.config.ts", "vite.config.mjs", ".git" },\n})\nvim.lsp.enable("ox_content")\n`;
    if (original !== undefined && original !== content)
      throw new Error(`Preserving existing ${file}. Move it before generating a replacement.`);
    plans.push({ file, original, content });
  }
  return plans;
}
