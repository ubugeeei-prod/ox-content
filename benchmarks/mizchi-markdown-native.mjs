import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";

import { renderNativeCommandSource } from "./mizchi-markdown-native-template.mjs";

const PACKAGE_NAME = "@mizchi/markdown";
const REPOSITORY_URL = "https://github.com/mizchi/markdown.mbt.git";
const NATIVE_LABEL = "@mizchi/markdown (native)";
const COMMAND_PATH = "src/cmd/ox-content-competitor";
const COMMAND_DIR = join("src", "cmd", "ox-content-competitor");
const BINARY_PATH = join(
  "_build",
  "native",
  "release",
  "build",
  "cmd",
  "ox-content-competitor",
  "ox-content-competitor.exe",
);

const KNOWN_GIT_HEADS = new Map([
  ["0.8.2", "159b5a38b500f066d78de6a5be61d32379abf0f9"],
  ["0.8.3", "b574e7f07c0d47e1c0aeff7dd344891511cfd631"],
]);

export function loadMizchiMarkdownNativeBenchmarks({ sampleMarkdown, sizeSpecs, runs }) {
  const command = prepareNativeCommand({ sampleMarkdown, sizeSpecs });
  if (!command) return null;

  const result = spawnSync(command.binaryPath, ["--runs", String(runs)], {
    cwd: command.checkoutDir,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    timeout: 10 * 60 * 1000,
  });

  if (result.error || result.status !== 0) {
    warnNativeUnavailable("run native benchmark", result);
    return null;
  }

  return parseBenchmarkOutput(result.stdout);
}

export function loadMizchiMarkdownNativeConformanceRenderings(examples) {
  const command = prepareNativeCommand({ sampleMarkdown: "", sizeSpecs: [] });
  if (!command) return null;

  const rendered = [];
  let failureWarned = false;
  for (const [index, example] of examples.entries()) {
    const result = spawnSync(command.binaryPath, ["--render-stdin"], {
      cwd: command.checkoutDir,
      input: example.markdown,
      encoding: "utf8",
      maxBuffer: 4 * 1024 * 1024,
      timeout: 30 * 1000,
    });

    if (result.error || result.status !== 0) {
      if (!failureWarned) {
        warnNativeUnavailable(`render CommonMark example ${index + 1}`, result);
        failureWarned = true;
      }
      rendered.push("");
      continue;
    }

    rendered.push(result.stdout);
  }

  return { name: NATIVE_LABEL, rendered };
}

function prepareNativeCommand({ sampleMarkdown, sizeSpecs }) {
  const metadata = loadInstalledPackageMetadata();
  if (!metadata) {
    console.warn(`Skipping ${NATIVE_LABEL}: ${PACKAGE_NAME} is not installed.`);
    return null;
  }

  const gitHead = resolvePackageGitHead(metadata);
  if (!gitHead) {
    console.warn(
      `Skipping ${NATIVE_LABEL}: could not resolve ${PACKAGE_NAME}@${metadata.version} git head.`,
    );
    return null;
  }

  if (!hasCommand("git")) {
    console.warn(`Skipping ${NATIVE_LABEL}: git is not available.`);
    return null;
  }

  if (!hasCommand("moon")) {
    console.warn(`Skipping ${NATIVE_LABEL}: moon is not available.`);
    return null;
  }

  const checkoutDir = ensureCheckout(gitHead);
  if (!checkoutDir) return null;

  writeNativeCommand(checkoutDir, sampleMarkdown, sizeSpecs);

  const update = spawnSync("moon", ["update"], {
    cwd: checkoutDir,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    timeout: 5 * 60 * 1000,
  });

  if (update.error || update.status !== 0) {
    warnNativeUnavailable("resolve native source dependencies", update);
    return null;
  }

  const build = spawnSync("moon", ["build", "--target", "native", "--release", COMMAND_PATH], {
    cwd: checkoutDir,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    timeout: 5 * 60 * 1000,
  });

  if (build.error || build.status !== 0) {
    warnNativeUnavailable("build native benchmark command", build);
    return null;
  }

  return {
    checkoutDir,
    binaryPath: join(checkoutDir, BINARY_PATH),
  };
}

function loadInstalledPackageMetadata() {
  const packageJsonPath = resolveInstalledPackageJson();
  if (!packageJsonPath) return null;
  return JSON.parse(readFileSync(packageJsonPath, "utf8"));
}

function resolveInstalledPackageJson() {
  const fromDirs = [
    join(process.cwd(), "benchmarks", "bundle-size"),
    join(process.cwd(), "benchmarks", "commonmark-conformance"),
    process.cwd(),
  ];

  for (const fromDir of fromDirs) {
    const packageJsonPath = findNodeModulesPackageJson(fromDir);
    if (packageJsonPath) return packageJsonPath;
  }

  return null;
}

function findNodeModulesPackageJson(startDir) {
  let currentDir = startDir;
  while (true) {
    const packageJsonPath = join(currentDir, "node_modules", "@mizchi", "markdown", "package.json");
    if (existsSync(packageJsonPath)) return packageJsonPath;

    const parentDir = dirname(currentDir);
    if (parentDir === currentDir) return null;
    currentDir = parentDir;
  }
}

function resolvePackageGitHead(metadata) {
  if (KNOWN_GIT_HEADS.has(metadata.version)) {
    return KNOWN_GIT_HEADS.get(metadata.version);
  }

  const result = spawnSync(
    "npm",
    ["view", `${PACKAGE_NAME}@${metadata.version}`, "gitHead", "--json"],
    {
      encoding: "utf8",
      maxBuffer: 1024 * 1024,
      timeout: 30 * 1000,
    },
  );

  if (result.error || result.status !== 0) return null;

  try {
    const gitHead = JSON.parse(result.stdout);
    return typeof gitHead === "string" && gitHead.length > 0 ? gitHead : null;
  } catch {
    return result.stdout.trim() || null;
  }
}

function hasCommand(command) {
  const result = spawnSync(command, ["--version"], {
    encoding: "utf8",
    maxBuffer: 1024 * 1024,
    timeout: 30 * 1000,
  });
  return !result.error && result.status === 0;
}

function ensureCheckout(gitHead) {
  const cacheRoot =
    process.env.OX_CONTENT_MIZCHI_MARKDOWN_NATIVE_CACHE_DIR ??
    join(tmpdir(), "ox-content-mizchi-markdown-native");
  const checkoutDir = join(cacheRoot, gitHead);

  if (existsSync(checkoutDir) && !existsSync(join(checkoutDir, ".git"))) {
    rmSync(checkoutDir, { force: true, recursive: true });
  }

  mkdirSync(cacheRoot, { recursive: true });

  if (!existsSync(checkoutDir)) {
    const clone = spawnSync("git", ["clone", "--filter=blob:none", REPOSITORY_URL, checkoutDir], {
      encoding: "utf8",
      maxBuffer: 64 * 1024 * 1024,
      timeout: 5 * 60 * 1000,
    });

    if (clone.error || clone.status !== 0) {
      warnNativeUnavailable("clone native source", clone);
      return null;
    }
  }

  const checkout = spawnSync("git", ["checkout", "--detach", gitHead], {
    cwd: checkoutDir,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
    timeout: 2 * 60 * 1000,
  });

  if (checkout.error || checkout.status !== 0) {
    warnNativeUnavailable("checkout native source", checkout);
    return null;
  }

  return checkoutDir;
}

function writeNativeCommand(checkoutDir, sampleMarkdown, sizeSpecs) {
  const outputDir = join(checkoutDir, COMMAND_DIR);
  mkdirSync(outputDir, { recursive: true });
  writeFileSync(
    join(outputDir, "moon.pkg"),
    `supported_targets = "native"

import {
  "mizchi/markdown",
  "moonbitlang/async",
  "moonbitlang/async/io",
  "moonbitlang/async/stdio",
  "moonbitlang/core/bench",
  "moonbitlang/core/env",
  "moonbitlang/core/string",
  "moonbitlang/x/sys",
}

pkgtype(kind: "executable")
`,
  );
  writeFileSync(
    join(outputDir, "main.mbt"),
    renderNativeCommandSource({
      nativeLabel: NATIVE_LABEL,
      sampleMarkdown,
      sizeSpecs,
    }),
  );
}

function parseBenchmarkOutput(output) {
  const grouped = {
    parse: Object.create(null),
    render: Object.create(null),
  };

  for (const line of output.trim().split(/\r?\n/)) {
    if (!line) continue;
    const [suite, name, size, opsPerSec, avgMs, throughputMBs] = line.split("\t");
    if (!grouped[suite] || !name || !size) continue;
    grouped[suite][size] ??= Object.create(null);
    grouped[suite][size][name] ??= [];
    grouped[suite][size][name].push({
      opsPerSec: Number.parseFloat(opsPerSec),
      avgMs: Number.parseFloat(avgMs),
      throughputMBs: Number.parseFloat(throughputMBs),
    });
  }

  return {
    parse: summarizeGroupedSamples(grouped.parse),
    render: summarizeGroupedSamples(grouped.render),
  };
}

function summarizeGroupedSamples(grouped) {
  const bySize = Object.create(null);
  for (const [size, rowsByName] of Object.entries(grouped)) {
    bySize[size] = Object.entries(rowsByName).map(([name, samples]) => {
      const sortedSamples = [...samples].sort((a, b) => a.opsPerSec - b.opsPerSec);
      const median = sortedSamples[Math.floor(sortedSamples.length / 2)];
      return {
        name,
        opsPerSec: median.opsPerSec,
        avgMs: median.avgMs,
        throughputMBs: median.throughputMBs,
        samples,
      };
    });
  }
  return bySize;
}

function warnNativeUnavailable(action, result) {
  const error = result?.error ? `${result.error.name}: ${result.error.message}` : null;
  const stderr = typeof result?.stderr === "string" ? result.stderr.trim() : "";
  const stdout = typeof result?.stdout === "string" ? result.stdout.trim() : "";
  const details = [error, stderr, stdout].filter(Boolean).join("\n");
  console.warn(`Skipping ${NATIVE_LABEL}: failed to ${action}.${details ? `\n${details}` : ""}`);
}
