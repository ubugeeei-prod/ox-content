import { existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { describe, expect, it } from "vite-plus/test";
import { typecheckCodeBlocks } from "./code-blocks";

const require = createRequire(import.meta.url);
const compilerRequire = createRequire(require.resolve("@typescript/native-preview/package.json"));
const platformPackage = compilerRequire.resolve(
  `@typescript/native-preview-${process.platform}-${process.arch}/package.json`,
);
const tsgoCommand = join(
  dirname(platformPackage),
  "lib",
  process.platform === "win32" ? "tsgo.exe" : "tsgo",
);

describe("standalone docs snippet typechecking", () => {
  it("checks valid snippets even when the consumer has a tsconfig.json", async () => {
    expect(existsSync("tsconfig.json")).toBe(true);
    const diagnostics = await typecheckCodeBlocks(
      "```ts typecheck\nexport const answer: number = 42;\n```",
      { tsgoCommand },
    );
    expect(diagnostics).toEqual([]);
  });

  it("still reports a real snippet type error instead of TS5112", async () => {
    const diagnostics = await typecheckCodeBlocks(
      '```ts typecheck\nexport const answer: number = "wrong";\n```',
      { tsgoCommand },
    );
    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0]?.message).toContain("TS2322");
    expect(diagnostics[0]?.message).not.toContain("TS5112");
  });
});
