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

  it("normalizes the configured base path", () => {
    const nav = generateNavMetadata([{ file: "/repo/src/context.ts", entries: [] }], "api-ox/");

    expect(nav).toEqual([{ title: "Context", path: "/api-ox/context" }]);
  });

  it("passes TypeDoc nav ordering options to the native binding", () => {
    const nav = generateNavMetadata(
      [
        {
          file: "default",
          entries: [
            {
              name: "cli",
              kind: "function",
              description: "Runs the CLI.",
              file: "/repo/src/cli.ts",
              line: 2,
              endLine: 2,
              signature: "export function cli(): void",
            },
            {
              name: "version",
              kind: "variable",
              description: "Package version.",
              file: "/repo/src/version.ts",
              line: 1,
              endLine: 1,
              signature: "export const version = '1.0.0'",
            },
          ],
        },
      ],
      {
        basePath: "/api",
        pathStrategy: "typedoc",
        groupOrder: ["Variables", "Functions"],
      },
    );

    expect(nav[0]?.children?.map((group) => group.title)).toEqual(["Variables", "Functions"]);
  });
});

describe("generateNavCode", () => {
  it("generates importable TypeScript nav metadata through the native binding", () => {
    const code = generateNavCode([{ title: "Docs", path: "/api/docs" }], "apiNav");

    expect(code).toMatchSnapshot();
    expect(code).toMatch(/ as const;\n$/);
  });
});
