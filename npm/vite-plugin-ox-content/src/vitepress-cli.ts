#!/usr/bin/env node
import { access, mkdir, writeFile } from "node:fs/promises";
import * as path from "node:path";
import { pathToFileURL } from "node:url";
import { loadConfigFromFile, type ConfigEnv } from "vite";
import type { OxContentOptions } from "./types";
import { generateVitePressMigrationConfig, type VitePressConfig } from "./vitepress";

interface CliOptions {
  configPath?: string;
  out?: string;
  srcDir?: string;
  outDir?: string;
  force: boolean;
  help: boolean;
}

const DEFAULT_CONFIG_FILES = [
  ".vitepress/config.ts",
  ".vitepress/config.mts",
  ".vitepress/config.js",
  ".vitepress/config.mjs",
  ".vitepress/config.cts",
  ".vitepress/config.cjs",
];

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));

  if (args.help) {
    console.log(helpText());
    return;
  }

  const configPath = await resolveConfigPath(args.configPath);
  const config = await loadVitePressConfig(configPath);
  const overrides: OxContentOptions = {
    ...(args.srcDir ? { srcDir: args.srcDir } : {}),
    ...(args.outDir ? { outDir: args.outDir } : {}),
  };
  const source = generateVitePressMigrationConfig(config, overrides);

  if (!args.out) {
    process.stdout.write(source);
    return;
  }

  const outPath = path.resolve(args.out);
  if (!args.force && (await fileExists(outPath))) {
    throw new Error(`Refusing to overwrite existing file: ${outPath}. Pass --force to overwrite.`);
  }

  await mkdir(path.dirname(outPath), { recursive: true });
  await writeFile(outPath, source);
  console.log(`Wrote ${path.relative(process.cwd(), outPath) || outPath}`);
}

function parseArgs(argv: string[]): CliOptions {
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

async function resolveConfigPath(configPath: string | undefined): Promise<string> {
  if (configPath) {
    return path.resolve(configPath);
  }

  for (const candidate of DEFAULT_CONFIG_FILES) {
    const resolved = path.resolve(candidate);
    if (await fileExists(resolved)) {
      return resolved;
    }
  }

  throw new Error(
    `Could not find a VitePress config. Pass one explicitly, e.g. ${DEFAULT_CONFIG_FILES[0]}`,
  );
}

async function loadVitePressConfig(configPath: string): Promise<VitePressConfig> {
  const env: ConfigEnv = {
    command: "build",
    mode: "production",
    isSsrBuild: false,
    isPreview: false,
  };
  const loaded = await loadConfigFromFile(env, configPath, process.cwd(), "silent");
  const config = loaded?.config ?? (await import(pathToFileURL(configPath).href)).default;

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

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});
