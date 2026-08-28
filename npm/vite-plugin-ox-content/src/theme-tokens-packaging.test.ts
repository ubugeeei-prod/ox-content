import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import packageJson from "../package.json" with { type: "json" };

const require = createRequire(import.meta.url);
const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));

const SERVER_GRAPH = /node:|@ox-content\/napi|@resvg|playwright|puppeteer|fsevents|satori|cspell/;
// Anchored to the start of a line so the JSDoc usage example is not mistaken
// for a real import statement.
const IMPORT_STATEMENT = /^\s*(?:import\s|export\s[^\n]*\sfrom\s|[^\n]*\brequire\s*\()/m;

describe("theme token entry packaging", () => {
  it("declares a build-tool-neutral package subpath", () => {
    const exported = (packageJson.exports as Record<string, PackageConditionalExport>)[
      "./theme-tokens"
    ];
    expect(exported).toBeDefined();
    expect(exported.import.types).toBe("./dist/theme-tokens.d.mts");
    expect(exported.import.default).toBe("./dist/theme-tokens.mjs");
    expect(exported.require.types).toBe("./dist/theme-tokens.d.cts");
    expect(exported.require.default).toBe("./dist/theme-tokens.cjs");

    const entries: string[] = require("../vite.config.ts").default.pack.entry;
    expect(entries).toContain("src/theme-tokens.ts");
  });

  it("keeps the source entry free of every import", () => {
    const source = readFileSync(join(packageRoot, "src/theme-tokens.ts"), "utf8");

    // A bare host imports this subpath without the Vite plugin, the SSG, the
    // native binding, or a filesystem API — so the module has no imports at
    // all, not merely no server-side ones. `ThemeConfig` is matched
    // structurally by `ThemeTokenSource` instead of being imported.
    expect(source).not.toMatch(IMPORT_STATEMENT);
    expect(source).not.toMatch(SERVER_GRAPH);
  });

  it("compiles the subpath standalone instead of through the shared runtime chunk", () => {
    // `vp pack` gives the CommonJS entry a shared runtime chunk that requires
    // `node:fs`, which is exactly what a bare host must not load.
    const script = readFileSync(join(packageRoot, "scripts/build-standalone-entries.mjs"), "utf8");

    expect(script).toContain('const STANDALONE_ENTRIES = ["markdown-tables", "theme-tokens"];');
    expect(packageJson.scripts.build).toContain("node scripts/build-standalone-entries.mjs");
  });

  it("guards the built subpath against server graph imports", () => {
    const distFiles = ["dist/theme-tokens.mjs", "dist/theme-tokens.cjs", "dist/theme-tokens.d.cts"];
    for (const distFile of distFiles) {
      const absolutePath = join(packageRoot, distFile);
      if (!existsSync(absolutePath)) {
        continue;
      }
      const source = readFileSync(absolutePath, "utf8");
      expect(source, distFile).not.toMatch(SERVER_GRAPH);
      expect(source, distFile).not.toMatch(IMPORT_STATEMENT);
    }
  });

  it("routes the built-in SSG through the same renderer the subpath publishes", () => {
    const theme = readFileSync(join(packageRoot, "src/theme.ts"), "utf8");

    // One implementation, so the `:root` / `[data-theme="dark"]` /
    // `prefers-color-scheme` contract cannot drift between the two.
    expect(theme).toContain('from "./theme-tokens"');
    expect(theme).toContain("renderThemeTokenCss(theme)");
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
