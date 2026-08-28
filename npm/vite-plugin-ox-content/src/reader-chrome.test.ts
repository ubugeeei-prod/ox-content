import { mkdtempSync, readFileSync, rmSync, writeFileSync, mkdirSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vite-plus/test";
import packageJson from "../package.json" with { type: "json" };
import {
  applyReaderChromeHtml,
  readerChromeAttributes,
  readerChromeCss,
  readerChromeScript,
  renderReaderChromeAttributes,
  renderReaderChromeScriptTag,
  renderReaderChromeStyleTag,
  resolveReaderChromeInput,
} from "./reader-chrome";
import { buildSsg } from "./ssg";
import { createTheme } from "./theme-renderer";
import type { ResolvedOptions } from "./types";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";

const require = createRequire(import.meta.url);
const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const tempDirs: string[] = [];

afterEach(() => {
  for (const dir of tempDirs.splice(0)) {
    rmSync(dir, { recursive: true, force: true });
  }
});

describe("reader chrome public API", () => {
  it("declares build-time, client, and stylesheet package subpaths", () => {
    const exportsField = packageJson.exports as Record<string, PackageConditionalExport | string>;
    expect(exportsField["./styles/reader-chrome.css"]).toBe("./dist/styles/reader-chrome.css");

    const server = exportsField["./reader-chrome"] as PackageConditionalExport;
    expect(server.import.types).toBe("./dist/reader-chrome.d.mts");
    expect(server.import.default).toBe("./dist/reader-chrome.mjs");
    expect(server.require.types).toBe("./dist/reader-chrome.d.cts");
    expect(server.require.default).toBe("./dist/reader-chrome.cjs");

    const client = exportsField["./reader-chrome/client"] as PackageConditionalExport;
    expect(client.import.types).toBe("./dist/reader-chrome-client.d.mts");
    expect(client.import.default).toBe("./dist/reader-chrome-client.mjs");
    expect(client.require.types).toBe("./dist/reader-chrome-client.d.cts");
    expect(client.require.default).toBe("./dist/reader-chrome-client.cjs");

    const entries: string[] = require("../vite.config.ts").default.pack.entry;
    expect(entries).toContain("src/reader-chrome.ts");
  });

  it("keeps the server helper out of the default renderer and builds the client from Rust", () => {
    const source = readFileSync(join(packageRoot, "src/reader-chrome.ts"), "utf8");
    expect(source).not.toContain('from "./ssg"');
    expect(source).not.toContain("defaultTheme");
    expect(source).not.toContain('from "./types"');

    const typesScript = readFileSync(
      join(packageRoot, "scripts/build-reader-chrome-types.mjs"),
      "utf8",
    );
    expect(typesScript).toContain("reader-chrome.d.mts");
    expect(typesScript).toContain("reader-chrome.d.cts");
    const buildScript = readFileSync(
      join(packageRoot, "scripts/build-reader-chrome-client.mjs"),
      "utf8",
    );
    expect(buildScript).toContain("reader_chrome_runtime.js");
    expect(packageJson.scripts.build).toContain("node scripts/build-reader-chrome-types.mjs");
    expect(packageJson.scripts.build).toContain("node scripts/build-reader-chrome-client.mjs");
  });

  it("applies the native code-copy transform without inventing disabled markup", () => {
    const code =
      '<pre class="ox-code-block" data-ox-code-source="const raw = 1;&#10;"><code>rendered</code></pre>';

    expect(applyReaderChromeHtml(code, false)).toBe(code);
    expect(applyReaderChromeHtml(code, { copy: false, externalLinks: false })).toBe(code);

    const html = applyReaderChromeHtml(code, {
      copy: true,
      externalLinks: false,
      backToTop: false,
    });
    expect(html).toContain('<div class="ox-code">');
    expect(html).toContain('aria-label="Copy code"');
    expect(html).toContain('data-ox-code-source="const raw = 1;&#10;"');
    expect(html).not.toContain("data-ox-copy-text");
  });

  it("renders host assets and root attributes only when controls are enabled", () => {
    expect(resolveReaderChromeInput({ copy: false })).toEqual({
      copy: false,
      externalLinks: true,
      backToTop: true,
    });
    expect(readerChromeAttributes(false)).toEqual({});
    expect(
      renderReaderChromeAttributes({ copy: true, externalLinks: false, backToTop: false }),
    ).toBe(" data-ox-reader-chrome data-ox-copy");

    expect(readerChromeCss(false)).toBe("");
    expect(readerChromeScript({ copy: false, externalLinks: true, backToTop: false })).toBe("");
    expect(
      renderReaderChromeStyleTag({ copy: true, externalLinks: false, backToTop: false }),
    ).toContain('data-ox-style="reader-chrome"');
    expect(
      renderReaderChromeScriptTag({ copy: true, externalLinks: false, backToTop: false }),
    ).toContain('data-ox-script="reader-chrome"');
  });
});

describe("reader chrome in custom SSG hosts", () => {
  it("lets bare mode enable official copy controls", async () => {
    const root = makeSite({
      "guide.md": ["# Guide", "", "```ts", "const value = 1;", "```"].join("\n"),
    });
    const built = await buildSsg(
      readerChromeOptions({
        bare: true,
        readerChrome: { copy: true, externalLinks: false, backToTop: false },
      }),
      root,
    );
    const html = readFileSync(join(root, "dist/guide/index.html"), "utf8");

    expect(built.errors).toEqual([]);
    expect(html).toContain('<div class="ox-code">');
    expect(html).toContain('data-ox-style="reader-chrome"');
    expect(html).toContain('data-ox-script="reader-chrome"');
    expect(html).toContain("initReaderChrome(document);");
  });

  it("leaves bare mode unchanged when reader chrome is disabled", async () => {
    const root = makeSite({
      "guide.md": ["# Guide", "", "```ts", "const value = 1;", "```"].join("\n"),
    });
    const built = await buildSsg(readerChromeOptions({ bare: true, readerChrome: false }), root);
    const html = readFileSync(join(root, "dist/guide/index.html"), "utf8");

    expect(built.errors).toEqual([]);
    expect(html).not.toContain("ox-code");
    expect(html).not.toContain("reader-chrome");
    expect(html).not.toContain("data-ox-copy");
  });

  it("post-processes ssg.render output without taking over the document", async () => {
    const root = makeSite({
      "guide.md": [
        "# Guide",
        "",
        "[Docs](https://example.com)",
        "",
        "```ts",
        "const value = 1;",
        "```",
      ].join("\n"),
    });
    const theme = createTheme({
      layouts: {
        default: ({ children }) => ({
          __html: `<!DOCTYPE html><html><head><title>Custom</title></head><body><article class="content">${children.__html}</article></body></html>`,
        }),
      },
    });
    const built = await buildSsg(
      readerChromeOptions({
        render: theme,
        readerChrome: { copy: true, externalLinks: true, backToTop: false },
      }),
      root,
    );
    const html = readFileSync(join(root, "dist/guide/index.html"), "utf8");

    expect(built.errors).toEqual([]);
    expect(html).toContain("<title>Custom</title>");
    expect(html).toContain('<div class="ox-code">');
    expect(html).toContain('class="ox-external"');
    expect(html.indexOf('data-ox-style="reader-chrome"')).toBeLessThan(html.indexOf("</head>"));
    expect(html.indexOf('data-ox-script="reader-chrome"')).toBeLessThan(html.indexOf("</body>"));
  });
});

function makeSite(files: Record<string, string>): string {
  const root = mkdtempSync(join(tmpdir(), "ox-reader-chrome-"));
  tempDirs.push(root);
  for (const [name, source] of Object.entries(files)) {
    const full = join(root, "content", name);
    mkdirSync(dirname(full), { recursive: true });
    writeFileSync(full, source);
  }
  return root;
}

function readerChromeOptions(ssg: Partial<ResolvedOptions["ssg"]>): ResolvedOptions {
  const base = createDocsResolvedOptions();
  return {
    ...base,
    search: { ...base.search, enabled: false },
    ssg: { ...base.ssg, ...ssg },
  };
}

interface PackageConditionalExport {
  import: {
    types: string;
    default: string;
  };
  require: {
    types: string;
    default: string;
  };
}
