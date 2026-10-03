import { spawnSync } from "node:child_process";
import { cp, mkdir, readdir } from "node:fs/promises";
import { homedir } from "node:os";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { prompts, optionValue } from "./oxct-prompts.mjs";
import { writePlan } from "./oxct-config-files.mjs";
import { ideCommands, ideNames, planIdeSetup } from "./oxct-ide-plan.mjs";

export async function runIde(args) {
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
  const options = parseIdeOptions(args);
  let ui;
  if (!options.yes && process.stdin.isTTY && process.stdout.isTTY) {
    ui = await prompts();
    ui.intro("◆ Ox Content · IDE setup");
    if (!options.ides.length)
      options.ides = ui.checked(
        await ui.multiselect({
          message: "Choose your IDEs",
          options: ideNames.map((value) => ({ value, label: value })),
          required: true,
        }),
      );
    if (!options.actionSet) {
      const actions = ui.checked(
        await ui.multiselect({
          message: "What should be configured?",
          options: [
            {
              value: "extensions",
              label: "Install extensions",
              hint: "Includes user-level Zed / Neovim settings",
            },
            {
              value: "config",
              label: "Create workspace configuration",
              hint: "Enable trusted project schema validation",
            },
          ],
          initialValues: ["extensions", "config"],
          required: true,
        }),
      );
      options.extensions = actions.includes("extensions");
      options.config = actions.includes("config");
    }
  }
  if (!options.ides.length) throw new Error("Choose an IDE interactively or pass --ide <name>.");
  const plans = await planIdeSetup(options, process.cwd());
  const commands = options.extensions
    ? options.ides
        .filter((ide) => ide in ideCommands)
        .map((ide) => [ideCommands[ide], ["--install-extension", "ubugeeei.vscode-ox-content"]])
    : [];
  const nvimTarget = join(
    process.env.XDG_DATA_HOME || join(homedir(), ".local/share"),
    "nvim/site/pack/ox-content/start/ox-content",
  );
  const description = [
    ...(options.config
      ? ["Enable frontmatter validation from trusted project Vite configuration."]
      : []),
    ...plans.map(
      (plan) =>
        `${plan.original === plan.content ? "Keep" : plan.original === undefined ? "Create" : "Merge and back up"}: ${plan.file}`,
    ),
    ...commands.map(([command, args]) => `${command} ${args.join(" ")}`),
    ...(options.extensions && options.ides.includes("neovim")
      ? [`Install bundled Neovim plugin: ${nvimTarget}`]
      : []),
  ].join("\n");
  if (options.dryRun) {
    console.log(description);
    return;
  }
  if (ui) {
    ui.note(description, "Setup plan");
    if (!ui.checked(await ui.confirm({ message: "Apply this setup?", initialValue: true }))) {
      ui.cancel("No changes made.");
      return;
    }
  }
  for (const plan of plans) await writePlan(plan);
  const failures = [];
  for (const [command, args] of commands) {
    const executable = process.platform === "win32" ? `${command}.cmd` : command;
    const result = spawnSync(executable, args, { stdio: "inherit" });
    if (result.error || result.status !== 0)
      failures.push(
        `${command}: ${result.error?.message ?? `exit ${result.status}`}. Install the IDE CLI and retry.`,
      );
  }
  if (options.extensions && options.ides.includes("neovim")) {
    try {
      await readdir(nvimTarget)
        .then(() => {
          throw new Error(
            `Plugin already exists at ${nvimTarget}; preserving it. Update it through your plugin manager.`,
          );
        })
        .catch((error) => {
          if (error.code !== "ENOENT") throw error;
        });
      await mkdir(resolve(nvimTarget, ".."), { recursive: true });
      await cp(fileURLToPath(new URL("../dist/editors/neovim", import.meta.url)), nvimTarget, {
        recursive: true,
        errorOnExist: true,
        force: false,
      });
    } catch (error) {
      failures.push(error.message);
    }
  }
  const hints = [
    options.ides.includes("zed") && options.extensions
      ? "Zed: restart to apply extension auto-install (requires ox-content in the Zed registry)."
      : "",
    options.ides.includes("neovim") && options.config
      ? "Neovim: :luafile .ox-content/neovim.lua (Neovim 0.11+). The bundled plugin uses oxct lsp from your project."
      : "",
  ]
    .filter(Boolean)
    .join("\n");
  if (failures.length)
    throw new Error(
      `Workspace setup was applied, but extension installation needs attention:\n${failures.join("\n")}\n${hints}`,
    );
  if (ui) {
    if (hints) ui.note(hints, "Next steps");
    ui.outro("IDE setup applied");
  } else console.log(`◆ IDE setup applied\n${hints}`);
}

export function parseIdeOptions(args) {
  if (args[0] !== "install") throw new Error("Use oxct ide install");
  const options = {
    ides: [],
    extensions: true,
    config: true,
    dryRun: false,
    yes: false,
    actionSet: false,
  };
  for (let index = 1; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--ide") {
      const value = optionValue(args, ++index, arg);
      if (!ideNames.includes(value))
        throw new Error(`Unknown IDE: ${value}. Choose ${ideNames.join(", ")}.`);
      options.ides.push(value);
    } else if (arg === "--config-only") {
      options.extensions = false;
      options.config = true;
      options.actionSet = true;
    } else if (arg === "--extensions-only") {
      options.extensions = true;
      options.config = false;
      options.actionSet = true;
    } else if (arg === "--dry-run") options.dryRun = true;
    else if (arg === "--yes") options.yes = true;
    else throw new Error(`Unknown ide install option: ${arg}`);
  }
  options.ides = [...new Set(options.ides)];
  return options;
}
