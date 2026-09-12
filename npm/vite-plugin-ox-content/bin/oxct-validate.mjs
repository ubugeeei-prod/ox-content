import { access } from "node:fs/promises";
import * as path from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const oxContentOptionsMeta = Symbol.for("ox-content:vite-plugin-options");

const defaultConfigFiles = [
  "vite.config.ts",
  "vite.config.mts",
  "vite.config.js",
  "vite.config.mjs",
  "vite.config.cts",
  "vite.config.cjs",
  "ox-content.config.ts",
  "ox-content.config.mts",
  "ox-content.config.js",
  "ox-content.config.mjs",
  "ox-content.config.cts",
  "ox-content.config.cjs",
];

const directOptionKeys = new Set([
  "srcDir",
  "outDir",
  "collections",
  "ssg",
  "permalinks",
  "cascade",
  "search",
  "siteMaps",
  "feeds",
  "blog",
  "gfm",
  "mdx",
  "frontmatter",
]);

export async function runValidate(args) {
  if (isExplicitHelp(args)) {
    printValidateHelp();
    return;
  }

  const parsed = parseValidateOptions(args);
  const api = await loadPackageApi();
  const { root, resolvedOptions } = await loadResolvedOptions(parsed, api);
  const selectedOptions = selectCollections(resolvedOptions, parsed.collections);
  const manifest = await api.buildCollectionManifest(root, selectedOptions);

  console.log(formatSuccess(manifest));
}

function parseValidateOptions(args) {
  const options = {
    collections: [],
    config: undefined,
    cwd: process.cwd(),
  };

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];

    if (arg === "--config" || arg === "-c") {
      options.config = readValue(args, ++index, arg);
      continue;
    }

    if (arg === "--collection") {
      options.collections.push(readValue(args, ++index, arg));
      while (args[index + 1] && !args[index + 1].startsWith("-")) {
        options.collections.push(args[++index]);
      }
      continue;
    }

    if (arg.startsWith("-")) {
      throw new Error(`Unknown validate option: ${arg}`);
    }

    throw new Error(`Unexpected validate argument: ${arg}`);
  }

  return options;
}

async function loadResolvedOptions(options, api) {
  const configPath = await resolveConfigPath(options.config, options.cwd);
  const config = await loadConfig(configPath, options.cwd);
  const resolvedOptions = findResolvedOptions(config, api);
  if (!resolvedOptions) {
    throw new Error(`Could not find an oxContent() plugin or Ox Content options in ${configPath}.`);
  }

  return {
    root: resolveConfigRoot(config, options.cwd),
    resolvedOptions,
  };
}

async function loadConfig(configPath, cwd) {
  const vite = await import("vite");
  const loaded = await vite.loadConfigFromFile(createConfigEnv(), configPath, cwd, "silent");
  return normalizeConfig(loaded?.config, configPath);
}

function findResolvedOptions(config, api) {
  const pluginOptions = collectPluginOptions(config);
  if (pluginOptions.length > 1) {
    throw new Error("Found multiple oxContent() configurations. Keep one plugin or use --config.");
  }
  if (pluginOptions.length === 1) {
    return pluginOptions[0];
  }
  if (looksLikeDirectOxContentOptions(config)) {
    return api.resolveOptions(config);
  }
  return undefined;
}

function collectPluginOptions(value, output = [], seen = new Set()) {
  if (Array.isArray(value)) {
    for (const item of value) {
      collectPluginOptions(item, output, seen);
    }
    return output;
  }

  if (!isRecord(value)) {
    return output;
  }

  const metadata = value[oxContentOptionsMeta];
  if (isRecord(metadata) && isRecord(metadata.resolvedOptions) && !seen.has(metadata)) {
    seen.add(metadata);
    output.push(metadata.resolvedOptions);
  }

  if (Array.isArray(value.plugins)) {
    collectPluginOptions(value.plugins, output, seen);
  }

  return output;
}

function looksLikeDirectOxContentOptions(value) {
  if (!isRecord(value) || "plugins" in value) {
    return false;
  }
  return Object.keys(value).some((key) => directOptionKeys.has(key));
}

function selectCollections(options, requestedCollections) {
  const names = [...new Set(requestedCollections)];
  if (names.length === 0) {
    return options;
  }

  const available = options.collections?.enabled ? options.collections.collections : {};
  const missing = names.filter((name) => !available[name]);
  if (missing.length > 0) {
    const availableNames = Object.keys(available);
    const suffix =
      availableNames.length > 0 ? ` Available collections: ${availableNames.join(", ")}.` : "";
    throw new Error(`Unknown collection: ${missing.join(", ")}.${suffix}`);
  }

  return {
    ...options,
    collections: {
      enabled: true,
      collections: Object.fromEntries(names.map((name) => [name, available[name]])),
    },
  };
}

async function loadPackageApi() {
  const entry = path.resolve(here, "../dist/index.mjs");
  try {
    return await import(pathToFileURL(entry).href);
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    throw new Error(`Could not load @ox-content/vite-plugin runtime from ${entry}: ${message}`);
  }
}

async function resolveConfigPath(configPath, cwd) {
  if (configPath) {
    return resolvePath(cwd, configPath);
  }

  for (const candidate of defaultConfigFiles) {
    const resolved = resolvePath(cwd, candidate);
    if (await fileExists(resolved)) {
      return resolved;
    }
  }

  throw new Error("Could not find a Vite or Ox Content config. Pass --config <path>.");
}

function normalizeConfig(value, configPath) {
  if (!value || typeof value !== "object") {
    throw new Error(`Config did not export an object: ${configPath}`);
  }
  return value;
}

function resolveConfigRoot(config, cwd) {
  if (isRecord(config) && typeof config.root === "string") {
    return resolvePath(cwd, config.root);
  }
  return cwd;
}

function formatSuccess(manifest) {
  const collectionCount = Object.keys(manifest.collections).length;
  const documentCount = Object.values(manifest.collections).reduce(
    (count, entries) => count + entries.length,
    0,
  );
  return `[ox-content] Collection validation passed for ${documentCount} document${
    documentCount === 1 ? "" : "s"
  } in ${collectionCount} collection${collectionCount === 1 ? "" : "s"}.`;
}

function createConfigEnv() {
  return {
    command: "build",
    mode: "production",
    isSsrBuild: false,
    isPreview: false,
  };
}

function resolvePath(cwd, value) {
  return path.isAbsolute(value) ? path.normalize(value) : path.resolve(cwd, value);
}

async function fileExists(filePath) {
  try {
    await access(filePath);
    return true;
  } catch {
    return false;
  }
}

function readValue(args, index, option) {
  const value = args[index];
  if (!value || value.startsWith("-")) {
    throw new Error(`${option} requires a value`);
  }
  return value;
}

function isRecord(value) {
  return typeof value === "object" && value !== null;
}

function isExplicitHelp(args) {
  return args[0] === "--help" || args[0] === "-h";
}

function printValidateHelp() {
  console.log(`oxct validate

Usage:
  oxct validate [--config <path>] [--collection <name>...]

Runs configured collection validate hooks without a full production build.`);
}
