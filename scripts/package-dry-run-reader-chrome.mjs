import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { join } from "node:path";

const publicValues = [
  "applyReaderChromeHtml",
  "readerChromeAttributes",
  "readerChromeCss",
  "readerChromeIsEnabled",
  "readerChromeNeedsJs",
  "readerChromeScript",
  "renderReaderChromeAttributes",
  "renderReaderChromeScriptTag",
  "renderReaderChromeStyleTag",
  "resolveReaderChromeInput",
];
const publicTypes = ["ReaderChromeInput", "ReaderChromeOptions", "ResolvedReaderChrome"];
const tscBin = join("node_modules", ".bin", process.platform === "win32" ? "tsc.cmd" : "tsc");

export function checkReaderChromeDeclarations({ pkg, tarball, packDir, failures, readPackedFile }) {
  for (const extension of ["mts", "cts"]) {
    const declaration = readPackedFile(tarball, `dist/reader-chrome.d.${extension}`);
    for (const name of [...publicValues, ...publicTypes]) {
      if (!new RegExp(`\\b${name}\\b`).test(declaration)) {
        failures.push(`${pkg.name} reader-chrome.d.${extension} is missing ${name}`);
      }
    }
    if (/\bapplyReaderChromeHtml\s+as\s+\w+\b/.test(declaration)) {
      failures.push(`${pkg.name} reader-chrome.d.${extension} aliases applyReaderChromeHtml`);
    }
    if (/reader-chrome2/.test(declaration)) {
      failures.push(`${pkg.name} reader-chrome.d.${extension} references reader-chrome2`);
    }
  }

  for (const mode of ["bundler", "nodenext", "node16"]) {
    checkReaderChromeConsumer({ pkg, tarball, packDir, failures, mode });
  }
}

function checkReaderChromeConsumer({ pkg, tarball, packDir, failures, mode }) {
  const consumerRoot = mkdtempSync(join(packDir, `reader-chrome-${mode}-`));
  const packageRoot = join(consumerRoot, "node_modules", "@ox-content", "vite-plugin");
  mkdirSync(packageRoot, { recursive: true });

  const extract = spawnSync("tar", ["-xzf", tarball, "-C", packageRoot, "--strip-components=1"], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (extract.error) {
    throw extract.error;
  }
  if (extract.status !== 0) {
    throw new Error(extract.stderr || `Failed to extract ${pkg.name} into ${consumerRoot}`);
  }

  writeFileSync(join(consumerRoot, "package.json"), JSON.stringify({ type: "module" }));
  writeFileSync(join(consumerRoot, "esm-fixture.ts"), esmFixture());
  writeFileSync(join(consumerRoot, "cjs-fixture.cts"), cjsFixture());
  writeFileSync(join(consumerRoot, "tsconfig.json"), tsconfig(mode));

  const result = spawnSync(tscBin, ["-p", join(consumerRoot, "tsconfig.json")], {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    failures.push(
      `${pkg.name} reader-chrome ${mode} consumer failed:\n${result.stdout}${result.stderr}`,
    );
  }
}

function esmFixture() {
  return [
    `import { ${publicValues.join(", ")} } from "@ox-content/vite-plugin/reader-chrome";`,
    `import type { ${publicTypes.join(", ")} } from "@ox-content/vite-plugin/reader-chrome";`,
    "",
    "const options: ReaderChromeOptions = { copy: true, externalLinks: false };",
    "const input: ReaderChromeInput = options;",
    "const resolved: ResolvedReaderChrome = resolveReaderChromeInput(input);",
    "const html: string = applyReaderChromeHtml('<pre><code>x</code></pre>', resolved);",
    "const attrs: Readonly<Record<string, ''>> = readerChromeAttributes(input);",
    "const snippets: string[] = [",
    "  html,",
    "  readerChromeCss(input),",
    "  readerChromeScript(input),",
    "  renderReaderChromeAttributes(input),",
    "  renderReaderChromeScriptTag(input),",
    "  renderReaderChromeStyleTag(input),",
    "];",
    "const flags: boolean[] = [readerChromeIsEnabled(resolved), readerChromeNeedsJs(resolved)];",
    "void attrs;",
    "void snippets;",
    "void flags;",
  ].join("\n");
}

function cjsFixture() {
  return [
    `import readerChrome = require("@ox-content/vite-plugin/reader-chrome");`,
    `type ReaderChromeInput = import("@ox-content/vite-plugin/reader-chrome").ReaderChromeInput;`,
    `type ReaderChromeOptions = import("@ox-content/vite-plugin/reader-chrome").ReaderChromeOptions;`,
    `type ResolvedReaderChrome = import("@ox-content/vite-plugin/reader-chrome").ResolvedReaderChrome;`,
    "",
    "const options: ReaderChromeOptions = { copy: true, backToTop: false };",
    "const input: ReaderChromeInput = options;",
    "const resolved: ResolvedReaderChrome = readerChrome.resolveReaderChromeInput(input);",
    "const html: string = readerChrome.applyReaderChromeHtml('<pre><code>x</code></pre>', input);",
    "const attrs: Readonly<Record<string, ''>> = readerChrome.readerChromeAttributes(resolved);",
    "const snippets: string[] = [",
    "  html,",
    "  readerChrome.readerChromeCss(input),",
    "  readerChrome.readerChromeScript(input),",
    "  readerChrome.renderReaderChromeAttributes(input),",
    "  readerChrome.renderReaderChromeScriptTag(input),",
    "  readerChrome.renderReaderChromeStyleTag(input),",
    "];",
    "const flags: boolean[] = [",
    "  readerChrome.readerChromeIsEnabled(resolved),",
    "  readerChrome.readerChromeNeedsJs(resolved),",
    "];",
    "void attrs;",
    "void snippets;",
    "void flags;",
  ].join("\n");
}

function tsconfig(mode) {
  const compilerOptions =
    mode === "bundler"
      ? {
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "Bundler",
          strict: true,
          skipLibCheck: true,
          noEmit: true,
        }
      : {
          target: "ES2022",
          module: mode === "nodenext" ? "NodeNext" : "Node16",
          moduleResolution: mode === "nodenext" ? "NodeNext" : "Node16",
          strict: true,
          skipLibCheck: true,
          noEmit: true,
        };

  return JSON.stringify(
    {
      compilerOptions,
      files: mode === "bundler" ? ["esm-fixture.ts"] : ["esm-fixture.ts", "cjs-fixture.cts"],
    },
    null,
    2,
  );
}
