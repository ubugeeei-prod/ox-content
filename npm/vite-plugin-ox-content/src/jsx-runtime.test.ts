import { describe, it, expect } from "vite-plus/test";
import { createRequire } from "node:module";
import { Fragment, jsx, jsxs } from "./jsx-runtime";
import { jsxDEV } from "./jsx-dev-runtime";
import packageJson from "../package.json" with { type: "json" };

const require = createRequire(import.meta.url);

// `jsxImportSource: "@ox-content/vite-plugin"` makes the JSX transform emit
// `import { jsx } from "@ox-content/vite-plugin/jsx-runtime"` (or
// `/jsx-dev-runtime` in dev), so both subpaths have to exist as their own
// modules — the same symbols on the package's main entry cannot satisfy them.
describe("jsx-runtime entry", () => {
  it("exports everything the automatic transform imports", () => {
    expect(jsx("span", { children: "a < b" }).__html).toBe("<span>a &lt; b</span>");
    expect(jsxs("ul", { children: [jsx("li", {})] }).__html).toBe("<ul><li></li></ul>");
    expect(Fragment({ children: "bare" }).__html).toBe("bare");
  });
});

describe("jsx-dev-runtime entry", () => {
  it("renders the same html as jsx, ignoring the debug arguments", () => {
    expect(
      jsxDEV("p", { children: "a < b" }, undefined, false, {
        fileName: "Hero.tsx",
        lineNumber: 3,
        columnNumber: 5,
      }).__html,
    ).toBe("<p>a &lt; b</p>");
  });
});

describe("jsx runtime packaging", () => {
  for (const subpath of ["./jsx-runtime", "./jsx-dev-runtime"] as const) {
    const name = subpath.slice(2);

    it(`declares ${subpath} in package exports`, () => {
      const exported = (packageJson.exports as Record<string, Record<string, string>>)[subpath];
      expect(exported).toBeDefined();
      expect(exported.import).toBe(`./dist/${name}.mjs`);
      expect(exported.require).toBe(`./dist/${name}.cjs`);
      expect(exported.types).toBe(`./dist/${name}.d.mts`);
    });

    it(`builds ${subpath} as its own bundle entry`, () => {
      const entries: string[] = require("../vite.config.ts").default.pack.entry;
      expect(entries).toContain(`src/${name}.ts`);
    });
  }
});

describe("typed hover packaging", () => {
  it("leaves the native-preview sync API external so it resolves its own tsgo binary", () => {
    const neverBundle: string[] = require("../vite.config.ts").default.pack.deps.neverBundle;
    expect(neverBundle).toContain("@typescript/native-preview");
    expect(neverBundle).toContain("@typescript/native-preview/unstable/sync");
  });
});
