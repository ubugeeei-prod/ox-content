import { spawnSync } from "node:child_process";
import { delimiter, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const extraArgs = process.argv.slice(2);
const defaultProject = process.env.VOID_PROJECT || "ox-content";
const docsBase = process.env.OX_CONTENT_DOCS_BASE || "/";
const docsSiteUrl = process.env.OX_CONTENT_DOCS_SITE_URL || "https://ox-content.void.app";

const commandName = (command) => (process.platform === "win32" ? `${command}.cmd` : command);

const hasOption = (args, option) =>
  args.some((arg) => arg === option || arg.startsWith(`${option}=`));

const childEnv = (cwd, overrides = {}) => {
  const env = { ...process.env, PWD: cwd, ...overrides };

  for (const key of Object.keys(env)) {
    if (key.startsWith("VP_")) {
      delete env[key];
    }
  }

  delete env.LC_ALL;
  delete env.LC_CTYPE;
  env.PATH = [resolve(cwd, "node_modules/.bin"), resolve(root, "node_modules/.bin"), env.PATH]
    .filter(Boolean)
    .join(delimiter);

  return env;
};

const run = (command, args, options = {}) => {
  const cwd = options.cwd ? resolve(root, options.cwd) : root;

  const result = spawnSync(commandName(command), args, {
    cwd,
    env: childEnv(cwd, options.env),
    stdio: "inherit",
  });

  if (result.error) {
    throw result.error;
  }

  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
};

const voidArgs = ["void@0.9.0", "deploy"];

if (!hasOption(extraArgs, "--project")) {
  voidArgs.push("--project", defaultProject);
}

if (!hasOption(extraArgs, "--dir")) {
  voidArgs.push("--dir", "docs/dist/docs");
}

voidArgs.push(...extraArgs);

run("cargo", ["build", "--workspace"]);
run("napi", ["build", "--release"], { cwd: "crates/ox_content_napi" });
run("vp", ["pack"], { cwd: "npm/ox-content-islands" });
run("vp", ["pack"], { cwd: "npm/vite-plugin-ox-content" });
run("vp", ["build"], {
  cwd: "docs",
  env: {
    OX_CONTENT_DOCS_BASE: docsBase,
    OX_CONTENT_DOCS_SITE_URL: docsSiteUrl,
  },
});
run("vpx", voidArgs);
