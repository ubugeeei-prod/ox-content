import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { runLinkCheck, runMdcCheck } from "./oxct-checkers.mjs";
import { runI18n } from "./oxct-i18n.mjs";
import { runOgPreview } from "./oxct-og-preview.mjs";
import { loadNapi } from "./oxct-napi.mjs";

const here = dirname(fileURLToPath(import.meta.url));

export function main(args) {
  if (isHelp(args)) {
    printHelp();
    return;
  }

  const [command, ...rest] = args;
  if (command === "i18n") {
    runI18n(rest);
    return;
  }
  if (command === "link-check") {
    runLinkCheck(rest, (args) => {
      runRustCli({ binary: "ox-content-link-check", crate: "ox_content_link_checker", args });
    });
    return;
  }
  if (command === "mdc-check") {
    runMdcCheck(rest, (args) => {
      runRustCli({ binary: "ox-content-mdc-check", crate: "ox_content_mdc_checker", args });
    });
    return;
  }
  if (command === "lsp") {
    runLsp(rest);
    return;
  }
  if (command === "migrate") {
    runMigrate(rest);
    return;
  }
  if (command === "og-preview") {
    runOgPreview(rest);
    return;
  }

  throw new Error(`Unknown command: ${command}`);
}

function runLsp(args) {
  if (isHelp(args)) {
    printLspHelp();
    return;
  }
  if (args.length > 0) {
    throw new Error(`Unknown lsp option: ${args[0]}`);
  }
  loadNapi().runLsp();
}

function runMigrate(args) {
  if (isHelp(args)) {
    printMigrateHelp();
    return;
  }

  const [target, ...rest] = args;
  if (target !== "vitepress") {
    throw new Error("Unknown migrate target. Use `oxct migrate vitepress`.");
  }
  if (isHelp(rest)) {
    printMigrateVitePressHelp();
    return;
  }

  runCommand(process.execPath, [resolve(here, "migrate-vitepress.mjs"), ...rest], {
    stdio: "inherit",
  });
}

function runRustCli({ binary, crate, args }) {
  const direct = spawnSync(binary, args, { stdio: "inherit" });
  if (!isCommandNotFound(direct)) {
    setExitCodeFromCommand(direct, binary);
    return;
  }

  const workspaceRoot = findCargoWorkspaceRoot(process.cwd()) ?? findCargoWorkspaceRoot(here);
  if (!workspaceRoot) {
    throw new Error(
      `Could not find ${binary}. Install the Rust binary or run oxct from the ox-content repository.`,
    );
  }

  runCommand("cargo", ["run", "-p", crate, "--bin", binary, "--", ...args], {
    cwd: workspaceRoot,
    stdio: "inherit",
  });
}

function runCommand(command, args, options) {
  setExitCodeFromCommand(spawnSync(command, args, options), command);
}

function setExitCodeFromCommand(result, command) {
  if (result.error) {
    throw new Error(`Failed to run ${command}: ${result.error.message}`);
  }
  if (result.signal) {
    throw new Error(`${command} terminated with signal ${result.signal}`);
  }
  process.exitCode = result.status ?? 0;
}

function isCommandNotFound(result) {
  return result.error && "code" in result.error && result.error.code === "ENOENT";
}

function findCargoWorkspaceRoot(start) {
  let dir = resolve(start);
  for (;;) {
    const manifest = join(dir, "Cargo.toml");
    if (existsSync(manifest) && readFileSync(manifest, "utf8").includes("ox_content")) {
      return dir;
    }
    const parent = dirname(dir);
    if (parent === dir) {
      return undefined;
    }
    dir = parent;
  }
}

function isHelp(args) {
  return args.length === 0 || args[0] === "--help" || args[0] === "-h";
}

function printHelp() {
  console.log(`oxct

Usage:
  oxct <command> [options]

Commands:
  i18n <command>           Check dictionaries and validate MessageFormat 2
  link-check <files...>    Check Markdown/MDC local links
  migrate vitepress        Generate ox-content config from VitePress config
  mdc-check <files...>     Check MDC component syntax
  lsp                      Run the Ox Content language server over stdio
  og-preview               Generate a social preview SVG`);
}

function printLspHelp() {
  console.log(`oxct lsp

Usage:
  oxct lsp

Runs the bundled Ox Content language server over stdio.`);
}

function printMigrateHelp() {
  console.log(`oxct migrate

Usage:
  oxct migrate vitepress [config] [options]`);
}

function printMigrateVitePressHelp() {
  console.log(`oxct migrate vitepress [config]

Generate an editable ox-content options object from a VitePress config.

Options:
  -o, --out <file>     Write the generated TypeScript module to a file.
      --src-dir <dir>  Add/override the ox-content srcDir option.
      --out-dir <dir>  Add/override the ox-content outDir option.
  -f, --force          Overwrite --out when the file already exists.
  -h, --help           Show this help.`);
}
