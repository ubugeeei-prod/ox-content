import { spawnSync } from "node:child_process";
import { mkdir, readdir, rm, writeFile } from "node:fs/promises";
import { basename, resolve } from "node:path";
import { prompts, optionValue } from "./oxct-prompts.mjs";
import { projectFiles, skins, palettes } from "./oxct-templates.mjs";

export async function runNew(args) {
  if (args.includes("--help") || args.includes("-h")) {
    console.log(`oxct new [directory]

Create a Vite/Ox Content site with a guided template, skin, palette, and package manager.

Options:
  --template docs|blog|minimal
  --skin <name>              Theme skin (default: editorial)
  --palette <name>           Color palette (default: nord)
  --package-manager npm|pnpm|yarn|bun|vp
  --install                 Install dependencies after creating the project
  --no-install              Only write the project
  --yes                     Use defaults without prompts
  -h, --help                Show this help`);
    return;
  }
  const options = parseNewOptions(args);
  let ui;
  if (!options.yes && process.stdin.isTTY && process.stdout.isTTY) {
    ui = await prompts();
    ui.intro("◆ Ox Content · Create a project");
    options.directory ??= ui.checked(
      await ui.text({
        message: "Project directory",
        placeholder: "my-content",
        defaultValue: "my-content",
        validate: (value) => (value?.trim() ? undefined : "Enter a directory"),
      }),
    );
    options.template ??= ui.checked(
      await ui.select({
        message: "What are you building?",
        options: [
          { value: "docs", label: "Documentation", hint: "Navigation, guides, and a home page" },
          { value: "blog", label: "Blog", hint: "Posts and content collections" },
          { value: "minimal", label: "Minimal site", hint: "A clean Markdown starting point" },
        ],
      }),
    );
    options.skin ??= ui.checked(
      await ui.select({
        message: "Choose a skin",
        options: skins.map((value) => ({ value, label: value })),
      }),
    );
    options.palette ??= ui.checked(
      await ui.select({
        message: "Choose a color palette",
        options: palettes.map((value) => ({ value, label: value })),
      }),
    );
    options.manager ??= ui.checked(
      await ui.select({
        message: "Package manager",
        initialValue: "vp",
        options: ["vp", "pnpm", "npm", "yarn", "bun"].map((value) => ({
          value,
          label: value === "vp" ? "Vite+" : value,
        })),
      }),
    );
    options.install ??= ui.checked(
      await ui.confirm({ message: "Install dependencies now?", initialValue: true }),
    );
  }
  options.directory ??= "my-content";
  options.template ??= "docs";
  options.skin ??= "editorial";
  options.palette ??= "nord";
  options.manager ??= "vp";
  options.install ??= false;
  validateNewOptions(options);
  const target = resolve(options.directory);
  await createProject(target, options);
  if (options.install) {
    const spinner = ui?.spinner();
    spinner?.start("Installing dependencies");
    try {
      const command =
        process.platform === "win32" && ["npm", "pnpm", "yarn"].includes(options.manager)
          ? `${options.manager}.cmd`
          : options.manager;
      const result = spawnSync(command, ["install"], { cwd: target, stdio: "inherit" });
      if (result.error || result.status !== 0)
        throw new Error(
          `Dependency installation failed. Project files are saved in ${target}; retry ${options.manager} install there. ${result.error?.message ?? ""}`,
        );
      spinner?.stop("Dependencies installed");
    } catch (error) {
      spinner?.stop("Installation failed");
      throw error;
    }
  }
  const runner = options.manager === "vp" ? "vp" : `${options.manager} run`;
  const steps = [
    `cd ${JSON.stringify(options.directory)}`,
    ...(!options.install ? [`${options.manager} install`] : []),
    `${runner} dev`,
  ];
  if (ui) {
    ui.note(steps.join("\n"), "Your next steps");
    ui.outro(
      `Created ${basename(target)} · ${options.template} · ${options.skin} / ${options.palette}`,
    );
  } else console.log(`◆ Created ${target}\n${steps.join("\n")}`);
}

export function parseNewOptions(args) {
  const options = {};
  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    const keys = {
      "--template": "template",
      "--skin": "skin",
      "--palette": "palette",
      "--package-manager": "manager",
    };
    if (keys[arg]) options[keys[arg]] = optionValue(args, ++index, arg);
    else if (arg === "--yes") options.yes = true;
    else if (arg === "--install") options.install = true;
    else if (arg === "--no-install") options.install = false;
    else if (arg.startsWith("-")) throw new Error(`Unknown new option: ${arg}`);
    else if (options.directory !== undefined) throw new Error(`Unexpected argument: ${arg}`);
    else options.directory = arg;
  }
  return options;
}

function validateNewOptions(options) {
  for (const [key, allowed] of [
    ["template", ["docs", "blog", "minimal"]],
    ["skin", skins],
    ["palette", palettes],
    ["manager", ["vp", "pnpm", "npm", "yarn", "bun"]],
  ]) {
    if (!allowed.includes(options[key]))
      throw new Error(`Invalid ${key}: ${options[key]}. Choose ${allowed.join(", ")}.`);
  }
}

export async function createProject(target, options) {
  let existed = true;
  try {
    if ((await readdir(target)).length > 0)
      throw new Error(`Directory is not empty: ${target}. Choose a new or empty directory.`);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
    existed = false;
  }
  const files = projectFiles(basename(target), options);
  await mkdir(target, { recursive: true });
  const written = [];
  try {
    for (const [name, content] of Object.entries(files)) {
      const file = resolve(target, name);
      await mkdir(resolve(file, ".."), { recursive: true });
      await writeFile(file, content, { flag: "wx" });
      written.push(file);
    }
  } catch (error) {
    // Remove only files created by this operation, preserving concurrent writes.
    await Promise.all(written.map((file) => rm(file)));
    if (!existed) await rm(target, { recursive: false }).catch(() => {});
    throw error;
  }
}
