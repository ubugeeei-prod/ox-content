import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import packageJson from "../package.json" with { type: "json" };

const require = createRequire(import.meta.url);
const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const ssgSrc = join(packageRoot, "../../crates/ox_content_ssg/src");

describe("markdown table browser entry packaging", () => {
  it("declares a browser-only package subpath", () => {
    const exported = (packageJson.exports as Record<string, Record<string, string>>)[
      "./markdown-tables"
    ];
    expect(exported).toBeDefined();
    expect(exported.import).toBe("./dist/markdown-tables.mjs");
    expect(exported.require).toBe("./dist/markdown-tables.cjs");
    expect(exported.types).toBe("./dist/markdown-tables.d.mts");

    const entries: string[] = require("../vite.config.ts").default.pack.entry;
    expect(entries).toContain("src/markdown-tables.ts");
  });

  it("keeps the source entry free of server imports", () => {
    const source = readFileSync(join(packageRoot, "src/markdown-tables.ts"), "utf8");

    expect(source).not.toMatch(/\bfrom\s+["']\.(?:\/|\.\/)/);
    expect(source).not.toMatch(/node:|@ox-content\/napi|@resvg|playwright|puppeteer|fsevents/);
  });

  it("guards the built browser subpath against server graph imports", () => {
    const distFiles = ["dist/markdown-tables.mjs", "dist/markdown-tables.cjs"];
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
    expect(css).toContain('data-ox-table-scrollable="true"');
    expect(css).not.toMatch(
      /\bbody\b|\*\s*\{|font-family|background:|color:|border-collapse|padding:/,
    );
  });
});
