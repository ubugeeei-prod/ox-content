import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const distDir = join(packageRoot, "dist");
const runtimeSource = join(
  packageRoot,
  "../../crates/ox_content_ssg/src/html/reader_chrome_runtime.js",
);

const runtime = await readFile(runtimeSource, "utf8");
const banner =
  "// Generated from crates/ox_content_ssg/src/html/reader_chrome_runtime.js.\n" +
  "// Run npm/vite-plugin-ox-content/scripts/build-reader-chrome-client.mjs.\n\n";
const declarations = `export type ReaderChromeRoot = Document | Element;
export declare function initReaderChrome(root?: ReaderChromeRoot): void;
`;

await mkdir(distDir, { recursive: true });
await writeFile(
  join(distDir, "reader-chrome-client.mjs"),
  `${banner}${runtime}\nexport { initReaderChrome };\n`,
);
await writeFile(
  join(distDir, "reader-chrome-client.cjs"),
  `${banner}"use strict";\n${runtime}\nmodule.exports = { initReaderChrome };\n`,
);
await writeFile(join(distDir, "reader-chrome-client.d.mts"), declarations);
await writeFile(join(distDir, "reader-chrome-client.d.cts"), declarations);
