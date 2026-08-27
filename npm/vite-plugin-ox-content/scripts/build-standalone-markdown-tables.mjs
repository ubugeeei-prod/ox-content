import { spawn } from "node:child_process";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const sourceFile = join(packageRoot, "src/markdown-tables.ts");
const distDir = join(packageRoot, "dist");
const tscBin = join(
  packageRoot,
  "node_modules/.bin",
  process.platform === "win32" ? "tsc.cmd" : "tsc",
);

const forbiddenRuntimeImports =
  /(?:\brequire\s*\(|\bfrom\s+["']|node:|@ox-content\/napi|@resvg|playwright|puppeteer|fsevents)/;

await mkdir(distDir, { recursive: true });
const tempRoot = await mkdtemp(join(tmpdir(), "ox-markdown-tables-"));

try {
  await compileStandalone("ES2022", "esm", "mjs");
  await compileStandalone("CommonJS", "cjs", "cjs");
} finally {
  await rm(tempRoot, { recursive: true, force: true });
}

async function compileStandalone(module, directoryName, extension) {
  const outDir = join(tempRoot, directoryName);
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

  const jsFile = `markdown-tables.${extension}`;
  const mapFile = `${jsFile}.map`;
  const compiledJs = join(outDir, "markdown-tables.js");
  const compiledMap = join(outDir, "markdown-tables.js.map");
  const outputJs = join(distDir, jsFile);
  const outputMap = join(distDir, mapFile);

  const js = (await readFile(compiledJs, "utf8")).replace(
    "sourceMappingURL=markdown-tables.js.map",
    `sourceMappingURL=${mapFile}`,
  );
  if (forbiddenRuntimeImports.test(js)) {
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
