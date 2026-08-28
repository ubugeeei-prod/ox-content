import { spawn } from "node:child_process";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const distDir = join(packageRoot, "dist");
const tscBin = join(
  packageRoot,
  "node_modules/.bin",
  process.platform === "win32" ? "tsc.cmd" : "tsc",
);

/**
 * Entries a browser, bare, or custom host imports without the plugin, the SSG,
 * or Node. `vp pack` routes them through a shared runtime chunk that pulls in
 * `node:fs`, so they are recompiled here as self-contained modules instead.
 */
const STANDALONE_ENTRIES = ["markdown-tables", "theme-tokens"];

const forbiddenIdentifiers = /(?:node:|@ox-content\/napi|@resvg|playwright|puppeteer|fsevents)/;
// Anchored to the start of a line so a documentation example inside a JSDoc
// block is not mistaken for a real import statement.
const forbiddenStatements = /^\s*(?:import\s|export\s[^\n]*\sfrom\s|[^\n]*\brequire\s*\()/m;

await mkdir(distDir, { recursive: true });
const tempRoot = await mkdtemp(join(tmpdir(), "ox-standalone-entries-"));

try {
  for (const entry of STANDALONE_ENTRIES) {
    await compileStandalone(entry, "ES2022", "esm", "mjs");
    await compileStandalone(entry, "CommonJS", "cjs", "cjs");
  }
} finally {
  await rm(tempRoot, { recursive: true, force: true });
}

async function compileStandalone(entry, module, directoryName, extension) {
  const sourceFile = join(packageRoot, `src/${entry}.ts`);
  const outDir = join(tempRoot, entry, directoryName);
  await runTsc([
    sourceFile,
    "--ignoreConfig",
    "--target",
    "ES2022",
    "--module",
    module,
    "--lib",
    "ES2022,DOM",
    "--strict",
    "--skipLibCheck",
    "--sourceMap",
    "--inlineSources",
    "--declaration",
    "false",
    "--outDir",
    outDir,
    "--pretty",
    "false",
  ]);

  const jsFile = `${entry}.${extension}`;
  const mapFile = `${jsFile}.map`;
  const compiledJs = join(outDir, `${entry}.js`);
  const compiledMap = join(outDir, `${entry}.js.map`);
  const outputJs = join(distDir, jsFile);
  const outputMap = join(distDir, mapFile);

  const js = (await readFile(compiledJs, "utf8")).replace(
    `sourceMappingURL=${entry}.js.map`,
    `sourceMappingURL=${mapFile}`,
  );
  if (forbiddenIdentifiers.test(js) || forbiddenStatements.test(js)) {
    throw new Error(`Refusing to publish ${jsFile}: server/runtime import found`);
  }

  const map = JSON.parse(await readFile(compiledMap, "utf8"));
  map.file = jsFile;

  await writeFile(outputJs, js);
  await writeFile(outputMap, `${JSON.stringify(map)}\n`);
}

async function runTsc(args) {
  await new Promise((resolve, reject) => {
    const child = spawn(tscBin, args, {
      cwd: packageRoot,
      stdio: "inherit",
    });
    child.on("error", reject);
    child.on("exit", (code) => {
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`tsc exited with ${code}`));
      }
    });
  });
}
