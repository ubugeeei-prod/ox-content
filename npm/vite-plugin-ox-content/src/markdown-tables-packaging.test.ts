import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import packageJson from "../package.json" with { type: "json" };

const require = createRequire(import.meta.url);
const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const ssgSrc = join(packageRoot, "../../crates/ox_content_ssg/src");
const SCROLLABLE_ATTR = "data-ox-table-scrollable";
const SCROLLABLE_FOCUS_SELECTOR = `.content table[${SCROLLABLE_ATTR}]:focus-visible`;

describe("markdown table browser entry packaging", () => {
  it("declares a browser-only package subpath", () => {
    const exported = (packageJson.exports as Record<string, PackageConditionalExport>)[
      "./markdown-tables"
    ];
    expect(exported).toBeDefined();
    expect(exported.import.types).toBe("./dist/markdown-tables.d.mts");
    expect(exported.import.default).toBe("./dist/markdown-tables.mjs");
    expect(exported.require.types).toBe("./dist/markdown-tables.d.cts");
    expect(exported.require.default).toBe("./dist/markdown-tables.cjs");

    const entries: string[] = require("../vite.config.ts").default.pack.entry;
    expect(entries).toContain("src/markdown-tables.ts");
  });

  it("keeps the source entry free of server imports", () => {
    const source = readFileSync(join(packageRoot, "src/markdown-tables.ts"), "utf8");

    expect(source).not.toMatch(/\bfrom\s+["']\.(?:\/|\.\/)/);
    expect(source).not.toMatch(/node:|@ox-content\/napi|@resvg|playwright|puppeteer|fsevents/);
  });

  it("guards the built browser subpath against server graph imports", () => {
    const distFiles = [
      "dist/markdown-tables.mjs",
      "dist/markdown-tables.cjs",
      "dist/markdown-tables.d.cts",
    ];
    for (const distFile of distFiles) {
      const absolutePath = join(packageRoot, distFile);
      if (!existsSync(absolutePath)) {
        continue;
      }
      const source = readFileSync(absolutePath, "utf8");
      expect(source).not.toMatch(
        /\brequire\s*\(|\bfrom\s+["']|node:|@ox-content\/napi|@resvg|playwright|puppeteer|fsevents/,
      );
    }
  });

  it("publishes isolated table styles separately from core.css", () => {
    const exportsField = packageJson.exports as Record<string, unknown>;
    expect(exportsField["./styles/markdown-tables.css"]).toBe("./dist/styles/markdown-tables.css");

    const css = readFileSync(join(ssgSrc, "plugins/markdown-tables.css"), "utf8");
    expect(css).toContain(".content table");
    expect(css).toContain(SCROLLABLE_FOCUS_SELECTOR);
    expect(css).toContain("outline: var(--octc-focus-ring");
    expect(css).toContain("outline-offset: var(--octc-focus-offset");
    expect(css).not.toMatch(
      /\bbody\b|\*\s*\{|font-family|background:|color:|border-collapse|padding:/,
    );
  });

  it("marks scrollable tables with one contract across every published artifact", () => {
    // `toggleAttribute()` writes the empty string, never "true", so a stylesheet
    // that compares the value can never match a table these writers marked.
    const helper = readFileSync(join(packageRoot, "src/markdown-tables.ts"), "utf8");
    expect(helper).toContain(`const SCROLLABLE_ATTR = "${SCROLLABLE_ATTR}";`);
    expect(helper).toContain("table.toggleAttribute(SCROLLABLE_ATTR, scrollable);");
    expect(helper).not.toMatch(/setAttribute\(\s*SCROLLABLE_ATTR/);

    const runtime = readFileSync(join(ssgSrc, "ssg.js"), "utf8");
    expect(runtime).toContain(`table.toggleAttribute("${SCROLLABLE_ATTR}", scrollable);`);

    for (const stylesheet of ["plugins/markdown-tables.css", "ssg.css"]) {
      const css = readFileSync(join(ssgSrc, stylesheet), "utf8");
      expect(css, stylesheet).toContain(SCROLLABLE_FOCUS_SELECTOR);
      expect(css, stylesheet).not.toMatch(new RegExp(`${SCROLLABLE_ATTR}\\s*[~|^$*]?=`));
    }
  });
});

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
