import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { extractDocs, generateMarkdown, resolveDocsOptions, writeDocs } from "./docs";
import type { ExtractedDocs } from "./types";

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("writeDocs", () => {
  it("removes stale generated files from the previous manifest", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-docs-"));
    tempDirs.push(outDir);

    await writeDocs(
      {
        "alpha.md": "# Alpha",
        "beta.md": "# Beta",
      },
      outDir,
    );

    await writeDocs(
      {
        "beta.md": "# Beta updated",
      },
      outDir,
    );

    await expect(fs.access(path.join(outDir, "alpha.md"))).rejects.toThrow();
    await expect(fs.readFile(path.join(outDir, "beta.md"), "utf-8")).resolves.toContain("updated");
  });

  it("does not delete files that were never tracked by the manifest", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-docs-"));
    tempDirs.push(outDir);

    await fs.writeFile(path.join(outDir, "manual.md"), "# Manual", "utf-8");

    await writeDocs(
      {
        "generated.md": "# Generated",
      },
      outDir,
    );

    await writeDocs(
      {
        "generated.md": "# Generated again",
      },
      outDir,
    );

    await expect(fs.readFile(path.join(outDir, "manual.md"), "utf-8")).resolves.toContain("Manual");
  });

  it("writes machine-readable docs data when extracted docs are provided", async () => {
    const outDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-docs-"));
    tempDirs.push(outDir);

    const extractedDocs: ExtractedDocs[] = [
      {
        file: "/repo/src/math.ts",
        entries: [
          {
            name: "clamp",
            kind: "function",
            description: "Clamps a number.",
            file: "/repo/src/math.ts",
            line: 10,
            endLine: 10,
            signature: "export function clamp(value: number, min: number, max: number): number",
          },
        ],
      },
    ];

    await writeDocs(
      { "math.md": "# math" },
      outDir,
      extractedDocs,
      resolveDocsOptions({ generateNav: true }),
    );

    const docsJson = JSON.parse(await fs.readFile(path.join(outDir, "docs.json"), "utf-8")) as {
      version: number;
      summary: {
        modules: number;
        entries: number;
        byKind: Record<string, number>;
      };
      modules: ExtractedDocs[];
    };

    expect(docsJson.version).toBe(1);
    expect(docsJson.summary).toMatchObject({
      modules: 1,
      entries: 1,
      byKind: {
        function: 1,
      },
    });
    expect(docsJson.modules[0]?.file).toBe("src/math.ts");
    expect(docsJson.modules[0]?.entries[0]?.file).toBe("src/math.ts");
    expect(docsJson.modules[0]?.entries[0]?.name).toBe("clamp");
  });
});

describe("generateMarkdown", () => {
  it("emits overview lines and accordion-style details for file docs", () => {
    const docs: ExtractedDocs[] = [
      {
        file: "/repo/src/utils.ts",
        entries: [
          {
            name: "capitalize",
            kind: "function",
            description: "Capitalizes the first letter of a string.",
            file: "/repo/src/utils.ts",
            line: 4,
            endLine: 4,
            signature: "export function capitalize(str: string): string",
            params: [{ name: "str", type: "string", description: "Input string" }],
            returns: { type: "string", description: "Capitalized string" },
          },
        ],
      },
    ];

    const markdown = generateMarkdown(docs, resolveDocsOptions({})!);

    expect(markdown["utils.md"]).not.toContain("## Overview");
    expect(markdown["utils.md"]).toContain("## Reference");
    expect(markdown["utils.md"]).toContain('<details id="capitalize" class="ox-api-entry">');
    expect(markdown["utils.md"]).toContain("Read the signatures first");
    expect(markdown["index.md"]).toContain("`@api transform`");
  });

  it("extracts declaration line ranges for source links", async () => {
    const srcDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-docs-src-"));
    tempDirs.push(srcDir);

    const filePath = path.join(srcDir, "range.ts");
    await fs.writeFile(
      filePath,
      `/**
 * Adds one to the input.
 */
export function addOne(value: number): number {
  const label = \`value:\${value}\`;
  return Number(label.split(":")[1]) + 1;
}
`,
      "utf-8",
    );

    const docs = await extractDocs([srcDir], resolveDocsOptions({ include: ["**/*.ts"] })!);

    expect(docs[0]?.entries[0]).toMatchObject({
      line: 4,
      endLine: 7,
    });
  });

  it("extracts JSDoc types from JavaScript files by default", async () => {
    const srcDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-docs-src-"));
    tempDirs.push(srcDir);

    await fs.writeFile(
      path.join(srcDir, "labels.js"),
      `/**
 * Creates a user-facing label.
 *
 * @param {string} value - The label source
 * @param {number} [maxLength=20] - Maximum length before truncation
 * @returns {string} Formatted label
 */
export function label(value, maxLength = 20) {
  return value.slice(0, maxLength);
}
`,
      "utf-8",
    );

    const docs = await extractDocs([srcDir], resolveDocsOptions({})!);
    const entry = docs[0]?.entries[0];

    expect(entry).toMatchObject({
      name: "label",
      kind: "function",
      description: "Creates a user-facing label.",
      params: [
        {
          name: "value",
          type: "string",
          description: "The label source",
        },
        {
          name: "maxLength",
          type: "number",
          description: "Maximum length before truncation",
          optional: true,
          default: "20",
        },
      ],
      returns: {
        type: "string",
        description: "Formatted label",
      },
    });
  });

  it("groups docs by public API entry points", async () => {
    const srcDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-docs-src-"));
    tempDirs.push(srcDir);

    await fs.writeFile(
      path.join(srcDir, "index.ts"),
      `export { add as sum } from "./math";
export type { Options } from "./types";
`,
      "utf-8",
    );
    await fs.writeFile(
      path.join(srcDir, "math.ts"),
      `/** Adds two numbers. */
export function add(a: number, b: number): number {
  return a + b;
}
`,
      "utf-8",
    );
    await fs.writeFile(
      path.join(srcDir, "types.ts"),
      `/** Runtime options. */
export interface Options {
  value: string;
}
`,
      "utf-8",
    );

    const docs = await extractDocs(
      [],
      resolveDocsOptions({
        entryPoints: [{ path: path.join(srcDir, "index.ts"), name: "default" }],
      })!,
    );

    expect(docs).toHaveLength(1);
    expect(docs[0]?.file).toBe("default");
    expect(docs[0]?.entries.map((entry) => entry.name)).toEqual(["sum", "Options"]);
  });

  it("extracts and renders highlighted interface signatures with generics", async () => {
    const srcDir = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-docs-src-"));
    tempDirs.push(srcDir);

    const filePath = path.join(srcDir, "interface.ts");
    await fs.writeFile(
      filePath,
      `/**
 * Render context shared by the renderer.
 */
export interface RenderContext<TValue extends Record<string, unknown> = Record<string, unknown>> extends AsyncDisposable {
  readonly value: TValue;
}
`,
      "utf-8",
    );

    const docs = await extractDocs([srcDir], resolveDocsOptions({ include: ["**/*.ts"] })!);
    const entry = docs[0]?.entries[0];

    expect(entry).toMatchObject({
      kind: "interface",
      name: "RenderContext",
      signature:
        "export interface RenderContext<TValue extends Record<string, unknown> = Record<string, unknown>> extends AsyncDisposable",
    });

    const markdown = generateMarkdown(docs, resolveDocsOptions({})!);

    expect(markdown["interface.md"]).toContain("RenderContext&lt;TValue extends");
    expect(markdown["interface.md"]).toContain("ox-api-entry__signature--highlighted");
    expect(markdown["index.md"]).toContain("ox-api-module__signature--highlighted");
  });
});
