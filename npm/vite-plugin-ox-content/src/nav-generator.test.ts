import { describe, expect, it } from "vite-plus/test";
import { generateNavCode, generateNavMetadata } from "./nav-generator";

describe("generateNavMetadata", () => {
  it("derives sorted API nav items from docs files through the native binding", () => {
    const nav = generateNavMetadata(
      [
        { file: "/repo/src/types.ts", entries: [] },
        { file: "/repo/src/index.ts", entries: [] },
        { file: "/repo/src/nav-generator.ts", entries: [] },
      ],
      "/api",
    );

    expect(nav).toEqual([
      { title: "Nav Generator", path: "/api/nav-generator" },
      { title: "Overview", path: "/api/index" },
      { title: "Types", path: "/api/types" },
    ]);
  });
});

describe("generateNavCode", () => {
  it("generates importable TypeScript nav metadata through the native binding", () => {
    const code = generateNavCode([{ title: "Docs", path: "/api/docs" }], "apiNav");

    expect(code).toContain("export interface NavItem");
    expect(code).toContain("export const apiNav: NavItem[] = [");
    expect(code).toContain('"title": "Docs"');
    expect(code).toMatch(/ as const;\n$/);
  });
});
