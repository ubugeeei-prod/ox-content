import { access, mkdir, writeFile } from "node:fs/promises";
import * as path from "node:path";
import { pathToFileURL } from "node:url";
import type { OxContentOptions } from "./types";
import { generateVitePressMigrationConfig, type VitePressConfig } from "./vitepress";

interface RuntimeGlobals {
  Deno?: {
    args: string[];
    cwd(): string;
    exit(code?: number): never;
    stderr: {
      write(data: Uint8Array): Promise<number>;
    };
    stdout: {
      write(data: Uint8Array): Promise<number>;
    };
  };
  Bun?: unknown;
  process?: NodeJS.Process;
}

export type VitePressMigrationCliRuntimeName = "node" | "deno" | "bun";

export interface VitePressMigrationCliRuntime {
  name: VitePressMigrationCliRuntimeName;
  argv: string[];
  cwd(): string;
  writeStdout(value: string): void | Promise<void>;
  writeStderr(value: string): void | Promise<void>;
  setExitCode(code: number): void;
}

interface CliOptions {
  configPath?: string;
  out?: string;
  srcDir?: string;
  outDir?: string;
  force: boolean;
  help: boolean;
}

interface ConfigEnv {
  command: "build" | "serve";
  mode: string;
  isSsrBuild: boolean;
  isPreview: boolean;
}

const DEFAULT_CONFIG_FILES = [
  ".vitepress/config.ts",
  ".vitepress/config.mts",
  ".vitepress/config.js",
  ".vitepress/config.mjs",
  ".vitepress/config.cts",
  ".vitepress/config.cjs",
];

const textEncoder = new TextEncoder();

export async function runVitePressMigrationCli(
  runtime = createVitePressMigrationCliRuntime(),
): Promise<void> {
  const args = parseVitePressMigrationCliArgs(runtime.argv);

  if (args.help) {
    await runtime.writeStdout(helpText());
    return;
  }

  const cwd = runtime.cwd();
  const configPath = await resolveConfigPath(args.configPath, cwd);
  const config = await loadVitePressConfig(configPath, cwd, runtime.name);
  const overrides: OxContentOptions = {
    ...(args.srcDir ? { srcDir: args.srcDir } : {}),
    ...(args.outDir ? { outDir: args.outDir } : {}),
  };
  const source = generateVitePressMigrationConfig(config, overrides);

  if (!args.out) {
    await runtime.writeStdout(source);
    return;
  }

  const outPath = resolvePath(cwd, args.out);
  if (!args.force && (await fileExists(outPath))) {
    throw new Error(`Refusing to overwrite existing file: ${outPath}. Pass --force to overwrite.`);
  }

  await mkdir(path.dirname(outPath), { recursive: true });
  await writeFile(outPath, source);
  await runtime.writeStdout(`Wrote ${path.relative(cwd, outPath) || outPath}\n`);
}

export function createVitePressMigrationCliRuntime(): VitePressMigrationCliRuntime {
  const globals = globalThis as typeof globalThis & RuntimeGlobals;
  const deno = globals.Deno;
  const process = globals.process;

  if (deno) {
    return {
      name: "deno",
      argv: deno.args,
      cwd: () => deno.cwd(),
      writeStdout: async (value) => {
        await deno.stdout.write(textEncoder.encode(value));
      },
      writeStderr: async (value) => {
        await deno.stderr.write(textEncoder.encode(value));
      },
      setExitCode: (code) => {
        deno.exit(code);
      },
    };
  }

  if (!process) {
    throw new Error("Could not detect a supported JavaScript runtime.");
  }

  return {
    name: globals.Bun ? "bun" : "node",
    argv: process.argv.slice(2),
    cwd: () => process.cwd(),
    writeStdout: (value) => {
      process.stdout.write(value);
    },
    writeStderr: (value) => {
      process.stderr.write(value);
    },
    setExitCode: (code) => {
      process.exitCode = code;
    },
  };
}

export function parseVitePressMigrationCliArgs(argv: string[]): CliOptions {
  const options: CliOptions = {
    force: false,
    help: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];

    if (arg === "--help" || arg === "-h") {
      options.help = true;
      continue;
    }

    if (arg === "--force" || arg === "-f") {
      options.force = true;
      continue;
    }

    if (arg === "--out" || arg === "-o") {
      options.out = readOptionValue(argv, ++index, arg);
      continue;
    }

    if (arg === "--src-dir") {
      options.srcDir = readOptionValue(argv, ++index, arg);
      continue;
    }

    if (arg === "--out-dir") {
      options.outDir = readOptionValue(argv, ++index, arg);
      continue;
    }

    if (arg.startsWith("-")) {
      throw new Error(`Unknown option: ${arg}`);
    }

    if (options.configPath) {
      throw new Error(`Unexpected positional argument: ${arg}`);
    }

    options.configPath = arg;
  }

  return options;
}

function readOptionValue(argv: string[], index: number, option: string): string {
  const value = argv[index];
  if (!value || value.startsWith("-")) {
    throw new Error(`Missing value for ${option}`);
  }
  return value;
}

async function resolveConfigPath(configPath: string | undefined, cwd: string): Promise<string> {
  if (configPath) {
    return resolvePath(cwd, configPath);
  }

  for (const candidate of DEFAULT_CONFIG_FILES) {
    const resolved = resolvePath(cwd, candidate);
    if (await fileExists(resolved)) {
      return resolved;
    }
  }

  throw new Error(
    `Could not find a VitePress config. Pass one explicitly, e.g. ${DEFAULT_CONFIG_FILES[0]}`,
  );
}

async function loadVitePressConfig(
  configPath: string,
  cwd: string,
  runtime: VitePressMigrationCliRuntimeName,
): Promise<VitePressConfig> {
  const loaders =
    runtime === "deno" || runtime === "bun"
      ? [loadConfigByNativeImport, loadConfigWithVite]
      : [loadConfigWithVite, loadConfigByNativeImport];
  const errors: string[] = [];

  for (const load of loaders) {
    try {
      return await load(configPath, cwd);
    } catch (error) {
      errors.push(error instanceof Error ? error.message : String(error));
    }
  }

  throw new Error(
    `Could not load VitePress config: ${configPath}\n${errors.map((error) => `- ${error}`).join("\n")}`,
  );
}

async function loadConfigWithVite(configPath: string, cwd: string): Promise<VitePressConfig> {
  const vite = (await import("vite")) as {
    loadConfigFromFile(
      env: ConfigEnv,
      configFile?: string,
      configRoot?: string,
      logLevel?: "silent",
    ): Promise<{ config: unknown } | null>;
  };
  const loaded = await vite.loadConfigFromFile(createConfigEnv(), configPath, cwd, "silent");

  return normalizeLoadedConfig(loaded?.config, configPath);
}

async function loadConfigByNativeImport(configPath: string): Promise<VitePressConfig> {
  const url = pathToFileURL(configPath);
  url.searchParams.set("mtime", String(Date.now()));
  const module = (await import(url.href)) as { default?: unknown };

  return normalizeLoadedConfig(module.default ?? module, configPath);
}

async function normalizeLoadedConfig(value: unknown, configPath: string): Promise<VitePressConfig> {
  const config = typeof value === "function" ? await value(createConfigEnv()) : await value;

  if (!config || typeof config !== "object" || Array.isArray(config)) {
    throw new Error(`VitePress config did not export an object: ${configPath}`);
  }

  return config as VitePressConfig;
}

async function fileExists(filePath: string): Promise<boolean> {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}

function createConfigEnv(): ConfigEnv {
  return {
    command: "build",
    mode: "production",
    isSsrBuild: false,
    isPreview: false,
  };
}

function resolvePath(cwd: string, value: string): string {
  return path.isAbsolute(value) ? path.normalize(value) : path.resolve(cwd, value);
}

function helpText(): string {
  return `ox-content-migrate-vitepress [config]

Generate an editable ox-content options object from a VitePress config.

Options:
  -o, --out <file>     Write the generated TypeScript module to a file.
      --src-dir <dir>  Add/override the ox-content srcDir option.
      --out-dir <dir>  Add/override the ox-content outDir option.
  -f, --force          Overwrite --out when the file already exists.
  -h, --help           Show this help.

When --out is omitted, the generated module is printed to stdout.
`;
}
