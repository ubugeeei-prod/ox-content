import { describe, expect, it } from "vite-plus/test";
import { readFileSync } from "node:fs";
import { verifyPublishWorkflow } from "./verify-publish-targets";

describe("publish workflow targets", () => {
  it("includes @ox-content/code-play and every non-theme npm workspace dir", () => {
    const workflow = readFileSync(".github/workflows/publish.yml", "utf8");
    expect(workflow).toContain("working-directory: npm/ox-content-code-play");
    expect(workflow).toContain("Publish @ox-content/code-play");

    verifyPublishWorkflow({
      root: process.cwd(),
      workflowRel: ".github/workflows/publish.yml",
      cargoPackages: ["ox_content_parser"],
      npmPackages: [
        "npm/ox-content-islands",
        "npm/ox-content-code-play",
        "npm/unplugin-ox-content",
        "npm/vite-plugin-ox-content",
        "npm/vite-plugin-ox-content-react",
        "npm/vite-plugin-ox-content-solid",
        "npm/vite-plugin-ox-content-svelte",
        "npm/vite-plugin-ox-content-vue",
        "npm/vscode-ox-content",
      ],
    });
  });

  it("builds @ox-content/code-play before the Void docs site", () => {
    const script = readFileSync("scripts/deploy-docs-to-void.mjs", "utf8");
    expect(script).toContain('cwd: "npm/ox-content-code-play"');
    expect(script).toMatch(/ox-content-code-play[\s\S]*docs/);
  });

  it("fails when an npm workspace dir is missing from the workflow", () => {
    expect(() =>
      verifyPublishWorkflow({
        root: process.cwd(),
        workflowRel: ".github/workflows/publish.yml",
        cargoPackages: [],
        npmPackages: ["npm/ox-content-missing"],
      }),
    ).toThrow(/npm=npm\/ox-content-missing/);
  });
});
