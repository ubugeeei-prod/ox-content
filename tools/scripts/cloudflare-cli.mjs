import { spawnSync } from "node:child_process";
import { delimiter, dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");

// Vite+ includes its argument separator in the task's argv. Consume it before
// forwarding options so `deploy#docs -- --dry-run` cannot publish accidentally.
export const forwardCliArgs = (args) => (args[0] === "--" ? args.slice(1) : args);

const commandName = (command) => (process.platform === "win32" ? `${command}.cmd` : command);

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

export const run = (command, args, options = {}) => {
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

export const runWrangler = (args) =>
  // dlx runs outside the workspace, so pnpm 12 needs explicit build approvals.
  run("vp", [
    "exec",
    "--",
    "pnpm",
    "dlx",
    "--yes",
    "--allow-build",
    "esbuild",
    "--allow-build",
    "workerd",
    "wrangler@4.145.0",
    ...args,
  ]);

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  runWrangler(forwardCliArgs(process.argv.slice(2)));
}
