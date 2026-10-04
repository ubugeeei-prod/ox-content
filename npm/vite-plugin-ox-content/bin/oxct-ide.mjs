import { runNativeCli } from "./oxct-native.mjs";

export function runIde(args) {
  if (!args.length || args.includes("--help") || args.includes("-h")) {
    console.log(`oxct ide install

Select IDEs, extension installation, and workspace configuration.
Existing JSONC comments and unrelated settings are preserved; changed files are backed up.
Workspace configuration enables Standard Schema validation by executing trusted project
Vite configuration. VS Code requires Workspace Trust; Zed/Neovim use explicit --project.

Options:
  --ide vscode|cursor|windsurf|vscodium|zed|neovim (repeatable)
  --config-only          Only create workspace configuration
  --extensions-only      Only install extensions
  --dry-run              Show the exact plan without writes or installations
  --yes                  Use both actions without prompting
  -h, --help             Show this help

Zed installation enables user-level auto_install_extensions; Zed installs it on next launch.
Neovim installs the bundled plugin in your native pack directory and generates a project Lua config.`);
    return;
  }
  runNativeCli("ide", args);
}
