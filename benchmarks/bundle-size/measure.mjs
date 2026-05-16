import { execSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSizeSync } from "gzip-size";

const __dirname = dirname(fileURLToPath(import.meta.url));
const WORKSPACE_ROOT = join(__dirname, "..", "..");
const options = parseOptions(process.argv.slice(2));

/**
 * Get total size of a directory
 * @param {string} dir
 * @returns {{ total: number, gzipped: number, files: number }}
 */
function getDirSize(dir) {
  let total = 0;
  let gzipped = 0;
  let files = 0;

  function walk(currentDir) {
    try {
      const entries = readdirSync(currentDir, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = join(currentDir, entry.name);
        if (entry.isDirectory()) {
          walk(fullPath);
        } else if (entry.isFile()) {
          const content = readFileSync(fullPath);
          total += content.length;
          // Only gzip JS/CSS/HTML files
          if (/\.(js|css|html|json)$/.test(entry.name)) {
            gzipped += gzipSizeSync(content);
          } else {
            gzipped += content.length;
          }
          files++;
        }
      }
    } catch {
      // Directory doesn't exist
    }
  }

  walk(dir);
  return { total, gzipped, files };
}

/**
 * Estimate the initial request count for the root document and its local assets.
 * @param {string} distDir
 * @returns {number}
 */
function getInitialRequestCount(distDir) {
  const indexPath = join(distDir, "index.html");
  if (!existsSync(indexPath)) {
    return 0;
  }

  const resources = new Set();
  collectHtmlResourceUrls(readFileSync(indexPath, "utf8"), resources);
  collectCssResourceUrls(distDir, resources);

  return 1 + resources.size;
}

/**
 * @param {string} html
 * @param {Set<string>} resources
 */
function collectHtmlResourceUrls(html, resources) {
  const tagPattern = /<([a-zA-Z][\w:-]*)\b[^>]*>/g;
  for (const match of html.matchAll(tagPattern)) {
    const tagName = match[1].toLowerCase();
    const attrs = parseAttributes(match[0]);

    if (tagName === "script") {
      addResource(resources, attrs.get("src"));
    }

    if (tagName === "link" && isInitialLink(attrs)) {
      addResource(resources, attrs.get("href"));
    }

    if (tagName === "img" || tagName === "source") {
      addResource(resources, attrs.get("src"));
      addSrcsetResources(resources, attrs.get("srcset"));
    }

    addResource(resources, attrs.get("component-url"));
    addResource(resources, attrs.get("renderer-url"));
    addResource(resources, attrs.get("before-hydration-url"));
  }
}

/**
 * @param {string} tag
 * @returns {Map<string, string>}
 */
function parseAttributes(tag) {
  const attrs = new Map();
  const attrPattern = /([:\w-]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+)))?/g;
  for (const match of tag.matchAll(attrPattern)) {
    const [, name, doubleQuoted, singleQuoted, unquoted] = match;
    const value = doubleQuoted ?? singleQuoted ?? unquoted;
    if (value !== undefined) {
      attrs.set(name.toLowerCase(), value);
    }
  }

  return attrs;
}

/**
 * @param {Map<string, string>} attrs
 * @returns {boolean}
 */
function isInitialLink(attrs) {
  const href = attrs.get("href");
  if (!href) {
    return false;
  }

  const rel = attrs.get("rel")?.toLowerCase() ?? "";
  const rels = new Set(rel.split(/\s+/).filter(Boolean));
  return (
    rels.has("stylesheet") ||
    rels.has("preload") ||
    rels.has("modulepreload") ||
    rels.has("icon") ||
    rels.has("apple-touch-icon") ||
    rels.has("manifest")
  );
}

/**
 * @param {Set<string>} resources
 * @param {string | undefined} srcset
 */
function addSrcsetResources(resources, srcset) {
  if (!srcset) {
    return;
  }

  for (const candidate of srcset.split(",")) {
    const url = candidate.trim().split(/\s+/)[0];
    addResource(resources, url);
  }
}

/**
 * @param {string} distDir
 * @param {Set<string>} resources
 */
function collectCssResourceUrls(distDir, resources) {
  const visitedCss = new Set();
  const cssQueue = [...resources].filter(isCssPath);

  while (cssQueue.length > 0) {
    const cssPath = cssQueue.shift();
    if (!cssPath || visitedCss.has(cssPath)) {
      continue;
    }

    visitedCss.add(cssPath);
    const fullPath = join(distDir, cssPath);
    if (!isInsideDir(fullPath, distDir) || !existsSync(fullPath)) {
      continue;
    }

    const css = readFileSync(fullPath, "utf8");
    const discovered = extractCssUrls(css);
    for (const url of discovered) {
      const added = addResource(resources, url, `/${cssPath}`);
      const resourcePath = normalizeLocalUrl(url, `/${cssPath}`);
      if (added && resourcePath && isCssPath(resourcePath)) {
        cssQueue.push(resourcePath);
      }
    }
  }
}

/**
 * @param {string} css
 * @returns {string[]}
 */
function extractCssUrls(css) {
  const urls = [];
  const urlPattern = /url\(\s*(?:"([^"]+)"|'([^']+)'|([^'")]+))\s*\)/g;
  for (const match of css.matchAll(urlPattern)) {
    urls.push(match[1] ?? match[2] ?? match[3]);
  }

  const importPattern = /@import\s+(?:url\(\s*)?(?:"([^"]+)"|'([^']+)'|([^'")\s;]+))/g;
  for (const match of css.matchAll(importPattern)) {
    urls.push(match[1] ?? match[2] ?? match[3]);
  }

  return urls;
}

/**
 * @param {Set<string>} resources
 * @param {string | undefined} rawUrl
 * @param {string} basePath
 * @returns {boolean}
 */
function addResource(resources, rawUrl, basePath = "/index.html") {
  const resourcePath = normalizeLocalUrl(rawUrl, basePath);
  if (!resourcePath) {
    return false;
  }

  const previousSize = resources.size;
  resources.add(resourcePath);
  return resources.size !== previousSize;
}

/**
 * @param {string | undefined} rawUrl
 * @param {string} basePath
 * @returns {string | null}
 */
function normalizeLocalUrl(rawUrl, basePath) {
  const value = rawUrl?.trim();
  if (!value || value.startsWith("#") || value.startsWith("//")) {
    return null;
  }
  if (/^(?:data|blob|mailto|tel|javascript):/i.test(value)) {
    return null;
  }

  let parsed;
  try {
    parsed = new URL(value, `https://benchmark.local${basePath}`);
  } catch {
    return null;
  }

  if (parsed.origin !== "https://benchmark.local") {
    return null;
  }

  const pathname = decodeURIComponent(parsed.pathname).replace(/^\/+/, "");
  if (!pathname || pathname.endsWith("/")) {
    return null;
  }

  return pathname;
}

/**
 * @param {string} path
 * @returns {boolean}
 */
function isCssPath(path) {
  return /\.css$/i.test(path);
}

/**
 * @param {string} filePath
 * @param {string} dir
 * @returns {boolean}
 */
function isInsideDir(filePath, dir) {
  const resolvedFilePath = resolve(filePath);
  const resolvedDir = resolve(dir);
  return resolvedFilePath === resolvedDir || resolvedFilePath.startsWith(`${resolvedDir}/`);
}

/**
 * Format bytes to human readable string
 * @param {number} bytes
 * @returns {string}
 */
function formatBytes(bytes) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
}

/**
 * @typedef {Object} AppConfig
 * @property {string} name
 * @property {string} dir
 * @property {string} distDir
 * @property {string} buildCmd
 */

/** @type {AppConfig[]} */
const apps = [
  {
    name: "ox-content (bare)",
    dir: join(__dirname, "apps/ox-content-bare"),
    distDir: "dist",
    buildCmd: "npm run build",
  },
  {
    name: "ox-content (default)",
    dir: join(__dirname, "apps/ox-content"),
    distDir: "dist",
    buildCmd: "npm run build",
  },
  {
    name: "ox-content + Vue",
    dir: join(__dirname, "apps/ox-content-vue"),
    distDir: "dist",
    buildCmd: "npm run build",
  },
  {
    name: "VitePress (bare)",
    dir: join(__dirname, "apps/vitepress-bare"),
    distDir: ".vitepress/dist",
    buildCmd: "npm run build",
  },
  {
    name: "VitePress (default)",
    dir: join(__dirname, "apps/vitepress"),
    distDir: ".vitepress/dist",
    buildCmd: "npm run build",
  },
  {
    name: "Astro",
    dir: join(__dirname, "apps/astro"),
    distDir: "dist",
    buildCmd: "npm run build",
  },
  {
    name: "Astro + Vue",
    dir: join(__dirname, "apps/astro-vue"),
    distDir: "dist",
    buildCmd: "npm run build",
  },
];

/**
 * @param {string[]} args
 * @returns {{ jsonPath: string | null; skipBuild: boolean; skipInstall: boolean }}
 */
function parseOptions(args) {
  const parsed = {
    jsonPath: null,
    skipBuild: false,
    skipInstall: false,
  };

  for (let index = 0; index < args.length; index++) {
    const arg = args[index];
    if (arg === "--json") {
      parsed.jsonPath = readOptionValue(args, ++index, "--json");
      continue;
    }
    if (arg.startsWith("--json=")) {
      parsed.jsonPath = readInlineOptionValue(arg, "--json");
      continue;
    }
    if (arg === "--skip-build") {
      parsed.skipBuild = true;
      continue;
    }
    if (arg === "--skip-install") {
      parsed.skipInstall = true;
      continue;
    }
    if (arg === "--help" || arg === "-h") {
      printUsage();
      process.exit(0);
    }

    throw new Error(`Unknown argument: ${arg}`);
  }

  return parsed;
}

/**
 * @param {string[]} args
 * @param {number} index
 * @param {string} optionName
 * @returns {string}
 */
function readOptionValue(args, index, optionName) {
  const value = args[index];
  if (!value || value.startsWith("--")) {
    throw new Error(`${optionName} requires a file path`);
  }

  return value;
}

/**
 * @param {string} arg
 * @param {string} optionName
 * @returns {string}
 */
function readInlineOptionValue(arg, optionName) {
  const value = arg.slice(`${optionName}=`.length);
  if (!value) {
    throw new Error(`${optionName} requires a file path`);
  }

  return value;
}

function printUsage() {
  console.log(`Usage: node measure.mjs [--json <path>] [--skip-build] [--skip-install]

Options:
  --json <path>    Write benchmark results as JSON
  --skip-build     Measure existing build outputs without rebuilding
  --skip-install   Skip workspace install before building
  -h, --help       Show this help message`);
}

async function main() {
  console.log("Bundle Size Benchmark");
  console.log("=====================\n");

  // Install dependencies
  if (!options.skipBuild && !options.skipInstall) {
    console.log("Installing workspace dependencies...\n");
    try {
      const installCmd = existsSync(join(WORKSPACE_ROOT, "node_modules/.bin/vp"))
        ? "./node_modules/.bin/vp install"
        : "npm exec --yes --package vite-plus -- vp install";
      execSync(installCmd, { cwd: WORKSPACE_ROOT, stdio: "pipe" });
    } catch (e) {
      console.log(`  Workspace install failed - ${e.message}`);
      return;
    }
    console.log("");
  }

  // Build each app
  const results = [];

  for (const app of apps) {
    console.log(`Building ${app.name}...`);

    if (!options.skipBuild) {
      try {
        execSync(app.buildCmd, { cwd: app.dir, stdio: "pipe" });
      } catch (e) {
        console.log(`  Build failed: ${e.message}\n`);
        results.push({
          name: app.name,
          total: -1,
          gzipped: -1,
          files: 0,
          requests: 0,
          error: true,
        });
        continue;
      }
    }

    const distPath = join(app.dir, app.distDir);
    const size = {
      ...getDirSize(distPath),
      requests: getInitialRequestCount(distPath),
    };

    results.push({
      name: app.name,
      ...size,
    });

    console.log(`  Total: ${formatBytes(size.total)}`);
    console.log(`  Gzipped: ${formatBytes(size.gzipped)}`);
    console.log(`  Requests: ${size.requests}`);
    console.log(`  Files: ${size.files}\n`);
  }

  // Find baseline (ox-content bare)
  const oxContentBare = results.find((r) => r.name === "ox-content (bare)");
  const baseline =
    oxContentBare && !oxContentBare.error
      ? oxContentBare.gzipped
      : Math.min(...results.filter((r) => !r.error && r.gzipped > 0).map((r) => r.gzipped));

  // Sort by gzipped size
  results.sort((a, b) => {
    if (a.error) return 1;
    if (b.error) return -1;
    return a.gzipped - b.gzipped;
  });

  console.log("\nResults (sorted by gzipped size):");
  console.log("=================================\n");

  // Calculate dynamic column widths
  const nameWidth = Math.max(9, ...results.map((r) => r.name.length)); // min 9 for "Framework"
  const totalWidth = 10;
  const gzippedWidth = 10;
  const ratioWidth = 9;
  const requestsWidth = 8;
  const filesWidth = 5;

  // Print markdown table
  const header = `| ${"Framework".padEnd(nameWidth)} | ${"Total".padEnd(totalWidth)} | ${"Gzipped".padEnd(gzippedWidth)} | ${"Ratio".padEnd(ratioWidth)} | ${"Requests".padEnd(requestsWidth)} | ${"Files".padEnd(filesWidth)} |`;
  const separator = `|${"-".repeat(nameWidth + 2)}|${"-".repeat(totalWidth + 2)}|${"-".repeat(gzippedWidth + 2)}|${"-".repeat(ratioWidth + 2)}|${"-".repeat(requestsWidth + 2)}|${"-".repeat(filesWidth + 2)}|`;
  console.log(header);
  console.log(separator);

  for (const result of results) {
    if (result.error) {
      console.log(
        `| ${result.name.padEnd(nameWidth)} | ${"Error".padEnd(totalWidth)} | ${"-".padEnd(gzippedWidth)} | ${"-".padEnd(ratioWidth)} | ${"-".padEnd(requestsWidth)} | ${"-".padEnd(filesWidth)} |`,
      );
      continue;
    }

    const name = result.name.padEnd(nameWidth);
    const total = formatBytes(result.total).padEnd(totalWidth);
    const gzipped = formatBytes(result.gzipped).padEnd(gzippedWidth);
    const ratio = ((result.gzipped / baseline).toFixed(2) + "x").padEnd(ratioWidth);
    const requests = String(result.requests).padEnd(requestsWidth);
    const files = String(result.files).padEnd(filesWidth);
    console.log(`| ${name} | ${total} | ${gzipped} | ${ratio} | ${requests} | ${files} |`);
  }

  console.log("\n\nNotes:");
  console.log("- All frameworks built with production settings");
  console.log("- Gzipped size calculated for JS/CSS/HTML files");
  console.log("- Requests include index.html plus referenced local initial assets");
  console.log("- Same markdown content used across all frameworks");
  console.log("- Ratio is relative to the smallest bundle");

  const report = {
    name: "Bundle Size Benchmark",
    generatedAt: new Date().toISOString(),
    baseline,
    results,
  };

  if (options.jsonPath) {
    writeFileSync(options.jsonPath, `${JSON.stringify(report, null, 2)}\n`);
    console.log(`\nWrote bundle size JSON to ${options.jsonPath}`);
  }
}

main().catch(console.error);
