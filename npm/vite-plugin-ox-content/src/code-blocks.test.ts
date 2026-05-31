import { describe, expect, it } from "vite-plus/test";
import { extractCodeBlocks, extractDocsTests, lintCodeBlocks } from "./code-blocks";

describe("code block utilities", () => {
  it("extracts docs tests from opted-in fences", async () => {
    const blocks = await extractDocsTests(
      ["```ts test", "expect(1).toBe(1)", "```", "", "```ts", "const x = 1", "```"].join("\n"),
    );

    expect(blocks).toHaveLength(1);
    expect(blocks[0]).toMatchObject({
      language: "ts",
      meta: "test",
      startLine: 2,
      endLine: 2,
    });
  });

  it("lints code block whitespace and exposes raw fence extraction", async () => {
    const source = ["```ts", "const x = 1;  ", "```"].join("\n");

    await expect(extractCodeBlocks(source)).resolves.toMatchObject([
      {
        language: "ts",
        code: "const x = 1;  ",
      },
    ]);

    const diagnostics = await lintCodeBlocks(source);
    expect(diagnostics).toMatchObject([
      {
        ruleId: "code-block-trailing-spaces",
        line: 2,
        language: "ts",
      },
    ]);
  });
});
